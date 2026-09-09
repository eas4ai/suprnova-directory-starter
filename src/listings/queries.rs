use std::collections::HashMap;

use sea_orm::{
    ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
    QueryTrait, sea_query::Expr,
};
use serde::Serialize;
use suprnova::{DB, FrameworkError};

use super::{
    database_error,
    entities::{category, entitlement, listing, revision, revision_category},
    invalid, missing,
};

#[derive(Clone, Serialize)]
pub struct Category {
    pub id: i64,
    pub slug: String,
    pub name: String,
}

#[derive(Serialize)]
pub struct Pagination {
    pub page: u64,
    pub per_page: u64,
    pub total: u64,
}

#[derive(Clone, Copy)]
pub struct Page {
    pub number: u64,
    pub size: u64,
}
impl Page {
    pub fn parse(number: Option<String>, size: Option<String>) -> Result<Self, FrameworkError> {
        let parse = |value: Option<String>, default: u64| {
            value
                .map(|v| v.parse::<u64>())
                .transpose()
                .map(|v| v.unwrap_or(default))
                .map_err(|_| invalid("page", "Enter a valid page number and size."))
        };
        let page = Self {
            number: parse(number, 1)?,
            size: parse(size, 24)?,
        };
        if page.number == 0 || page.number > 1_000_000 || page.size == 0 || page.size > 100 {
            return Err(invalid(
                "page",
                "Page size must be between 1 and 100; page must be between 1 and 1000000.",
            ));
        }
        Ok(page)
    }
    fn pagination(self, total: u64) -> Pagination {
        Pagination {
            page: self.number,
            per_page: self.size,
            total,
        }
    }
}

/// Every public listing surface uses this predicate at the current request time.
/// Content approval alone never grants publication, and test payments stay private.
pub fn eligible(now: i64) -> Condition {
    Condition::all().add(listing::Column::ApprovedRevisionId.is_not_null())
        .add(listing::Column::Archived.eq(false)).add(listing::Column::Suspended.eq(false))
        .add(Expr::cust("NOT EXISTS (SELECT 1 FROM account_access a WHERE a.user_id = listings.owner_id AND a.suspended = TRUE)"))
          .add(Expr::cust("EXISTS (SELECT 1 FROM listing_revisions approved WHERE approved.id = listings.approved_revision_id AND approved.listing_id = listings.id AND approved.status = 'approved')"))
        .add(Expr::cust_with_values("EXISTS (SELECT 1 FROM publication_entitlements e WHERE e.listing_id = listings.id AND e.mode IN ('free', 'live') AND e.status = 'active' AND e.valid_from <= ? AND (e.valid_until IS NULL OR e.valid_until > ?))", [now, now]))
}

pub async fn categories() -> Result<Vec<Category>, FrameworkError> {
    let db = DB::connection()?;
    Ok(category::Entity::find()
        .filter(category::Column::Active.eq(true))
        .order_by_asc(category::Column::Name)
        .order_by_asc(category::Column::Id)
        .limit(1000)
        .all(db.inner())
        .await
        .map_err(database_error)?
        .into_iter()
        .map(|row| Category {
            id: row.id,
            slug: row.slug,
            name: row.name,
        })
        .collect())
}

/// Public taxonomy navigation uses the same eligibility predicate as results.
pub async fn public_categories(now: i64) -> Result<Vec<Category>, FrameworkError> {
    let revisions = listing::Entity::find()
        .select_only()
        .column(listing::Column::ApprovedRevisionId)
        .filter(eligible(now))
        .into_query();
    let terms = revision_category::Entity::find()
        .select_only()
        .column(revision_category::Column::CategoryId)
        .filter(revision_category::Column::RevisionId.in_subquery(revisions))
        .into_query();
    let db = DB::connection()?;
    Ok(category::Entity::find()
        .filter(category::Column::Active.eq(true))
        .filter(category::Column::Id.in_subquery(terms))
        .order_by_asc(category::Column::Name)
        .order_by_asc(category::Column::Id)
        .limit(1000)
        .all(db.inner())
        .await
        .map_err(database_error)?
        .into_iter()
        .map(|row| Category {
            id: row.id,
            slug: row.slug,
            name: row.name,
        })
        .collect())
}

