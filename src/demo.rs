//! An isolated, insert-only fixture set. It never enters billing or notification workflows.
use crate::{articles::entities as articles, listings::entities as listings, models::user};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseTransaction, EntityTrait,
    IntoActiveModel, QueryFilter, Set, TransactionTrait,
};
use suprnova::{DB, FrameworkError};

pub const OWNER_EMAIL: &str = "directory-demo@example.invalid";
const CATEGORY_KEYS: [&str; 2] = ["demo-tools", "demo-learning"];
const LISTING_STATES: [&str; 4] = ["approved", "submitted", "rejected", "draft"];
const ARTICLE_KEYS: [&str; 2] = ["demo-welcome", "demo-editorial-draft"];
const ENTITLEMENT_KEY: &str = "demo:synthetic:approved:v1";

mod state {
    use sea_orm::entity::prelude::*;
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
    #[sea_orm(table_name = "demo_seed_state")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: i32,
        pub version: i64,
        pub completed: bool,
    }
    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}
    impl ActiveModelBehavior for ActiveModel {}
}

fn database_error(error: sea_orm::DbErr) -> FrameworkError {
    tracing::error!(error = %error, "Demo seed transaction failed");
    FrameworkError::database(
        "Demo seed failed. Check database diagnostics and reserved demo keys; rerunning preserves a completed seed.",
    )
}

/// Return true only when this call inserted the entire fixture set.
pub async fn seed(allow_production: bool) -> Result<bool, FrameworkError> {
    let environment = std::env::var("APP_ENV")
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    if !allow_production
        && !matches!(
            environment.as_str(),
            "local" | "development" | "test" | "testing"
        )
    {
        return Err(FrameworkError::database(
            "Demo seed refused outside local/development/test. Use --allow-production explicitly to add demonstration content to this installation.",
        ));
    }
    let db = DB::connection()?;
    let tx = db.inner().begin().await.map_err(database_error)?;
    // Write first: SQLite must not upgrade a read snapshot; PostgreSQL locks this
    // singleton until commit. A second seed observes the first seed's completion.
    let lock = tx
        .execute_unprepared("UPDATE demo_seed_state SET version = version + 1 WHERE id = 1")
        .await
        .map_err(database_error)?;
    if lock.rows_affected() != 1 {
        return Err(FrameworkError::database(
            "Demo seed state is missing. Run migrations before seeding.",
        ));
    }
    let marker = state::Entity::find_by_id(1)
        .one(&tx)
        .await
        .map_err(database_error)?
        .ok_or_else(|| FrameworkError::database("Demo seed state is missing."))?;
    if marker.completed {
        tx.rollback().await.map_err(database_error)?;
        return Ok(false);
    }
    ensure_keys_available(&tx).await?;
    let now = chrono::Utc::now();
    // Use the account model's generated ORM and framework password hash, inside
    // this transaction. No credential is printed or accepted as a fixed default.
    let owner = user::ActiveModel {
        name: Set("Demonstration owner (synthetic account)".into()),
        email: Set(OWNER_EMAIL.into()),
        password: Set(suprnova::hashing::hash(&uuid::Uuid::new_v4().to_string())?),
        remember_token: Set(None),
        email_verified_at: Set(None),
        created_at: Set(now.to_rfc3339()),
        updated_at: Set(now.to_rfc3339()),
        ..Default::default()
    }
    .insert(&tx)
    .await
    .map_err(database_error)?;
    let mut category_ids = Vec::new();
    for (slug, name) in CATEGORY_KEYS
        .into_iter()
        .zip(["Demo tools", "Demo learning"])
    {
        let row = listings::category::ActiveModel {
            slug: Set(slug.into()),
            name: Set(name.into()),
            active: Set(true),
            version: Set(0),
            ..Default::default()
        }
        .insert(&tx)
        .await
        .map_err(database_error)?;
        category_ids.push(row.id);
    }
    for (index, status) in LISTING_STATES.into_iter().enumerate() {
        insert_listing(
            &tx,
            owner.id,
            category_ids[index % category_ids.len()],
            status,
            now.timestamp(),
        )
        .await?;
    }
    for (index, slug) in ARTICLE_KEYS.into_iter().enumerate() {
        insert_article(&tx, owner.id, slug, index == 0, now.timestamp()).await?;
    }
    let mut marker = marker.into_active_model();
    marker.completed = Set(true);
    marker.update(&tx).await.map_err(database_error)?;
    tx.commit().await.map_err(database_error)?;
    Ok(true)
}

