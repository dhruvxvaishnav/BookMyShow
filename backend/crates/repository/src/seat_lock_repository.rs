use async_trait::async_trait;
use common::AppError;
use domain::{LockStatus, SeatLock};

#[async_trait]
pub trait SeatLockRepository: Send + Sync {
    async fn save(&self, lock: SeatLock) -> Result<SeatLock, AppError>;
    async fn find_by_id(&self, lock_id: &str) -> Result<Option<SeatLock>, AppError>;
    /// Find all active locks for a given show.
    async fn find_active_by_show(&self, show_id: &str) -> Result<Vec<SeatLock>, AppError>;
    /// Find all locks that have expired (status Active but past expires_at).
    async fn find_expired_locks(&self, grace_period_secs: i64) -> Result<Vec<SeatLock>, AppError>;
    /// Mark a lock's status.
    async fn update_status(&self, lock_id: &str, status: LockStatus) -> Result<SeatLock, AppError>;
    /// Increment the extension counter.
    async fn increment_extension(&self, lock_id: &str) -> Result<SeatLock, AppError>;
}
