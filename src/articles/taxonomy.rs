use super::{
    TAXONOMY_PERMISSION,
    entities::{revision_term, term},
    require_permission,
    validation::valid_slug,
};
use crate::listings::{
    conflict, database_error,
    entities::{category, revision_category},
    invalid, missing,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, ExprTrait, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, Set, TransactionTrait, sea_query::Expr,
};
use serde::{Deserialize, Serialize};
use suprnova::{DB, FrameworkError};

#[derive(Serialize)]
pub struct TaxonomyItem {
    pub id: i64,
    pub kind: String,
    pub slug: String,
    pub name: String,
    pub active: bool,
    pub version: i64,
    pub seo: crate::seo::Overrides,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SaveTerm {
    pub version: i64,
    #[serde(default)]
    pub seo: crate::seo::Overrides,
    pub slug: String,
    pub name: String,
    pub active: bool,
}

fn validate_kind(kind: &str) -> Result<(), FrameworkError> {
    if !matches!(kind, "listing_category" | "category" | "tag") {
        return Err(invalid(
            "kind",
            "Choose listing category, article category or article tag.",
        ));
    }
    Ok(())
}

pub async fn list(actor: i64, kind: &str) -> Result<Vec<TaxonomyItem>, FrameworkError> {
    require_permission(actor, TAXONOMY_PERMISSION).await?;
    validate_kind(kind)?;
    let db = DB::connection()?;
    if kind == "listing_category" {
        return category::Entity::find()
            .order_by_asc(category::Column::Name)
            .order_by_asc(category::Column::Id)
            .limit(1000)
            .all(db.inner())
            .await
            .map_err(database_error)?
            .into_iter()
            .map(|row| {
                Ok(TaxonomyItem {
                    id: row.id,
                    kind: kind.into(),
                    slug: row.slug,
                    name: row.name,
                    active: row.active,
                    version: row.version,
                    seo: crate::seo::Overrides::decode(&row.seo)?,
                })
            })
            .collect::<Result<Vec<_>, FrameworkError>>();
    }
    term::Entity::find()
        .filter(term::Column::Kind.eq(kind))
        .order_by_asc(term::Column::Name)
        .order_by_asc(term::Column::Id)
        .limit(1000)
        .all(db.inner())
        .await
        .map_err(database_error)?
        .into_iter()
        .map(|row| {
            Ok(TaxonomyItem {
                id: row.id,
                kind: row.kind,
                slug: row.slug,
                name: row.name,
                active: row.active,
                version: row.version,
                seo: crate::seo::Overrides::decode(&row.seo)?,
            })
        })
        .collect::<Result<Vec<_>, FrameworkError>>()
}

pub async fn save(
    actor: i64,
    kind: &str,
    id: Option<i64>,
    mut input: SaveTerm,
) -> Result<i64, FrameworkError> {
    require_permission(actor, TAXONOMY_PERMISSION).await?;
    validate_kind(kind)?;
    input.seo = input.seo.validate()?;
    input.name = input.name.trim().into();
    input.slug = input.slug.trim().into();
    if input.name.is_empty() || input.name.chars().count() > 100 || input.name.contains('\0') {
        return Err(invalid(
            "name",
            "Enter a name between 1 and 100 characters.",
        ));
    }
    if !valid_slug(&input.slug) {
        return Err(invalid(
            "slug",
            "Use a lowercase slug of at most 120 letters, digits and hyphens.",
        ));
    }
    if input.version < 0 || (id.is_none() && input.version != 0) {
        return Err(conflict());
    }
    let db = DB::connection()?;
    let tx = db.inner().begin().await.map_err(database_error)?;
    crate::accounts::guard_permission(&tx, actor, TAXONOMY_PERMISSION).await?;
    tx.execute_unprepared("UPDATE taxonomy_write_lock SET version = version + 1 WHERE id = 1")
        .await
        .map_err(database_error)?;
    let saved = if kind == "listing_category" {
        if let Some(id) = id {
            let row = category::Entity::find_by_id(id)
                .one(&tx)
                .await
                .map_err(database_error)?
                .ok_or_else(missing)?;
            if row.version != input.version {
                return Err(conflict());
            }
            if row.slug != input.slug {
                return Err(invalid(
                    "slug",
                    "Existing taxonomy URLs are stable. Rename the displayed name instead.",
                ));
            }
            category::Entity::update_many()
                .col_expr(
                    category::Column::Version,
                    Expr::col(category::Column::Version).add(1),
                )
                .col_expr(category::Column::Seo, Expr::value(input.seo.encode()?))
                .col_expr(category::Column::Name, Expr::value(input.name))
                .col_expr(category::Column::Active, Expr::value(input.active))
                .filter(category::Column::Id.eq(id))
                .exec(&tx)
                .await
                .map_err(database_error)?;
            id
        } else {
            if category::Entity::find()
                .count(&tx)
                .await
                .map_err(database_error)?
                >= 1000
            {
                return Err(invalid(
                    "name",
                    "At most 1000 listing categories are supported.",
                ));
            }
            if category::Entity::find()
                .filter(category::Column::Slug.eq(&input.slug))
                .one(&tx)
                .await
                .map_err(database_error)?
                .is_some()
            {
                return Err(invalid("slug", "This slug is already in use."));
            }
            category::ActiveModel {
                seo: Set(input.seo.encode()?),
                slug: Set(input.slug),
                name: Set(input.name),
                active: Set(input.active),
                version: Set(1),
                ..Default::default()
            }
            .insert(&tx)
            .await
            .map_err(database_error)?
            .id
        }
    } else if let Some(id) = id {
        let row = term::Entity::find_by_id(id)
            .filter(term::Column::Kind.eq(kind))
            .one(&tx)
            .await
            .map_err(database_error)?
            .ok_or_else(missing)?;
        if row.version != input.version {
            return Err(conflict());
        }
        if row.slug != input.slug {
            return Err(invalid(
                "slug",
                "Existing taxonomy URLs are stable. Rename the displayed name instead.",
            ));
        }
        term::Entity::update_many()
            .col_expr(
                term::Column::Version,
                Expr::col(term::Column::Version).add(1),
            )
            .col_expr(term::Column::Seo, Expr::value(input.seo.encode()?))
            .col_expr(term::Column::Name, Expr::value(input.name))
            .col_expr(term::Column::Active, Expr::value(input.active))
            .filter(term::Column::Id.eq(id))
            .exec(&tx)
            .await
            .map_err(database_error)?;
        id
    } else {
        if term::Entity::find()
            .count(&tx)
            .await
            .map_err(database_error)?
            >= 1000
        {
            return Err(invalid(
                "name",
                "At most 1000 article categories and tags are supported.",
            ));
        }
        if term::Entity::find()
            .filter(term::Column::Kind.eq(kind))
            .filter(term::Column::Slug.eq(&input.slug))
            .one(&tx)
            .await
            .map_err(database_error)?
            .is_some()
        {
            return Err(invalid("slug", "This slug is already in use."));
        }
        term::ActiveModel {
            seo: Set(input.seo.encode()?),
            kind: Set(kind.into()),
            slug: Set(input.slug),
            name: Set(input.name),
            active: Set(input.active),
            version: Set(1),
            ..Default::default()
        }
        .insert(&tx)
        .await
        .map_err(database_error)?
        .id
    };
    crate::audit::record(
        &tx,
        actor,
        if kind == "listing_category" {
            "listing_category"
        } else {
            "article_term"
        },
        saved.to_string(),
        "taxonomy_saved",
        "Changed taxonomy name and enablement; retained existing content relationships.",
    )
    .await?;
    tx.commit().await.map_err(database_error)?;
    Ok(saved)
}

/// In-use terms must be disabled instead; no implicit detach can alter content.
pub async fn remove(actor: i64, kind: &str, id: i64, version: i64) -> Result<(), FrameworkError> {
    require_permission(actor, TAXONOMY_PERMISSION).await?;
    validate_kind(kind)?;
    let db = DB::connection()?;
    let tx = db.inner().begin().await.map_err(database_error)?;
    crate::accounts::guard_permission(&tx, actor, TAXONOMY_PERMISSION).await?;
    tx.execute_unprepared("UPDATE taxonomy_write_lock SET version = version + 1 WHERE id = 1")
        .await
        .map_err(database_error)?;
    if kind == "listing_category" {
        let row = category::Entity::find_by_id(id)
            .one(&tx)
            .await
            .map_err(database_error)?
            .ok_or_else(missing)?;
        if row.version != version {
            return Err(conflict());
        }
        if revision_category::Entity::find()
            .filter(revision_category::Column::CategoryId.eq(id))
            .one(&tx)
            .await
            .map_err(database_error)?
            .is_some()
        {
            return Err(invalid(
                "term",
                "This category is used by listing history. Disable it to retain those relationships.",
            ));
        }
        category::Entity::delete_by_id(id)
            .exec(&tx)
            .await
            .map_err(database_error)?;
    } else {
        let row = term::Entity::find_by_id(id)
            .filter(term::Column::Kind.eq(kind))
            .one(&tx)
            .await
            .map_err(database_error)?
            .ok_or_else(missing)?;
        if row.version != version {
            return Err(conflict());
        }
        if revision_term::Entity::find()
            .filter(revision_term::Column::TermId.eq(id))
            .one(&tx)
            .await
            .map_err(database_error)?
            .is_some()
        {
            return Err(invalid(
                "term",
                "This term is used by article history. Disable it to retain those relationships.",
            ));
        }
        term::Entity::delete_by_id(id)
            .exec(&tx)
            .await
            .map_err(database_error)?;
    }
    crate::audit::record(
        &tx,
        actor,
        if kind == "listing_category" {
            "listing_category"
        } else {
            "article_term"
        },
        id.to_string(),
        "taxonomy_removed",
        "Removed an unused taxonomy term.",
    )
    .await?;
    tx.commit().await.map_err(database_error)
}
