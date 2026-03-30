#[derive(Debug, Clone, PartialEq)]
pub enum SeatStatus {
    Available,
    Booked,
    Locked, // tutor has this extra status
}