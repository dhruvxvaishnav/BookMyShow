use async_trait::async_trait;
use common::AppError;
use domain::CompensationLog;
use repository::CompensationLogRepository;
use std::collections::HashMap;
use tokio::sync::RwLock;

#[derive(Default)]
pub struct InMemoryCompensationLogRepository {
    logs: RwLock<HashMap<String, CompensationLog>>,
}

impl InMemoryCompensationLogRepository {
    pub fn new() -> Self {
        Self {
            logs: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl CompensationLogRepository for InMemoryCompensationLogRepository {
    async fn save(&self, log: CompensationLog) -> Result<CompensationLog, AppError> {
        let mut w = self.logs.write().await;
        w.insert(log.compensation_id.clone(), log.clone());
        Ok(log)
    }

    async fn find_by_booking(&self, booking_id: &str) -> Result<Vec<CompensationLog>, AppError> {
        let r = self.logs.read().await;
        Ok(r.values()
            .filter(|l| l.booking_id == booking_id)
            .cloned()
            .collect())
    }

    async fn find_by_user(&self, user_id: &str) -> Result<Vec<CompensationLog>, AppError> {
        let r = self.logs.read().await;
        Ok(r.values()
            .filter(|l| l.user_id == user_id)
            .cloned()
            .collect())
    }

    async fn find_all(&self) -> Result<Vec<CompensationLog>, AppError> {
        let r = self.logs.read().await;
        Ok(r.values().cloned().collect())
    }
}