#[derive(Serialize)]
pub struct PublicCard {
    pub id: i64,
    pub slug: String,
    pub title: String,
    pub summary: String,
    pub url: String,
    pub media_url: Option<String>,
    pub media_alt: String,
    pub categories: Vec<Category>,
    pub created_at: i64,
}

#[derive(Serialize)]
pub struct PublicDetail {
    #[serde(flatten)]
    pub card: PublicCard,
    pub description_html: String,
}

pub async fn search(
    q: &str,
    category_slug: &str,
    page: Page,
    now: i64,
) -> Result<(Vec<PublicCard>, Pagination), FrameworkError> {
    if q.chars().count() > 200 {
        return Err(invalid("q", "Search must be at most 200 characters."));
    }
    if category_slug.len() > 140
        || !category_slug
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-')
    {
        return Err(invalid("category", "Choose a valid category."));
    }
    let db = DB::connection()?;
    let mut query = listing::Entity::find().filter(eligible(now));
    if !q.trim().is_empty() {
        let pattern = format!(
            "%{}%",
            q.trim()
                .to_lowercase()
                .replace('!', "!!")
                .replace('%', "!%")
                .replace('_', "!_")
        );
        query = query.filter(Expr::cust_with_values("EXISTS (SELECT 1 FROM listing_revisions r WHERE r.id = listings.approved_revision_id AND r.search_text LIKE ? ESCAPE '!')", [pattern]));
    }
    if !category_slug.is_empty() {
        query = query.filter(Expr::cust_with_values("EXISTS (SELECT 1 FROM listing_revision_categories rc JOIN listing_categories c ON c.id = rc.category_id WHERE rc.revision_id = listings.approved_revision_id AND c.active = TRUE AND c.slug = ?)", [category_slug]));
    }
    let total = query
        .clone()
        .count(db.inner())
        .await
        .map_err(database_error)?;
    let rows = query
        .order_by_desc(listing::Column::CreatedAt)
        .order_by_desc(listing::Column::Id)
        .offset((page.number - 1) * page.size)
        .limit(page.size)
        .all(db.inner())
        .await
        .map_err(database_error)?;
    let bundle = Revisions::load(
        rows.iter()
            .filter_map(|row| row.approved_revision_id)
            .collect(),
    )
    .await?;
    let cards = rows
        .iter()
        .map(|row| bundle.public_card(row))
        .collect::<Result<Vec<_>, _>>()?;
    Ok((cards, page.pagination(total)))
}

pub async fn public_listing(slug: &str, now: i64) -> Result<listing::Model, FrameworkError> {
    let db = DB::connection()?;
    listing::Entity::find()
        .filter(listing::Column::Slug.eq(slug))
        .filter(eligible(now))
        .one(db.inner())
        .await
        .map_err(database_error)?
        .ok_or_else(missing)
}

pub async fn detail(slug: &str, now: i64) -> Result<PublicDetail, FrameworkError> {
    let row = public_listing(slug, now).await?;
    let bundle = Revisions::load(row.approved_revision_id.into_iter().collect()).await?;
    let approved = bundle
        .rows
        .get(&row.approved_revision_id.ok_or_else(missing)?)
        .ok_or_else(missing)?;
    let description_html = render_description(&approved.description)?;
    Ok(PublicDetail {
        card: bundle.public_card(&row)?,
        description_html,
    })
}

