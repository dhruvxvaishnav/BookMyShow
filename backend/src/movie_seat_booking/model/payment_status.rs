#[derive(Debug, Clone, PartialEq)]
pub enum PaymentStatus {
    Pending,
    Success,
    Failed,
}

impl PaymentStatus {
    pub fn from_str(status: &str) -> Self {
        match status {
            "SUCCESS" => PaymentStatus::Success,
            "FAILED" => PaymentStatus::Failed,
            _ => PaymentStatus::Pending,
        }
    }
}