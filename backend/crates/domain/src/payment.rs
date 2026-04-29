use super::PaymentStatus;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a payment associated with a booking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    pub payment_id: String,
    /// External reference ID (UUID generated before gateway call).
    pub payment_intent_id: String,
    pub booking_id: String,
    pub user_id: String,
    pub amount: f64,
    /// Currency code, default "INR".
    pub currency: String,
    pub status: PaymentStatus,
    /// Name of the payment gateway (e.g., "mock", "razorpay").
    pub gateway_name: String,
    /// Raw gateway response payload (JSON string).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateway_response: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    /// Set when gateway confirms success.
    #[serde(skip_serializing_if = "Option::is_none", with = "opt_ts_seconds")]
    pub confirmed_at: Option<DateTime<Utc>>,
    /// Set when payment fails.
    #[serde(skip_serializing_if = "Option::is_none", with = "opt_ts_seconds")]
    pub failed_at: Option<DateTime<Utc>>,
    /// Set when refund is issued.
    #[serde(skip_serializing_if = "Option::is_none", with = "opt_ts_seconds")]
    pub refunded_at: Option<DateTime<Utc>>,
}

mod opt_ts_seconds {
    use chrono::{DateTime, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(v: &Option<DateTime<Utc>>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match v {
            Some(dt) => s.serialize_i64(dt.timestamp()),
            None => s.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(d: D) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<i64> = Deserialize::deserialize(d)?;
        Ok(opt.map(|ts| DateTime::from_timestamp(ts, 0).unwrap()))
    }
}

impl Payment {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        payment_id: String,
        payment_intent_id: String,
        booking_id: String,
        user_id: String,
        amount: f64,
        gateway_name: String,
        idempotency_key: Option<String>,
    ) -> Self {
        Self {
            payment_id,
            payment_intent_id,
            booking_id,
            user_id,
            amount,
            currency: "INR".to_string(),
            status: PaymentStatus::Pending,
            gateway_name,
            gateway_response: None,
            idempotency_key,
            created_at: Utc::now(),
            confirmed_at: None,
            failed_at: None,
            refunded_at: None,
        }
    }
}