pub fn render_description(source: &str) -> Result<String, FrameworkError> {
    use suprnova::content::{MarkdownOptions, MarkdownRenderer};
    MarkdownRenderer::new(MarkdownOptions {
        unsafe_html: false,
        heading_anchor_prefix: "listing-".to_owned(),
        render_math: false,
    })
    .render(source)
    .map(|rendered| rendered.html)
    .map_err(|_| FrameworkError::internal("The listing description could not be rendered."))
}

#[derive(Serialize)]
pub struct Revision {
    pub id: i64,
    pub title: String,
    pub summary: String,
    pub description: String,
    pub url: String,
    pub category_ids: Vec<i64>,
    pub media_id: Option<String>,
    pub media_alt: String,
    pub status: String,
    pub reason: Option<String>,
    pub media_url: Option<String>,
}

#[derive(Serialize)]
pub struct OwnerListing {
    pub id: i64,
    pub slug: String,
    pub version: i64,
    pub archived: bool,
    pub suspended: bool,
    pub current: Revision,
    pub approved: Option<Revision>,
    pub moderation_status: String,
    pub payment_status: String,
    pub publication_status: String,
    pub next_action: String,
    pub purchase_id: Option<String>,
}

pub async fn owner_listing(actor_id: i64, id: i64) -> Result<OwnerListing, FrameworkError> {
    let db = DB::connection()?;
    let row = listing::Entity::find_by_id(id)
        .filter(listing::Column::OwnerId.eq(actor_id))
        .one(db.inner())
        .await
        .map_err(database_error)?
        .ok_or_else(missing)?;
    owner_views(vec![row], chrono::Utc::now().timestamp())
        .await?
        .pop()
        .ok_or_else(missing)
}

pub async fn review_listing(actor_id: i64, id: i64) -> Result<OwnerListing, FrameworkError> {
    super::workflow::require_moderator(actor_id).await?;
    let db = DB::connection()?;
    let row = listing::Entity::find_by_id(id)
        .one(db.inner())
        .await
        .map_err(database_error)?
        .ok_or_else(missing)?;
    owner_views(vec![row], chrono::Utc::now().timestamp())
        .await?
        .pop()
        .ok_or_else(missing)
}

pub async fn owner_search(
    actor_id: i64,
    page: Page,
) -> Result<(Vec<OwnerListing>, Pagination), FrameworkError> {
    let db = DB::connection()?;
    let query = listing::Entity::find().filter(listing::Column::OwnerId.eq(actor_id));
    let total = query
        .clone()
        .count(db.inner())
        .await
        .map_err(database_error)?;
    let rows = query
        .order_by_desc(listing::Column::CreatedAt)
        .order_by_desc(listing::Column::Id)
        .offset((page.number - 1) * page.size)
        .limit(page.size)
        .all(db.inner())
        .await
        .map_err(database_error)?;
    Ok((
        owner_views(rows, chrono::Utc::now().timestamp()).await?,
        page.pagination(total),
    ))
}

pub async fn review_queue(
    actor_id: i64,
    page: Page,
) -> Result<(Vec<OwnerListing>, Pagination), FrameworkError> {
    super::workflow::require_moderator(actor_id).await?;
    let db = DB::connection()?;
    let query = listing::Entity::find().filter(listing::Column::Archived.eq(false))
        .filter(Expr::cust("EXISTS (SELECT 1 FROM listing_revisions r WHERE r.id = listings.current_revision_id AND r.status = 'submitted')"));
    let total = query
        .clone()
        .count(db.inner())
        .await
        .map_err(database_error)?;
    let rows = query
        .order_by_asc(listing::Column::Id)
        .offset((page.number - 1) * page.size)
        .limit(page.size)
        .all(db.inner())
        .await
        .map_err(database_error)?;
    Ok((
        owner_views(rows, chrono::Utc::now().timestamp()).await?,
        page.pagination(total),
    ))
}

struct Revisions {
    rows: HashMap<i64, revision::Model>,
    categories: HashMap<i64, Vec<Category>>,
}