async fn ensure_keys_available(tx: &DatabaseTransaction) -> Result<(), FrameworkError> {
    let collision = user::Entity::find()
        .filter(user::Column::Email.eq(OWNER_EMAIL))
        .one(tx)
        .await
        .map_err(database_error)?
        .is_some()
        || listings::category::Entity::find()
            .filter(listings::category::Column::Slug.is_in(CATEGORY_KEYS))
            .one(tx)
            .await
            .map_err(database_error)?
            .is_some()
        || listings::listing::Entity::find()
            .filter(
                listings::listing::Column::Slug.is_in(LISTING_STATES.map(|s| format!("demo-{s}"))),
            )
            .one(tx)
            .await
            .map_err(database_error)?
            .is_some()
        || articles::article::Entity::find()
            .filter(articles::article::Column::Slug.is_in(ARTICLE_KEYS))
            .one(tx)
            .await
            .map_err(database_error)?
            .is_some()
        || articles::slug::Entity::find()
            .filter(articles::slug::Column::Slug.is_in(ARTICLE_KEYS))
            .one(tx)
            .await
            .map_err(database_error)?
            .is_some()
        || listings::entitlement::Entity::find_by_id(ENTITLEMENT_KEY)
            .one(tx)
            .await
            .map_err(database_error)?
            .is_some();
    if collision {
        return Err(FrameworkError::database(
            "Demo seed refused: a reserved demo email, slug or entitlement key already exists. Existing records were preserved; use a fresh database for demonstrations.",
        ));
    }
    Ok(())
}

async fn insert_listing(
    tx: &DatabaseTransaction,
    owner_id: i64,
    category_id: i64,
    status: &str,
    now: i64,
) -> Result<(), FrameworkError> {
    let title = format!("Demo {status} resource");
    let row = listings::listing::ActiveModel {
        owner_id: Set(owner_id),
        slug: Set(format!("demo-{status}")),
        version: Set(1),
        current_revision_id: Set(None),
        approved_revision_id: Set(None),
        archived: Set(false),
        suspended: Set(false),
        created_at: Set(now),
        ..Default::default()
    }
    .insert(tx)
    .await
    .map_err(database_error)?;
    let revision = listings::revision::ActiveModel {
        listing_id: Set(row.id), title: Set(title.clone()), summary: Set("Demonstration content for exploring the directory.".into()),
        search_text: Set(title.to_lowercase()), description: Set("This is synthetic demonstration content. The publication entitlement is a demo fixture, not a purchase.".into()),
        url: Set("https://example.invalid/demo".into()), media_id: Set(None), media_alt: Set(String::new()),
        status: Set(status.into()), reason: Set((status == "rejected").then(|| "Demo rejection: add a more detailed description.".into())),
        decided_by: Set(None), decided_at: Set(matches!(status, "approved" | "rejected").then_some(now)), created_at: Set(now),
        ..Default::default()
    }.insert(tx).await.map_err(database_error)?;
    listings::revision_category::ActiveModel {
        revision_id: Set(revision.id),
        category_id: Set(category_id),
    }
    .insert(tx)
    .await
    .map_err(database_error)?;
    let listing_id = row.id;
    let mut row = row.into_active_model();
    row.current_revision_id = Set(Some(revision.id));
    row.approved_revision_id = Set((status == "approved").then_some(revision.id));
    row.update(tx).await.map_err(database_error)?;
    if status == "approved" {
        listings::entitlement::ActiveModel {
            id: Set(ENTITLEMENT_KEY.into()),
            listing_id: Set(listing_id),
            mode: Set("free".into()),
            status: Set("active".into()),
            valid_from: Set(now),
            valid_until: Set(None),
            updated_at: Set(now),
        }
        .insert(tx)
        .await
        .map_err(database_error)?;
    }
    Ok(())
}

async fn insert_article(
    tx: &DatabaseTransaction,
    author_id: i64,
    slug: &str,
    published: bool,
    now: i64,
) -> Result<(), FrameworkError> {
    let title = if published {
        "Demo: welcome to your directory"
    } else {
        "Demo: an unfinished editorial draft"
    };
    let row = articles::article::ActiveModel {
        author_id: Set(author_id),
        slug: Set(slug.into()),
        version: Set(1),
        current_revision_id: Set(None),
        published_revision_id: Set(None),
        published_at: Set(published.then_some(now)),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(tx)
    .await
    .map_err(database_error)?;
    let revision = articles::revision::ActiveModel {
        article_id: Set(row.id), slug: Set(slug.into()), title: Set(title.into()), summary: Set("Synthetic demonstration article.".into()),
        search_text: Set(title.to_lowercase()), body: Set("This demonstration article shows the editorial workflow. Replace it with your own content when exploring a disposable installation.".into()),
        media_id: Set(None), media_alt: Set(String::new()), created_at: Set(now), ..Default::default()
    }.insert(tx).await.map_err(database_error)?;
    articles::slug::ActiveModel {
        slug: Set(slug.into()),
        article_id: Set(row.id),
    }
    .insert(tx)
    .await
    .map_err(database_error)?;
    let mut row = row.into_active_model();
    row.current_revision_id = Set(Some(revision.id));
    row.published_revision_id = Set(published.then_some(revision.id));
    row.update(tx).await.map_err(database_error)?;
    Ok(())
}
