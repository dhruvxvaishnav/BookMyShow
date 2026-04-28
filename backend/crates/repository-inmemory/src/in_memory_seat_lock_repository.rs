use async_trait::async_trait;
use chrono::Utc;
use common::AppError;
use domain::{LockStatus, SeatLock};
use repository::SeatLockRepository;
use std::collections::HashMap;
use tokio::sync::RwLock;

#[derive(Default)]
pub struct InMemorySeatLockRepository {
    locks: RwLock<HashMap<String, SeatLock>>,
}

impl InMemorySeatLockRepository {
    pub fn new() -> Self {
        Self { locks: RwLock::new(HashMap::new()) }
    }
}

#[async_trait]
impl SeatLockRepository for InMemorySeatLockRepository {
    async fn save(&self, lock: SeatLock) -> Result<SeatLock, AppError> {
        let mut w = self.locks.write().await;
        w.insert(lock.lock_id.clone(), lock.clone());
        Ok(lock)
    }

    async fn find_by_id(&self, lock_id: &str) -> Result<Option<SeatLock>, AppError> {
        let r = self.locks.read().await;
        Ok(r.get(lock_id).cloned())
    }

    async fn find_active_by_show(&self, show_id: &str) -> Result<Vec<SeatLock>, AppError> {
        let r = self.locks.read().await;
        Ok(r.values()
            .filter(|l| l.show_id == show_id && l.status == LockStatus::Active)
            .cloned()
            .collect())
    }

    async fn find_expired_locks(&self, grace_period_secs: i64) -> Result<Vec<SeatLock>, AppError> {
        let r = self.locks.read().await;
        let now = Utc::now();
        let cutoff = now - chrono::Duration::seconds(grace_period_secs);

        Ok(r.values()
            .filter(|l| l.status == LockStatus::Active && l.expires_at < cutoff)
            .cloned()
            .collect())
    }

    async fn update_status(&self, lock_id: &str, status: LockStatus) -> Result<SeatLock, AppError> {
        let mut w = self.locks.write().await;
        let lock = w
            .get_mut(lock_id)
            .ok_or_else(|| AppError::LockNotFound(lock_id.to_string()))?;
        lock.status = status;
        Ok(lock.clone())
    }

    async fn increment_extension(&self, lock_id: &str) -> Result<SeatLock, AppError> {
        let mut w = self.locks.write().await;
        let lock = w
            .get_mut(lock_id)
            .ok_or_else(|| AppError::LockNotFound(lock_id.to_string()))?;
        lock.extended_count += 1;
        Ok(lock.clone())
    }
}