impl Revisions {
    async fn load(mut ids: Vec<i64>) -> Result<Self, FrameworkError> {
        ids.sort_unstable();
        ids.dedup();
        let mut bundle = Self {
            rows: HashMap::new(),
            categories: HashMap::new(),
        };
        if ids.is_empty() {
            return Ok(bundle);
        }
        let db = DB::connection()?;
        bundle.rows = revision::Entity::find()
            .filter(revision::Column::Id.is_in(ids.clone()))
            .all(db.inner())
            .await
            .map_err(database_error)?
            .into_iter()
            .map(|row| (row.id, row))
            .collect();
        let links = revision_category::Entity::find()
            .filter(revision_category::Column::RevisionId.is_in(ids))
            .order_by_asc(revision_category::Column::CategoryId)
            .all(db.inner())
            .await
            .map_err(database_error)?;
        let terms: HashMap<_, _> = category::Entity::find()
            .filter(
                category::Column::Id.is_in(
                    links
                        .iter()
                        .map(|link| link.category_id)
                        .collect::<Vec<_>>(),
                ),
            )
            .all(db.inner())
            .await
            .map_err(database_error)?
            .into_iter()
            .map(|row| (row.id, row))
            .collect();
        for link in links {
            if let Some(term) = terms.get(&link.category_id) {
                bundle
                    .categories
                    .entry(link.revision_id)
                    .or_default()
                    .push(Category {
                        id: term.id,
                        slug: term.slug.clone(),
                        name: term.name.clone(),
                    });
            }
        }
        Ok(bundle)
    }

    fn private_revision(&self, id: i64) -> Result<Revision, FrameworkError> {
        let row = self.rows.get(&id).ok_or_else(missing)?;
        Ok(Revision {
            id,
            title: row.title.clone(),
            summary: row.summary.clone(),
            description: row.description.clone(),
            url: row.url.clone(),
            category_ids: self
                .categories
                .get(&id)
                .into_iter()
                .flatten()
                .map(|term| term.id)
                .collect(),
            media_id: row.media_id.clone(),
            media_alt: row.media_alt.clone(),
            status: row.status.clone(),
            reason: row.reason.clone(),
            media_url: row
                .media_id
                .as_ref()
                .map(|id| format!("/dashboard/listings/media/{id}")),
        })
    }

    fn public_card(&self, listing: &listing::Model) -> Result<PublicCard, FrameworkError> {
        let id = listing.approved_revision_id.ok_or_else(missing)?;
        let row = self
            .rows
            .get(&id)
            .filter(|row| row.listing_id == listing.id && row.status == "approved")
            .ok_or_else(missing)?;
        Ok(PublicCard {
            id: listing.id,
            slug: listing.slug.clone(),
            title: row.title.clone(),
            summary: row.summary.clone(),
            url: row.url.clone(),
            media_url: row
                .media_id
                .as_ref()
                .map(|id| format!("/media/listings/{}/{id}", listing.slug)),
            media_alt: row.media_alt.clone(),
            categories: self.categories.get(&id).cloned().unwrap_or_default(),
            created_at: listing.created_at,
        })
    }
}

