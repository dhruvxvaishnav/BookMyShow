use async_trait::async_trait;
use common::AppError;
use domain::{QueueEntry, QueueStatus};
use repository::QueueRepository;
use std::collections::HashMap;
use tokio::sync::RwLock;

#[derive(Default)]
pub struct InMemoryQueueRepository {
    entries: RwLock<HashMap<String, QueueEntry>>,
}

impl InMemoryQueueRepository {
    pub fn new() -> Self {
        Self { entries: RwLock::new(HashMap::new()) }
    }
}

#[async_trait]
impl QueueRepository for InMemoryQueueRepository {
    async fn save(&self, entry: QueueEntry) -> Result<QueueEntry, AppError> {
        let mut w = self.entries.write().await;
        w.insert(entry.queue_id.clone(), entry.clone());
        Ok(entry)
    }

    async fn find_by_id(&self, queue_id: &str) -> Result<Option<QueueEntry>, AppError> {
        let r = self.entries.read().await;
        Ok(r.get(queue_id).cloned())
    }

    async fn find_waiting_by_show(&self, show_id: &str) -> Result<Vec<QueueEntry>, AppError> {
        let r = self.entries.read().await;
        let mut entries: Vec<_> = r
            .values()
            .filter(|e| e.show_id == show_id && e.status == QueueStatus::Waiting)
            .cloned()
            .collect();
        entries.sort_by_key(|e| e.position);
        Ok(entries)
    }

    async fn count_by_show_and_status(
        &self,
        show_id: &str,
        status: QueueStatus,
    ) -> Result<u32, AppError> {
        let r = self.entries.read().await;
        Ok(r.values().filter(|e| e.show_id == show_id && e.status == status).count() as u32)
    }

    async fn max_position(&self, show_id: &str) -> Result<u32, AppError> {
        let r = self.entries.read().await;
        Ok(r.values()
            .filter(|e| e.show_id == show_id)
            .map(|e| e.position)
            .max()
            .unwrap_or(0))
    }

    async fn mark_processed(&self, queue_id: &str, status: QueueStatus) -> Result<QueueEntry, AppError> {
        let mut w = self.entries.write().await;
        let entry = w
            .get_mut(queue_id)
            .ok_or_else(|| AppError::QueueEntryNotFound(queue_id.to_string()))?;
        entry.status = status;
        entry.processed_at = Some(chrono::Utc::now());
        Ok(entry.clone())
    }

    async fn delete(&self, queue_id: &str) -> Result<(), AppError> {
        let mut w = self.entries.write().await;
        w.remove(queue_id);
        Ok(())
    }

    async fn find_all_show_ids(&self) -> Result<Vec<String>, AppError> {
        let r = self.entries.read().await;
        let show_ids: std::collections::HashSet<_> = r
            .values()
            .filter(|e| matches!(e.status, QueueStatus::Waiting | QueueStatus::Processing))
            .map(|e| e.show_id.clone())
            .collect();
        Ok(show_ids.into_iter().collect())
    }
}
