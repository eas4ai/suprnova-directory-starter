use super::{
    EDIT_PERMISSION,
    entities::{article, revision, revision_term, slug, term},
    require_permission,
};
use crate::listings::{
    database_error, invalid, missing,
    queries::{Page, Pagination},
};
use sea_orm::{
    ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
    sea_query::Expr,
};
use serde::Serialize;
use std::collections::HashMap;
use suprnova::{DB, FrameworkError};

pub fn published() -> Condition {
    Condition::all().add(article::Column::PublishedRevisionId.is_not_null())
        .add(article::Column::PublishedAt.is_not_null())
        .add(Expr::cust("EXISTS (SELECT 1 FROM article_revisions published WHERE published.id = articles.published_revision_id AND published.article_id = articles.id)"))
}

#[derive(Clone, Serialize)]
pub struct Term {
    pub id: i64,
    pub kind: String,
    pub slug: String,
    pub name: String,
    pub active: bool,
}
impl From<term::Model> for Term {
    fn from(row: term::Model) -> Self {
        Self {
            id: row.id,
            kind: row.kind,
            slug: row.slug,
            name: row.name,
            active: row.active,
        }
    }
}
#[derive(Serialize)]
pub struct ArticleRevision {
    pub id: i64,
    pub slug: String,
    pub title: String,
    pub summary: String,
    pub body: String,
    pub body_html: String,
    pub media_id: Option<String>,
    pub media_url: Option<String>,
    pub media_alt: String,
    pub terms: Vec<Term>,
}
#[derive(Serialize)]
pub struct EditorArticle {
    pub id: i64,
    pub version: i64,
    pub status: String,
    pub public_url: Option<String>,
    pub current: ArticleRevision,
    pub published_revision_id: Option<i64>,
}
#[derive(Serialize)]
pub struct ArticleSummary {
    pub id: i64,
    pub version: i64,
    pub title: String,
    pub slug: String,
    pub status: String,
    pub updated_at: i64,
}
#[derive(Serialize)]
pub struct PublicArticle {
    pub id: i64,
    pub slug: String,
    pub title: String,
    pub summary: String,
    pub media_url: Option<String>,
    pub media_alt: String,
    pub terms: Vec<Term>,
    pub published_at: i64,
    pub modified_at: i64,
}
#[derive(Serialize)]
pub struct PublicDetail {
    #[serde(flatten)]
    pub card: PublicArticle,
    pub body_html: String,
}

fn status(row: &article::Model) -> &'static str {
    match row.published_revision_id {
        None => "draft",
        Some(id) if Some(id) == row.current_revision_id => "published",
        Some(_) => "unpublished_changes",
    }
}

struct Revisions {
    rows: HashMap<i64, revision::Model>,
    terms: HashMap<i64, Vec<Term>>,
}
impl Revisions {
    async fn load(ids: Vec<i64>) -> Result<Self, FrameworkError> {
        if ids.is_empty() {
            return Ok(Self {
                rows: HashMap::new(),
                terms: HashMap::new(),
            });
        }
        let db = DB::connection()?;
        let rows = revision::Entity::find()
            .filter(revision::Column::Id.is_in(ids.clone()))
            .all(db.inner())
            .await
            .map_err(database_error)?
            .into_iter()
            .map(|row| (row.id, row))
            .collect();
        let links = revision_term::Entity::find()
            .filter(revision_term::Column::RevisionId.is_in(ids))
            .all(db.inner())
            .await
            .map_err(database_error)?;
        let mut result = HashMap::<i64, Vec<Term>>::new();
        if !links.is_empty() {
            let terms: HashMap<_, _> = term::Entity::find()
                .filter(
                    term::Column::Id
                        .is_in(links.iter().map(|link| link.term_id).collect::<Vec<_>>()),
                )
                .all(db.inner())
                .await
                .map_err(database_error)?
                .into_iter()
                .map(|row| (row.id, Term::from(row)))
                .collect();
            for link in links {
                if let Some(term) = terms.get(&link.term_id) {
                    result
                        .entry(link.revision_id)
                        .or_default()
                        .push(term.clone());
                }
            }
            for terms in result.values_mut() {
                terms.sort_by(|a, b| (&a.kind, &a.name, a.id).cmp(&(&b.kind, &b.name, b.id)));
            }
        }
        Ok(Self {
            rows,
            terms: result,
        })
    }
    fn row(&self, id: Option<i64>, article: i64) -> Result<&revision::Model, FrameworkError> {
        self.rows
            .get(&id.ok_or_else(missing)?)
            .filter(|row| row.article_id == article)
            .ok_or_else(missing)
    }
    fn card(&self, row: &article::Model) -> Result<PublicArticle, FrameworkError> {
        let revision = self.row(row.published_revision_id, row.id)?;
        Ok(PublicArticle {
            id: row.id,
            slug: row.slug.clone(),
            title: revision.title.clone(),
            summary: revision.summary.clone(),
            media_url: revision
                .media_id
                .as_ref()
                .map(|id| format!("/media/articles/{}/{id}", row.slug)),
            media_alt: revision.media_alt.clone(),
            terms: self
                .terms
                .get(&revision.id)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter(|t| t.active)
                .collect(),
            published_at: row.published_at.ok_or_else(missing)?,
            modified_at: std::cmp::max(revision.created_at, row.published_at.ok_or_else(missing)?),
        })
    }
}

