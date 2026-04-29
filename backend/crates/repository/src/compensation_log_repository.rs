use async_trait::async_trait;
use common::AppError;
use domain::CompensationLog;

#[async_trait]
pub trait CompensationLogRepository: Send + Sync {
    async fn save(&self, log: CompensationLog) -> Result<CompensationLog, AppError>;
    async fn find_by_booking(&self, booking_id: &str) -> Result<Vec<CompensationLog>, AppError>;
}
