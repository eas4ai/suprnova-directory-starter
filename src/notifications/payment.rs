use crate::billing::fulfillment::Effect;
use sea_orm::DatabaseTransaction;
use suprnova::FrameworkError;

pub(crate) async fn effects(
    tx: &DatabaseTransaction,
    id: &str,
    event_id: &str,
    effects: &[Effect],
) -> Result<(), FrameworkError> {
    let mut updates = Vec::new();
    for effect in effects {
        match effect {
                Effect::Settled => updates.push("A payment was confirmed."),
                Effect::Adverse(state) => {
                    if state.refund_total > 0 { updates.push("A refund was recorded. A full refund removes the affected payment's publishing eligibility."); }
                    if state.lost_dispute { updates.push("A lost dispute removed the affected payment's publishing eligibility."); }
                    else if state.disputed { updates.push("An open dispute suspended the affected payment's publishing eligibility."); }
                    else if state.refund_total == 0 { updates.push("The provider confirmed the current payment and dispute state."); }
                }
                Effect::Cancellation => updates.push("Subscription cancellation status was confirmed. Check the purchase page for its effective date."),
                Effect::CheckoutEnded => updates.push("The provider confirmed that the checkout ended."),
            }
    }
    if !updates.is_empty() {
        super::payment_event(
            tx,
            id,
            &format!("payment-event:{event_id}"),
            &updates.join("\n"),
        )
        .await?;
    }
    Ok(())
}
