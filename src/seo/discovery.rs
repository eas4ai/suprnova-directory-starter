use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter, sea_query::Expr};
use suprnova::{DB, FrameworkError};

pub fn listing_indexable() -> Condition {
    Condition::all().add(Expr::cust("EXISTS (SELECT 1 FROM listing_revisions r WHERE r.id = listings.approved_revision_id AND COALESCE(json_extract(r.seo, '$.noindex'), 0) = 0)"))
}
pub fn article_indexable() -> Condition {
    Condition::all().add(Expr::cust("EXISTS (SELECT 1 FROM article_revisions r WHERE r.id = articles.published_revision_id AND COALESCE(json_extract(r.seo, '$.noindex'), 0) = 0)"))
}

/// Fetch taxonomy flags in two bounded batches, never one query per term.
pub async fn page_paths(now: i64) -> Result<Vec<String>, FrameworkError> {
    use crate::{
        articles,
        listings::{self, database_error, entities::category},
    };
    let db = DB::connection()?;
    let categories = listings::queries::public_categories(now).await?;
    let terms = articles::queries::terms(true).await?;
    let mut paths = vec!["/".into(), "/listings".into(), "/articles".into()];
    if !categories.is_empty() {
        let rows = category::Entity::find()
            .filter(
                category::Column::Id.is_in(categories.iter().map(|row| row.id).collect::<Vec<_>>()),
            )
            .all(db.inner())
            .await
            .map_err(database_error)?;
        for row in rows {
            if !super::Overrides::decode(&row.seo)?.noindex {
                paths.push(format!("/listings?category={}", row.slug));
            }
        }
    }
    if !terms.is_empty() {
        use articles::entities::term;
        let rows = term::Entity::find()
            .filter(term::Column::Id.is_in(terms.iter().map(|row| row.id).collect::<Vec<_>>()))
            .all(db.inner())
            .await
            .map_err(database_error)?;
        for row in rows {
            if !super::Overrides::decode(&row.seo)?.noindex {
                paths.push(format!("/articles?{}={}", row.kind, row.slug));
            }
        }
    }
    paths.sort();
    Ok(paths)
}
