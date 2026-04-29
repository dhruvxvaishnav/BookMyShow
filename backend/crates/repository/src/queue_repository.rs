use async_trait::async_trait;
use common::AppError;
use domain::{QueueEntry, QueueStatus};

#[async_trait]
pub trait QueueRepository: Send + Sync {
    async fn save(&self, entry: QueueEntry) -> Result<QueueEntry, AppError>;
    async fn find_by_id(&self, queue_id: &str) -> Result<Option<QueueEntry>, AppError>;
    /// Get all waiting entries for a show, ordered by position.
    async fn find_waiting_by_show(&self, show_id: &str) -> Result<Vec<QueueEntry>, AppError>;
    /// Count entries in a given status for a show.
    async fn count_by_show_and_status(
        &self,
        show_id: &str,
        status: QueueStatus,
    ) -> Result<u32, AppError>;
    /// Get the current max position for a show (used to assign next position).
    async fn max_position(&self, show_id: &str) -> Result<u32, AppError>;
    /// Update entry status and processed_at.
    async fn mark_processed(
        &self,
        queue_id: &str,
        status: QueueStatus,
    ) -> Result<QueueEntry, AppError>;
    /// Delete a queue entry.
    async fn delete(&self, queue_id: &str) -> Result<(), AppError>;
    /// Get all unique show_ids that have waiting/processing entries.
    async fn find_all_show_ids(&self) -> Result<Vec<String>, AppError>;
}
