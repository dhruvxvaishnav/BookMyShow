use serde::{Deserialize, Serialize};

/// Seat category that determines the price modifier applied to the show's base price.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SeatType {
    /// Standard row seat, price_modifier = 1.0
    #[default]
    Standard,
    /// Comfort seat, price_modifier = 1.5
    Comfort,
    /// Recliner seat, price_modifier = 2.0
    Recliner,
}

impl SeatType {
    /// Returns the price multiplier for this seat type.
    pub fn price_modifier(&self) -> f64 {
        match self {
            SeatType::Standard => 1.0,
            SeatType::Comfort => 1.5,
            SeatType::Recliner => 2.0,
        }
    }
}

impl std::fmt::Display for SeatType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SeatType::Standard => write!(f, "standard"),
            SeatType::Comfort => write!(f, "comfort"),
            SeatType::Recliner => write!(f, "recliner"),
        }
    }
}
