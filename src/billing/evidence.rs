//! Validated facts derived from authenticated provider payloads and SDK reads.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EvidenceError(pub &'static str);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Settlement {
    pub payment_ref: String,
    pub customer_ref: String,
    pub session_ref: Option<String>,
    pub subscription_ref: Option<String>,
    pub amount_total: i64,
    pub period_start: i64,
    pub period_end: Option<i64>,
    pub paid_at: i64,
    pub references: Vec<(String, String)>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AdverseState {
    pub refund_total: i64,
    pub disputed: bool,
    pub lost_dispute: bool,
    pub observed_at: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Cancellation {
    pub effective_at: Option<i64>,
    pub scheduled_at: Option<i64>,
    pub observed_at: i64,
}
