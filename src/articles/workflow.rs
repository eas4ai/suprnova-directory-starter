use super::{
    EDIT_PERMISSION,
    entities::{article, media, revision, revision_term, slug, term},
    require_permission,
    validation::SaveArticle,
};
use crate::listings::{conflict, database_error, invalid, missing};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, ExprTrait, PaginatorTrait,
    QueryFilter, Set, TransactionTrait,
    sea_query::{Expr, OnConflict},
};
use suprnova::{DB, FrameworkError};

async fn lock(
    tx: &DatabaseTransaction,
    id: i64,
    version: i64,
) -> Result<article::Model, FrameworkError> {
    let changed = article::Entity::update_many()
        .col_expr(
            article::Column::Version,
            Expr::col(article::Column::Version).add(1),
        )
        .filter(article::Column::Id.eq(id))
        .filter(article::Column::Version.eq(version))
        .exec(tx)
        .await
        .map_err(database_error)?;
    let row = article::Entity::find_by_id(id)
        .one(tx)
        .await
        .map_err(database_error)?
        .ok_or_else(missing)?;
    if changed.rows_affected != 1 {
        return Err(conflict());
    }
    Ok(row)
}

async fn reserve_slug(tx: &DatabaseTransaction, id: i64, name: &str) -> Result<(), FrameworkError> {
    slug::Entity::insert(slug::ActiveModel {
        slug: Set(name.into()),
        article_id: Set(id),
    })
    .on_conflict(
        OnConflict::column(slug::Column::Slug)
            .do_nothing()
            .to_owned(),
    )
    .try_insert()
    .exec(tx)
    .await
    .map_err(database_error)?;
    let retained = slug::Entity::find_by_id(name)
        .one(tx)
        .await
        .map_err(database_error)?
        .ok_or_else(missing)?;
    if retained.article_id != id {
        return Err(invalid(
            "slug",
            "This URL belongs to another article. Choose another slug.",
        ));
    }
    Ok(())
}

/// A draft never changes the public pointer, URL, body or media.
pub async fn save(actor: i64, id: Option<i64>, input: SaveArticle) -> Result<i64, FrameworkError> {
    require_permission(actor, EDIT_PERMISSION).await?;
    let input = input.validate()?;
    let db = DB::connection()?;
    let tx = db.inner().begin().await.map_err(database_error)?;
    let now = chrono::Utc::now().timestamp();
    let row = if let Some(id) = id {
        lock(&tx, id, input.version).await?
    } else {
        if input.version != 0 {
            return Err(conflict());
        }
        article::ActiveModel {
            author_id: Set(actor),
            slug: Set(input.slug.clone()),
            version: Set(1),
            current_revision_id: Set(None),
            published_revision_id: Set(None),
            published_at: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(&tx)
        .await
        .map_err(database_error)?
    };
    // Draft URL edits do not become public redirects until publication.
    if id.is_none() {
        reserve_slug(&tx, row.id, &input.slug).await?;
    } else if slug::Entity::find_by_id(&input.slug)
        .one(&tx)
        .await
        .map_err(database_error)?
        .is_some_and(|alias| alias.article_id != row.id)
    {
        return Err(invalid(
            "slug",
            "This URL belongs to another article. Choose another slug.",
        ));
    }
    if !input.term_ids.is_empty() {
        let count = term::Entity::find()
            .filter(term::Column::Id.is_in(input.term_ids.clone()))
            .filter(term::Column::Active.eq(true))
            .count(&tx)
            .await
            .map_err(database_error)?;
        if count != input.term_ids.len() as u64 {
            return Err(invalid(
                "term_ids",
                "Choose existing active categories and tags.",
            ));
        }
    }
    if let Some(id) = &input.media_id {
        if media::Entity::find_by_id(id)
            .one(&tx)
            .await
            .map_err(database_error)?
            .is_none()
        {
            return Err(invalid(
                "media_id",
                "Upload an article image before attaching it.",
            ));
        }
    }
    let proposed = revision::ActiveModel {
        article_id: Set(row.id),
        slug: Set(input.slug),
        search_text: Set(format!("{}\n{}", input.title, input.summary).to_lowercase()),
        title: Set(input.title),
        summary: Set(input.summary),
        body: Set(input.body),
        media_id: Set(input.media_id),
        media_alt: Set(input.media_alt),
        created_at: Set(now),
        ..Default::default()
    }
    .insert(&tx)
    .await
    .map_err(database_error)?;
    for id in input.term_ids {
        revision_term::ActiveModel {
            revision_id: Set(proposed.id),
            term_id: Set(id),
        }
        .insert(&tx)
        .await
        .map_err(database_error)?;
    }
    article::Entity::update_many()
        .col_expr(article::Column::CurrentRevisionId, Expr::value(proposed.id))
        .col_expr(article::Column::UpdatedAt, Expr::value(now))
        .filter(article::Column::Id.eq(row.id))
        .exec(&tx)
        .await
        .map_err(database_error)?;
    crate::audit::record(
        &tx,
        actor,
        "article",
        row.id.to_string(),
        "draft_saved",
        "Saved title, summary, body, slug, media and taxonomy in a private revision.",
    )
    .await?;
    tx.commit().await.map_err(database_error)?;
    Ok(row.id)
}

pub async fn publish(
    actor: i64,
    id: i64,
    version: i64,
    publish: bool,
) -> Result<(), FrameworkError> {
    require_permission(actor, EDIT_PERMISSION).await?;
    let db = DB::connection()?;
    let tx = db.inner().begin().await.map_err(database_error)?;
    let row = lock(&tx, id, version).await?;
    let current = revision::Entity::find_by_id(row.current_revision_id.ok_or_else(missing)?)
        .filter(revision::Column::ArticleId.eq(id))
        .one(&tx)
        .await
        .map_err(database_error)?
        .ok_or_else(missing)?;
    if publish {
        reserve_slug(&tx, id, &current.slug).await?;
    }
    let now = chrono::Utc::now().timestamp();
    article::Entity::update_many()
        .col_expr(
            article::Column::Slug,
            Expr::value(if publish { current.slug } else { row.slug }),
        )
        .col_expr(
            article::Column::PublishedRevisionId,
            Expr::value(if publish { Some(current.id) } else { None }),
        )
        .col_expr(
            article::Column::PublishedAt,
            Expr::value(if publish { Some(now) } else { None }),
        )
        .col_expr(article::Column::UpdatedAt, Expr::value(now))
        .filter(article::Column::Id.eq(id))
        .exec(&tx)
        .await
        .map_err(database_error)?;
    crate::audit::record(
        &tx,
        actor,
        "article",
        id.to_string(),
        if publish { "published" } else { "unpublished" },
        if publish {
            "Published the current article revision and canonical slug."
        } else {
            "Removed the article from public surfaces."
        },
    )
    .await?;
    tx.commit().await.map_err(database_error)
}
