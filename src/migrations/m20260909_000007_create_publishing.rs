use sea_orm_migration::prelude::*;

pub struct Migration;
impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260909_000007_create_publishing"
    }
}

fn text(name: &str) -> ColumnDef {
    let mut c = ColumnDef::new(Alias::new(name));
    c.text().not_null();
    c
}
fn integer(name: &str) -> ColumnDef {
    let mut c = ColumnDef::new(Alias::new(name));
    c.big_integer().not_null();
    c
}
fn boolean(name: &str) -> ColumnDef {
    let mut c = ColumnDef::new(Alias::new(name));
    c.boolean().not_null();
    c
}
fn nullable(mut c: ColumnDef) -> ColumnDef {
    c.null();
    c
}
fn primary(mut c: ColumnDef) -> ColumnDef {
    c.primary_key();
    c
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("publishing_plans"))
                    .col(primary(text("key")))
                    .col(text("name"))
                    .col(text("description"))
                    .col(boolean("enabled"))
                    .col(text("billing_type"))
                    .col(integer("amount"))
                    .col(text("currency"))
                    .col(integer("version"))
                    .col(integer("created_at"))
                    .col(integer("updated_at"))
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("publishing_purchases"))
                    .col(primary(text("id")))
                    .col(integer("listing_id"))
                    .col(integer("owner_id"))
                    .col(integer("approved_revision_id"))
                    .col(text("plan_key"))
                    .col(text("plan_name"))
                    .col(text("billing_type"))
                    .col(integer("amount"))
                    .col(text("currency"))
                    .col(text("provider"))
                    .col(text("mode"))
                    .col(nullable(text("price_id")))
                    .col(nullable(text("public_key")))
                    .col(nullable(text("credentials")))
                    .col(nullable(integer("profile_revision")))
                    .col(text("return_origin"))
                    .col(nullable(text("customer_ref")))
                    .col(nullable(text("session_ref")))
                    .col(nullable(text("subscription_ref")))
                    .col(nullable(text("checkout_payload")))
                    .col(text("state"))
                    .col(nullable(text("error_code")))
                    .col(integer("version"))
                    .col(nullable(integer("lease_until")))
                    .col(boolean("cancel_requested"))
                    .col(nullable(integer("cancel_at")))
                    .col(integer("cancel_event_at"))
                    .col(integer("created_at"))
                    .col(integer("updated_at"))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Alias::new("publishing_purchases"), Alias::new("listing_id"))
                            .to(Alias::new("listings"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Alias::new("publishing_purchases"), Alias::new("owner_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                Alias::new("publishing_purchases"),
                                Alias::new("approved_revision_id"),
                            )
                            .to(Alias::new("listing_revisions"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Alias::new("publishing_purchases"), Alias::new("plan_key"))
                            .to(Alias::new("publishing_plans"), Alias::new("key"))
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("publishing_slots"))
                    .col(primary(integer("listing_id")))
                    .col(text("purchase_id"))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Alias::new("publishing_slots"), Alias::new("listing_id"))
                            .to(Alias::new("listings"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Alias::new("publishing_slots"), Alias::new("purchase_id"))
                            .to(Alias::new("publishing_purchases"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("publishing_payments"))
                    .col(primary(text("id")))
                    .col(text("purchase_id"))
                    .col(text("provider_payment_id"))
                    .col(integer("amount_total"))
                    .col(integer("period_start"))
                    .col(nullable(integer("period_end")))
                    .col(text("status"))
                    .col(integer("paid_at"))
                    .col(integer("refund_event_at"))
                    .col(integer("dispute_event_at"))
                    .col(text("adverse"))
                    .col(integer("created_at"))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Alias::new("publishing_payments"), Alias::new("purchase_id"))
                            .to(Alias::new("publishing_purchases"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("publishing_references"))
                    .col(primary(text("id")))
                    .col(text("provider"))
                    .col(text("mode"))
                    .col(text("kind"))
                    .col(text("reference"))
                    .col(text("purchase_id"))
                    .col(nullable(text("payment_id")))
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                Alias::new("publishing_references"),
                                Alias::new("purchase_id"),
                            )
                            .to(Alias::new("publishing_purchases"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                Alias::new("publishing_references"),
                                Alias::new("payment_id"),
                            )
                            .to(Alias::new("publishing_payments"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("publishing_events"))
                    .col(primary(text("id")))
                    .col(text("provider"))
                    .col(text("mode"))
                    .col(text("provider_event_id"))
                    .col(text("event_type"))
                    .col(
                        ColumnDef::new(Alias::new("raw_body"))
                            .binary()
                            .not_null()
                            .to_owned(),
                    )
                    .col(text("signature"))
                    .col(integer("occurred_at"))
                    .col(integer("received_at"))
                    .col(nullable(text("purchase_id")))
                    .col(text("credential_source"))
                    .col(text("status"))
                    .col(integer("attempts"))
                    .col(integer("next_attempt_at"))
                    .col(nullable(text("error_code")))
                    .col(nullable(text("lease_token")))
                    .col(nullable(integer("lease_until")))
                    .col(nullable(text("enrichment")))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Alias::new("publishing_events"), Alias::new("purchase_id"))
                            .to(Alias::new("publishing_purchases"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("publishing_receipts"))
                    .col(primary(text("event_record_id")))
                    .col(nullable(text("purchase_id")))
                    .col(text("outcome"))
                    .col(integer("applied_at"))
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                Alias::new("publishing_receipts"),
                                Alias::new("event_record_id"),
                            )
                            .to(Alias::new("publishing_events"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Alias::new("publishing_receipts"), Alias::new("purchase_id"))
                            .to(Alias::new("publishing_purchases"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("publishing_purchase_owner")
                    .table(Alias::new("publishing_purchases"))
                    .col(Alias::new("owner_id"))
                    .col(Alias::new("created_at"))
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("publishing_purchase_listing")
                    .table(Alias::new("publishing_purchases"))
                    .col(Alias::new("listing_id"))
                    .col(Alias::new("created_at"))
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("publishing_slot_purchase")
                    .table(Alias::new("publishing_slots"))
                    .col(Alias::new("purchase_id"))
                    .unique()
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("publishing_payment_reference")
                    .table(Alias::new("publishing_payments"))
                    .col(Alias::new("purchase_id"))
                    .col(Alias::new("provider_payment_id"))
                    .unique()
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("publishing_reference_lookup")
                    .table(Alias::new("publishing_references"))
                    .col(Alias::new("provider"))
                    .col(Alias::new("mode"))
                    .col(Alias::new("kind"))
                    .col(Alias::new("reference"))
                    .unique()
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("publishing_event_identity")
                    .table(Alias::new("publishing_events"))
                    .col(Alias::new("provider"))
                    .col(Alias::new("mode"))
                    .col(Alias::new("provider_event_id"))
                    .unique()
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("publishing_event_pending")
                    .table(Alias::new("publishing_events"))
                    .col(Alias::new("status"))
                    .col(Alias::new("next_attempt_at"))
                    .col(Alias::new("received_at"))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(Alias::new("publishing_receipts"))
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(Alias::new("publishing_events"))
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(Alias::new("publishing_references"))
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(Alias::new("publishing_payments"))
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(Alias::new("publishing_slots"))
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(Alias::new("publishing_purchases"))
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(Alias::new("publishing_plans"))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
