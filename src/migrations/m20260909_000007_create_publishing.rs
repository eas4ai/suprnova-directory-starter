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
                    .col(&mut primary(text("key")))
                    .col(&mut text("name"))
                    .col(&mut text("description"))
                    .col(&mut boolean("enabled"))
                    .col(&mut text("billing_type"))
                    .col(&mut integer("amount"))
                    .col(&mut text("currency"))
                    .col(&mut integer("version"))
                    .col(&mut integer("created_at"))
                    .col(&mut integer("updated_at"))
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("publishing_purchases"))
                    .col(&mut primary(text("id")))
                    .col(&mut integer("listing_id"))
                    .col(&mut integer("owner_id"))
                    .col(&mut integer("approved_revision_id"))
                    .col(&mut text("plan_key"))
                    .col(&mut text("plan_name"))
                    .col(&mut text("billing_type"))
                    .col(&mut integer("amount"))
                    .col(&mut text("currency"))
                    .col(&mut text("provider"))
                    .col(&mut text("mode"))
                    .col(&mut nullable(text("price_id")))
                    .col(&mut nullable(text("public_key")))
                    .col(&mut nullable(text("credentials")))
                    .col(&mut nullable(integer("profile_revision")))
                    .col(&mut text("return_origin"))
                    .col(&mut nullable(text("customer_ref")))
                    .col(&mut nullable(text("session_ref")))
                    .col(&mut nullable(text("subscription_ref")))
                    .col(&mut nullable(text("checkout_payload")))
                    .col(&mut text("state"))
                    .col(&mut nullable(text("error_code")))
                    .col(&mut integer("version"))
                    .col(&mut nullable(integer("lease_until")))
                    .col(&mut boolean("cancel_requested"))
                    .col(&mut nullable(integer("cancel_at")))
                    .col(&mut integer("cancel_event_at"))
                    .col(&mut integer("created_at"))
                    .col(&mut integer("updated_at"))
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
                    .col(&mut primary(integer("listing_id")))
                    .col(&mut text("purchase_id"))
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
                    .col(&mut primary(text("id")))
                    .col(&mut text("purchase_id"))
                    .col(&mut text("provider_payment_id"))
                    .col(&mut integer("amount_total"))
                    .col(&mut integer("period_start"))
                    .col(&mut nullable(integer("period_end")))
                    .col(&mut text("status"))
                    .col(&mut integer("paid_at"))
                    .col(&mut integer("refund_event_at"))
                    .col(&mut integer("dispute_event_at"))
                    .col(&mut text("adverse"))
                    .col(&mut integer("created_at"))
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
                    .col(&mut primary(text("id")))
                    .col(&mut text("provider"))
                    .col(&mut text("mode"))
                    .col(&mut text("kind"))
                    .col(&mut text("reference"))
                    .col(&mut text("purchase_id"))
                    .col(&mut nullable(text("payment_id")))
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
                    .col(&mut primary(text("id")))
                    .col(&mut text("provider"))
                    .col(&mut text("mode"))
                    .col(&mut text("provider_event_id"))
                    .col(&mut text("event_type"))
                    .col(
                        &mut ColumnDef::new(Alias::new("raw_body"))
                            .binary()
                            .not_null()
                            .to_owned(),
                    )
                    .col(&mut text("signature"))
                    .col(&mut integer("occurred_at"))
                    .col(&mut integer("received_at"))
                    .col(&mut nullable(text("purchase_id")))
                    .col(&mut text("credential_source"))
                    .col(&mut text("status"))
                    .col(&mut integer("attempts"))
                    .col(&mut integer("next_attempt_at"))
                    .col(&mut nullable(text("error_code")))
                    .col(&mut nullable(text("lease_token")))
                    .col(&mut nullable(integer("lease_until")))
                    .col(&mut nullable(text("enrichment")))
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
                    .col(&mut primary(text("event_record_id")))
                    .col(&mut nullable(text("purchase_id")))
                    .col(&mut text("outcome"))
                    .col(&mut integer("applied_at"))
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
