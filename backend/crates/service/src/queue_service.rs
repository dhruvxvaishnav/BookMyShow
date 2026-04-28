use common::AppConfig;
use domain::{QueueEntry, QueueStatus};
use repository::{QueueRepository, SeatLockRepository, SeatRepository};
use std::sync::Arc;
use uuid::Uuid;

use super::queue::{QueueJoined, QueueStatusResult};
use super::seat_locking_service::SeatLockingService;

/// Per-show queue service for ordering concurrent seat requests fairly.
#[derive(Clone)]
pub struct QueueService {
    queue_repo: Arc<dyn QueueRepository>,
    seat_repo: Arc<dyn SeatRepository>,
    seat_lock_repo: Arc<dyn SeatLockRepository>,
    locking_svc: Arc<SeatLockingService>,
    cfg: AppConfig,
}

impl QueueService {
    pub fn new(
        queue_repo: Arc<dyn QueueRepository>,
        seat_repo: Arc<dyn SeatRepository>,
        seat_lock_repo: Arc<dyn SeatLockRepository>,
        locking_svc: Arc<SeatLockingService>,
        cfg: AppConfig,
    ) -> Self {
        Self {
            queue_repo,
            seat_repo,
            seat_lock_repo,
            locking_svc,
            cfg,
        }
    }

    /// Add a user to the per-show seat request queue.
    pub async fn join_queue(
        &self,
        show_id: &str,
        user_id: &str,
        requested_seat_ids: Vec<String>,
    ) -> Result<QueueJoined, common::AppError> {
        let max_position = self.queue_repo.max_position(show_id).await?;
        let queue_id = Uuid::new_v4().to_string();

        let entry = QueueEntry::new(
            queue_id.clone(),
            user_id.to_string(),
            show_id.to_string(),
            requested_seat_ids,
            max_position + 1,
        );

        self.queue_repo.save(entry).await?;

        tracing::info!(
            queue_id = %queue_id,
            show_id = %show_id,
            user_id = %user_id,
            position = max_position + 1,
            "user joined queue"
        );

        Ok(QueueJoined {
            queue_id,
            show_id: show_id.to_string(),
            position: max_position + 1,
            status: "waiting".to_string(),
        })
    }

    /// Poll the status of a queue entry.
    pub async fn get_queue_status(
        &self,
        queue_id: &str,
    ) -> Result<Option<QueueStatusResult>, common::AppError> {
        let entry = self.queue_repo.find_by_id(queue_id).await?;

        Ok(entry.map(|e| QueueStatusResult {
            queue_id: e.queue_id,
            status: e.status.to_string(),
            position: e.position,
            booking_id: e.booking_id,
            lock_id: e.lock_id,
            conflict_seats: e.conflict_seats,
        }))
    }

    /// Remove a user from the queue.
    pub async fn leave_queue(&self, queue_id: &str, user_id: &str) -> Result<(), common::AppError> {
        let entry = self
            .queue_repo
            .find_by_id(queue_id)
            .await?
            .ok_or_else(|| common::AppError::QueueEntryNotFound(queue_id.to_string()))?;

        if entry.user_id != user_id {
            return Err(common::AppError::LockNotOwnedByUser);
        }

        self.queue_repo.delete(queue_id).await?;
        Ok(())
    }

    /// Process the next waiting entry in a show's queue.
    /// Called by the background queue processor task.
    pub async fn process_next(&self, show_id: &str) -> Result<Option<QueueStatusResult>, common::AppError> {
        let waiting = self.queue_repo.find_waiting_by_show(show_id).await?;

        let entry = match waiting.into_iter().next() {
            Some(e) => e,
            None => return Ok(None),
        };

        // Mark as processing
        self.queue_repo
            .mark_processed(&entry.queue_id, QueueStatus::Processing)
            .await?;

        // Attempt lock
        let lock_result = self.locking_svc
            .lock_seats(&entry.show_id, entry.requested_seat_ids.clone(), &entry.user_id)
            .await;

        match lock_result {
            Ok(lock) => {
                let position = entry.position;
                self.queue_repo.save(domain::QueueEntry {
                    status: QueueStatus::Locked,
                    processed_at: Some(chrono::Utc::now()),
                    booking_id: Some(lock.booking_id.clone()),
                    lock_id: Some(lock.lock_id.clone()),
                    queue_id: entry.queue_id,
                    user_id: entry.user_id,
                    show_id: entry.show_id,
                    requested_seat_ids: entry.requested_seat_ids,
                    created_at: entry.created_at,
                    conflict_seats: None,
                    position: entry.position,
                }).await?;

                Ok(Some(QueueStatusResult {
                    queue_id: lock.lock_id.clone(),
                    status: "locked".to_string(),
                    position,
                    booking_id: Some(lock.booking_id),
                    lock_id: Some(lock.lock_id),
                    conflict_seats: None,
                }))
            }
            Err(e) => {
                // Lock failed — check if seats are unavailable
                let conflict_seats = match &e {
                    common::AppError::SeatsUnavailable(ids) => Some(ids.clone()),
                    _ => None,
                };
                let queue_id = entry.queue_id.clone();
                let user_id = entry.user_id.clone();
                let position = entry.position;

                self.queue_repo.save(domain::QueueEntry {
                    status: QueueStatus::Conflict,
                    processed_at: Some(chrono::Utc::now()),
                    conflict_seats: conflict_seats.clone(),
                    queue_id: entry.queue_id,
                    user_id: entry.user_id,
                    show_id: entry.show_id,
                    requested_seat_ids: entry.requested_seat_ids,
                    created_at: entry.created_at,
                    booking_id: None,
                    lock_id: None,
                    position: entry.position,
                }).await?;

                tracing::info!(
                    queue_id = %queue_id,
                    user_id = %user_id,
                    error = %e,
                    "queue entry conflict"
                );

                Ok(Some(QueueStatusResult {
                    queue_id,
                    status: "conflict".to_string(),
                    position,
                    booking_id: None,
                    lock_id: None,
                    conflict_seats,
                }))
            }
        }
    }
}