pub async fn editor(actor: i64, id: i64) -> Result<EditorArticle, FrameworkError> {
    require_permission(actor, EDIT_PERMISSION).await?;
    let db = DB::connection()?;
    let row = article::Entity::find_by_id(id)
        .one(db.inner())
        .await
        .map_err(database_error)?
        .ok_or_else(missing)?;
    let bundle = Revisions::load(row.current_revision_id.into_iter().collect()).await?;
    let current = bundle.row(row.current_revision_id, id)?;
    Ok(EditorArticle {
        id,
        version: row.version,
        status: status(&row).into(),
        public_url: row
            .published_revision_id
            .map(|_| format!("/articles/{}", row.slug)),
        published_revision_id: row.published_revision_id,
        current: ArticleRevision {
            id: current.id,
            slug: current.slug.clone(),
            title: current.title.clone(),
            summary: current.summary.clone(),
            body: current.body.clone(),
            body_html: render_body(&current.body)?,
            media_id: current.media_id.clone(),
            media_url: current
                .media_id
                .as_ref()
                .map(|id| format!("/admin/articles/media/{id}")),
            media_alt: current.media_alt.clone(),
            terms: bundle.terms.get(&current.id).cloned().unwrap_or_default(),
        },
    })
}

pub async fn admin_search(
    actor: i64,
    q: &str,
    state: &str,
    page: Page,
) -> Result<(Vec<ArticleSummary>, Pagination), FrameworkError> {
    require_permission(actor, EDIT_PERMISSION).await?;
    validate_search(q, "", "")?;
    let db = DB::connection()?;
    let mut query = article::Entity::find();
    match state {
        "" => {}
        "published" => {
            query = query.filter(published());
        }
        "draft" => {
            query = query.filter(
                Condition::any()
                    .add(article::Column::PublishedRevisionId.is_null())
                    .add(Expr::cust(
                        "articles.published_revision_id <> articles.current_revision_id",
                    )),
            );
        }
        _ => return Err(invalid("state", "Choose all, draft or published articles.")),
    }
    if !q.trim().is_empty() {
        query = query.filter(Expr::cust_with_values("EXISTS (SELECT 1 FROM article_revisions r WHERE r.id = articles.current_revision_id AND r.search_text LIKE ? ESCAPE '!')", [pattern(q)]));
    }
    let total = query
        .clone()
        .count(db.inner())
        .await
        .map_err(database_error)?;
    let rows = query
        .order_by_desc(article::Column::UpdatedAt)
        .order_by_desc(article::Column::Id)
        .offset((page.number - 1) * page.size)
        .limit(page.size)
        .all(db.inner())
        .await
        .map_err(database_error)?;
    let bundle = Revisions::load(
        rows.iter()
            .filter_map(|row| row.current_revision_id)
            .collect(),
    )
    .await?;
    let articles = rows
        .iter()
        .map(|row| {
            let current = bundle.row(row.current_revision_id, row.id)?;
            Ok(ArticleSummary {
                id: row.id,
                version: row.version,
                title: current.title.clone(),
                slug: current.slug.clone(),
                status: status(row).into(),
                updated_at: row.updated_at,
            })
        })
        .collect::<Result<Vec<_>, FrameworkError>>()?;
    Ok((
        articles,
        Pagination {
            page: page.number,
            per_page: page.size,
            total,
        },
    ))
}