async fn owner_views(
    rows: Vec<listing::Model>,
    now: i64,
) -> Result<Vec<OwnerListing>, FrameworkError> {
    if rows.is_empty() {
        return Ok(Vec::new());
    }
    let bundle = Revisions::load(
        rows.iter()
            .flat_map(|row| {
                [row.current_revision_id, row.approved_revision_id]
                    .into_iter()
                    .flatten()
            })
            .collect(),
    )
    .await?;
    let db = DB::connection()?;
    let slots = crate::billing::lifecycle_entities::slot::Entity::find()
        .filter(
            crate::billing::lifecycle_entities::slot::Column::ListingId
                .is_in(rows.iter().map(|row| row.id).collect::<Vec<_>>()),
        )
        .all(db.inner())
        .await
        .map_err(database_error)?;
    let payments = entitlement::Entity::find()
        .filter(
            entitlement::Column::ListingId.is_in(rows.iter().map(|row| row.id).collect::<Vec<_>>()),
        )
        .filter(Expr::cust_with_values("publication_entitlements.id = (SELECT preferred.id FROM publication_entitlements preferred WHERE preferred.listing_id = publication_entitlements.listing_id ORDER BY CASE WHEN preferred.status = 'active' AND preferred.valid_from <= ? AND (preferred.valid_until IS NULL OR preferred.valid_until > ?) THEN CASE preferred.mode WHEN 'free' THEN 0 WHEN 'live' THEN 0 WHEN 'test' THEN 1 ELSE 2 END ELSE 2 END, preferred.updated_at DESC, preferred.id DESC LIMIT 1)", [now, now]))
        .order_by_desc(entitlement::Column::UpdatedAt)
        .order_by_desc(entitlement::Column::Id)
        .all(db.inner())
        .await
        .map_err(database_error)?;
    rows.into_iter()
        .map(|row| {
            let purchase_id = slots
                .iter()
                .find(|slot| slot.listing_id == row.id)
                .map(|slot| slot.purchase_id.clone());
            let current = bundle.private_revision(row.current_revision_id.ok_or_else(missing)?)?;
            let approved = row
                .approved_revision_id
                .map(|id| bundle.private_revision(id))
                .transpose()?;
            let for_listing: Vec<_> = payments
                .iter()
                .filter(|payment| payment.listing_id == row.id)
                .collect();
            let payment = for_listing
                .iter()
                .copied()
                .find(|payment| active(payment, now) && payment.mode != "test")
                .or_else(|| for_listing.first().copied());
            let payment_status = payment
                .map(|payment| {
                    if matches!(payment.status.as_str(), "disputed" | "revoked" | "refunded") {
                        payment.status.as_str()
                    } else if !active(payment, now) {
                        "expired"
                    } else if payment.mode == "test" {
                        "test"
                    } else if payment.mode == "free" {
                        "free"
                    } else {
                        "paid"
                    }
                })
                .unwrap_or(if purchase_id.is_some() {
                    "pending"
                } else {
                    "none"
                });
            let entitled = matches!(payment_status, "free" | "paid");
            let publication_status = if row.archived {
                "archived"
            } else if row.suspended {
                "suspended"
            } else if approved.is_some() && entitled {
                "published"
            } else if approved.is_none() {
                "awaiting_review"
            } else {
                match payment_status {
                    "test" => "test_payment",
                    "expired" => "expired",
                    "disputed" => "disputed",
                    "revoked" => "revoked",
                    "refunded" => "refunded",
                    _ => "awaiting_payment",
                }
            };
            let next_action = if row.archived {
                "none"
            } else if purchase_id.is_some() && current.status == "approved" {
                "manage_payment"
            } else if approved.is_some() && matches!(payment_status, "none" | "expired" | "revoked")
            {
                "checkout"
            } else {
                match current.status.as_str() {
                    "draft" => "submit",
                    "submitted" => "await_review",
                    "rejected" => "edit",
                    _ => {
                        if entitled {
                            "manage_payment"
                        } else {
                            "none"
                        }
                    }
                }
            };
            Ok(OwnerListing {
                id: row.id,
                slug: row.slug,
                version: row.version,
                archived: row.archived,
                suspended: row.suspended,
                moderation_status: current.status.clone(),
                current,
                approved,
                payment_status: payment_status.to_owned(),
                publication_status: publication_status.to_owned(),
                next_action: next_action.to_owned(),
                purchase_id,
            })
        })
        .collect()
}

fn active(payment: &entitlement::Model, now: i64) -> bool {
    payment.status == "active"
        && payment.valid_from <= now
        && payment.valid_until.is_none_or(|until| until > now)
}
