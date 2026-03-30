#[derive(Debug, Clone, PartialEq)]
pub enum BookingStatus {
    Pending,
    PaymentProcessing,
    Success,
    Failed
}