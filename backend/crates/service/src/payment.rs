/// DTO returned after payment initiation.
#[derive(Debug, Clone)]
pub struct PaymentInitiated {
    pub payment_id: String,
    pub payment_intent_id: String,
    pub amount: f64,
    pub gateway_name: String,
    pub client_secret: Option<String>,
}

/// DTO for mock gateway payment request.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MockPaymentRequest {
    pub payment_intent_id: String,
    pub amount: f64,
    #[serde(default)]
    pub simulate_failure: bool,
    #[serde(default)]
    pub simulate_delay_ms: u64,
}

/// DTO for mock gateway response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MockPaymentResponse {
    pub status: String, // "SUCCESS" or "FAILED"
    pub gateway_reference: String,
}