fn pattern(value: &str) -> String {
    format!(
        "%{}%",
        value
            .trim()
            .to_lowercase()
            .replace('!', "!!")
            .replace('%', "!%")
            .replace('_', "!_")
    )
}
fn validate_search(q: &str, category: &str, tag: &str) -> Result<(), FrameworkError> {
    if q.chars().count() > 200 || q.contains('\0') {
        return Err(invalid("q", "Search must be at most 200 characters."));
    }
    for (field, value) in [("category", category), ("tag", tag)] {
        if !value.is_empty() && !super::validation::valid_slug(value) {
            return Err(invalid(field, "Choose a valid category or tag."));
        }
    }
    Ok(())
}

pub async fn search(
    q: &str,
    category: &str,
    tag: &str,
    page: Page,
) -> Result<(Vec<PublicArticle>, Pagination), FrameworkError> {
    validate_search(q, category, tag)?;
    let db = DB::connection()?;
    let mut query = article::Entity::find().filter(published());
    if !q.trim().is_empty() {
        query = query.filter(Expr::cust_with_values("EXISTS (SELECT 1 FROM article_revisions r WHERE r.id = articles.published_revision_id AND r.search_text LIKE ? ESCAPE '!')", [pattern(q)]));
    }
    for (kind, slug) in [("category", category), ("tag", tag)] {
        if !slug.is_empty() {
            query = query.filter(Expr::cust_with_values("EXISTS (SELECT 1 FROM article_revision_terms rt JOIN article_terms t ON t.id = rt.term_id WHERE rt.revision_id = articles.published_revision_id AND t.active = TRUE AND t.kind = ? AND t.slug = ?)", [kind, slug]));
        }
    }
    let total = query
        .clone()
        .count(db.inner())
        .await
        .map_err(database_error)?;
    let rows = query
        .order_by_desc(article::Column::PublishedAt)
        .order_by_desc(article::Column::Id)
        .offset((page.number - 1) * page.size)
        .limit(page.size)
        .all(db.inner())
        .await
        .map_err(database_error)?;
    let bundle = Revisions::load(
        rows.iter()
            .filter_map(|row| row.published_revision_id)
            .collect(),
    )
    .await?;
    Ok((
        rows.iter()
            .map(|row| bundle.card(row))
            .collect::<Result<Vec<_>, _>>()?,
        Pagination {
            page: page.number,
            per_page: page.size,
            total,
        },
    ))
}

pub async fn public_article(name: &str) -> Result<article::Model, FrameworkError> {
    if !super::validation::valid_slug(name) {
        return Err(missing());
    }
    let db = DB::connection()?;
    let alias = slug::Entity::find_by_id(name)
        .one(db.inner())
        .await
        .map_err(database_error)?
        .ok_or_else(missing)?;
    article::Entity::find_by_id(alias.article_id)
        .filter(published())
        .one(db.inner())
        .await
        .map_err(database_error)?
        .ok_or_else(missing)
}

pub async fn detail(name: &str) -> Result<PublicDetail, FrameworkError> {
    let row = public_article(name).await?;
    let bundle = Revisions::load(row.published_revision_id.into_iter().collect()).await?;
    let revision = bundle.row(row.published_revision_id, row.id)?;
    Ok(PublicDetail {
        card: bundle.card(&row)?,
        body_html: render_body(&revision.body)?,
    })
}

pub async fn terms(public_only: bool) -> Result<Vec<Term>, FrameworkError> {
    let db = DB::connection()?;
    let mut query = term::Entity::find();
    if public_only {
        query = query.filter(term::Column::Active.eq(true)).filter(Expr::cust("EXISTS (SELECT 1 FROM article_revision_terms rt JOIN articles a ON a.published_revision_id = rt.revision_id JOIN article_revisions r ON r.id = rt.revision_id AND r.article_id = a.id WHERE rt.term_id = article_terms.id AND a.published_at IS NOT NULL)"));
    }
    Ok(query
        .order_by_asc(term::Column::Kind)
        .order_by_asc(term::Column::Name)
        .order_by_asc(term::Column::Id)
        .limit(1000)
        .all(db.inner())
        .await
        .map_err(database_error)?
        .into_iter()
        .map(Into::into)
        .collect())
}

pub fn render_body(body: &str) -> Result<String, FrameworkError> {
    // Reuse the verified Markdown renderer and active-content policy.
    crate::listings::queries::render_description(body)
}
