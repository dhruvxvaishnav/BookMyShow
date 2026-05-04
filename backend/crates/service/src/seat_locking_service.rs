use chrono::{Duration, Utc};
use common::{AppConfig, AppError};
use domain::{Booking, BookingStatus, CompensationLog, LockStatus, SeatLock, SeatStatus};
use repository::{
    BookingRepository, CompensationLogRepository, SeatLockRepository, SeatRepository,
    ShowRepository, UserRepository,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{OwnedRwLockWriteGuard, RwLock};
use uuid::Uuid;

use super::seat_locking::{LockResult, OverrideResult};

/// Core seat locking service. Implements per-show Mutex and double-checked locking
/// to guarantee that exactly one user can lock a given seat at a time.
#[derive(Clone)]
pub struct SeatLockingService {
    show_repo: Arc<dyn ShowRepository>,
    seat_repo: Arc<dyn SeatRepository>,
    booking_repo: Arc<dyn BookingRepository>,
    seat_lock_repo: Arc<dyn SeatLockRepository>,
    user_repo: Arc<dyn UserRepository>,
    compensation_log_repo: Option<Arc<dyn CompensationLogRepository>>,
    /// Per-show Mutex — only one lock acquisition can proceed per show at a time.
    show_locks: Arc<RwLock<HashMap<String, Arc<RwLock<()>>>>>,
    cfg: AppConfig,
}

impl SeatLockingService {
    pub fn new(
        show_repo: Arc<dyn ShowRepository>,
        seat_repo: Arc<dyn SeatRepository>,
        booking_repo: Arc<dyn BookingRepository>,
        seat_lock_repo: Arc<dyn SeatLockRepository>,
        user_repo: Arc<dyn UserRepository>,
        cfg: AppConfig,
    ) -> Self {
        Self {
            show_repo,
            seat_repo,
            booking_repo,
            seat_lock_repo,
            user_repo,
            compensation_log_repo: None,
            show_locks: Arc::new(RwLock::new(HashMap::new())),
            cfg,
        }
    }

    pub fn with_audit_log_repo(mut self, repo: Arc<dyn CompensationLogRepository>) -> Self {
        self.compensation_log_repo = Some(repo);
        self
    }

    /// Acquire a lock on one or more seats for a user.
    ///
    /// Algorithm:
    /// 1. Validate show exists and user exists.
    /// 2. Validate seat count (1–MAX_SEATS).
    /// 3. Validate all seats belong to the same show.
    /// 4. Acquire per-show Mutex.
    /// 5. Double-check: verify all seats are Available (not Locked, not Booked).
    /// 6. Atomically lock all seats and create SeatLock + Booking records.
    /// 7. Release per-show Mutex.
    /// 8. Return LockResult.
    pub async fn lock_seats(
        &self,
        show_id: &str,
        seat_ids: Vec<String>,
        user_id: &str,
    ) -> Result<LockResult, AppError> {
        // ── Pre-validation ────────────────────────────────────────────────────

        self.show_repo
            .find_by_id(show_id)
            .await?
            .ok_or_else(|| AppError::ShowNotFound(show_id.to_string()))?;

        self.user_repo
            .exists(user_id)
            .await?
            .then_some(())
            .ok_or_else(|| AppError::UserNotFound(user_id.to_string()))?;

        // Seat count validation
        let max_seats = 10;
        if seat_ids.is_empty() {
            return Err(AppError::NoSeatsSelected);
        }
        if seat_ids.len() > max_seats {
            return Err(AppError::TooManySeats(max_seats, seat_ids.len()));
        }

        // Fetch and validate all seats
        let seats = self.seat_repo.find_by_ids(&seat_ids).await?;
        if seats.len() != seat_ids.len() {
            let found_ids: std::collections::HashSet<_> =
                seats.iter().map(|s| &s.seat_id).collect();
            let missing: Vec<_> = seat_ids
                .iter()
                .filter(|id| !found_ids.contains(id))
                .collect();
            return Err(AppError::SeatNotFound(
                (*missing.first().ok_or_else(|| {
                    AppError::InternalError("seat lookup returned inconsistent results".to_string())
                })?)
                .clone(),
            ));
        }

        // All seats must belong to the same show
        if !seats.iter().all(|s| s.show_id == show_id) {
            return Err(AppError::SeatsMustBelongToSameShow);
        }

        // ── Acquire per-show Mutex ────────────────────────────────────────────
        let _guard = self.acquire_show_lock(show_id).await;

        // ── Double-check: re-read seats inside critical section ──────────────
        let current_seats = self.seat_repo.find_by_ids(&seat_ids).await?;

        let mut unavailable: Vec<String> = Vec::new();
        let mut already_owned: Vec<String> = Vec::new();

        for seat in &current_seats {
            match seat.status {
                SeatStatus::Available => {}
                SeatStatus::Locked => {
                    if seat.locked_by.as_ref().map(|u| &u.user_id) == Some(&user_id.to_string()) {
                        already_owned.push(seat.seat_id.clone());
                    } else {
                        unavailable.push(seat.seat_id.clone());
                    }
                }
                SeatStatus::Booked => {
                    unavailable.push(seat.seat_id.clone());
                }
            }
        }

        if !already_owned.is_empty() {
            return Err(AppError::SeatsAlreadyLockedByUser);
        }
        if !unavailable.is_empty() {
            return Err(AppError::SeatsUnavailable(unavailable));
        }

        // ── All clear — acquire locks ─────────────────────────────────────────
        let now = Utc::now();
        let ttl = Duration::seconds(self.cfg.seat_lock.ttl_seconds as i64);
        let expires_at = now + ttl;

        let lock_id = Uuid::new_v4().to_string();
        let booking_id = Uuid::new_v4().to_string();

        // Lock all seats. Roll back already-locked seats if any repository call fails.
        let mut locked_seat_ids = Vec::with_capacity(current_seats.len());
        for seat in &current_seats {
            if let Err(e) = self
                .seat_repo
                .lock_seat(&seat.seat_id, user_id, &lock_id, expires_at)
                .await
            {
                self.rollback_locked_seats(&locked_seat_ids).await;
                return Err(e);
            }
            locked_seat_ids.push(seat.seat_id.clone());
        }

        // Create SeatLock record
        let seat_lock = SeatLock::new(
            lock_id.clone(),
            user_id.to_string(),
            show_id.to_string(),
            seat_ids.clone(),
            expires_at,
        );
        if let Err(e) = self.seat_lock_repo.save(seat_lock).await {
            self.rollback_locked_seats(&locked_seat_ids).await;
            return Err(e);
        }

        // Calculate total amount
        let show = self
            .show_repo
            .find_by_id(show_id)
            .await?
            .ok_or_else(|| AppError::ShowNotFound(show_id.to_string()))?;
        let total_amount: f64 = current_seats
            .iter()
            .map(|s| s.effective_price(show.price_per_seat))
            .sum();

        // Create Booking record
        let booking = Booking::new(
            booking_id.clone(),
            user_id.to_string(),
            show_id.to_string(),
            seat_ids.clone(),
            total_amount,
            lock_id.clone(),
            expires_at,
        );
        if let Err(e) = self.booking_repo.save(booking).await {
            self.rollback_locked_seats(&locked_seat_ids).await;
            let _ = self
                .seat_lock_repo
                .update_status(&lock_id, LockStatus::Released)
                .await;
            self.save_audit_event(
                &booking_id,
                show_id,
                user_id,
                "lock_rollback",
                Some(user_id.to_string()),
                Some("Pending".to_string()),
                Some("Cancelled".to_string()),
                Some("rolled back seat lock after booking insert failure".to_string()),
                Some(serde_json::json!({ "lock_id": lock_id, "seat_ids": seat_ids })),
            )
            .await;
            return Err(e);
        }

        self.save_audit_event(
            &booking_id,
            show_id,
            user_id,
            "lock",
            Some(user_id.to_string()),
            None,
            Some("Pending".to_string()),
            Some("seat lock acquired and booking created".to_string()),
            Some(serde_json::json!({ "lock_id": lock_id.clone(), "seat_ids": seat_ids.clone() })),
        )
        .await;

        tracing::info!(
            booking_id = %booking_id,
            lock_id = %lock_id,
            user_id = %user_id,
            show_id = %show_id,
            seat_count = seat_ids.len(),
            "seat lock acquired"
        );

        Ok(LockResult {
            lock_id,
            booking_id,
            show_id: show_id.to_string(),
            seat_ids,
            total_amount,
            expires_at,
            status: "pending".to_string(),
        })
    }

    /// Extend the TTL of an active lock.
    pub async fn extend_lock(
        &self,
        booking_id: &str,
        user_id: &str,
    ) -> Result<LockResult, AppError> {
        let booking = self
            .booking_repo
            .find_by_id(booking_id)
            .await?
            .ok_or_else(|| AppError::BookingNotFound(booking_id.to_string()))?;

        if booking.user_id != user_id {
            return Err(AppError::LockNotOwnedByUser);
        }

        if !booking.is_lockable() {
            return Err(AppError::BookingAlreadyProcessed(booking_id.to_string()));
        }

        let lock_id = booking
            .lock_id
            .as_ref()
            .ok_or_else(|| AppError::LockNotFound(booking_id.to_string()))?
            .clone();

        let lock = self
            .seat_lock_repo
            .find_by_id(&lock_id)
            .await?
            .ok_or_else(|| AppError::LockNotFound(lock_id.to_string()))?;

        if !lock.can_extend(self.cfg.seat_lock.max_extensions) {
            return Err(AppError::LockMaxExtensionsReached);
        }

        if !lock.is_active(self.cfg.seat_lock.grace_period_seconds as i64) {
            return Err(AppError::LockExpired(lock_id.clone()));
        }

        // Extend expiry
        let ext_duration = Duration::seconds(self.cfg.seat_lock.extension_seconds as i64);
        let new_expires_at = lock.expires_at + ext_duration;

        // Update SeatLock
        self.seat_lock_repo.increment_extension(&lock_id).await?;
        let updated_lock = self
            .seat_lock_repo
            .find_by_id(&lock_id)
            .await?
            .ok_or_else(|| AppError::LockNotFound(lock_id.clone()))?;
        let mut updated_lock = updated_lock;
        updated_lock.expires_at = new_expires_at;
        self.seat_lock_repo.save(updated_lock.clone()).await?;

        // Update all locked seats' expiry timestamps
        let seat_ids_clone = booking.seat_ids.clone();
        let show_id_clone = booking.show_id.clone();
        for seat_id in &seat_ids_clone {
            let seat = self
                .seat_repo
                .find_by_id(seat_id)
                .await?
                .ok_or_else(|| AppError::SeatNotFound(seat_id.clone()))?;
            let mut updated_seat = seat;
            updated_seat.lock_expires_at = Some(new_expires_at);
            self.seat_repo.save(updated_seat).await?;
        }

        // Update booking expiry
        let mut updated_booking = booking;
        updated_booking.expires_at = new_expires_at;
        self.booking_repo.save(updated_booking.clone()).await?;

        self.save_audit_event(
            booking_id,
            &show_id_clone,
            user_id,
            "lock_extended",
            Some(user_id.to_string()),
            Some("Pending".to_string()),
            Some("Pending".to_string()),
            Some("seat lock expiry extended".to_string()),
            Some(serde_json::json!({
                "lock_id": lock_id.clone(),
                "new_expires_at": new_expires_at.timestamp(),
                "extension_count": updated_lock.extended_count
            })),
        )
        .await;

        tracing::info!(
            booking_id = %booking_id,
            lock_id = %lock_id,
            new_expires_at = %new_expires_at,
            extension_count = updated_lock.extended_count,
            "lock extended"
        );

        let show = self
            .show_repo
            .find_by_id(&show_id_clone)
            .await?
            .ok_or_else(|| AppError::ShowNotFound(show_id_clone.clone()))?;
        let seats = self.seat_repo.find_by_ids(&seat_ids_clone).await?;
        let total_amount: f64 = seats
            .iter()
            .map(|s| s.effective_price(show.price_per_seat))
            .sum();

        Ok(LockResult {
            lock_id: lock_id.clone(),
            booking_id: booking_id.to_string(),
            show_id: show_id_clone,
            seat_ids: seat_ids_clone,
            total_amount,
            expires_at: new_expires_at,
            status: "pending".to_string(),
        })
    }

    /// Release a lock (user cancellation).
    pub async fn release_lock(&self, booking_id: &str, user_id: &str) -> Result<(), AppError> {
        let booking = self
            .booking_repo
            .find_by_id(booking_id)
            .await?
            .ok_or_else(|| AppError::BookingNotFound(booking_id.to_string()))?;

        if booking.user_id != user_id {
            return Err(AppError::LockNotOwnedByUser);
        }

        if !booking.is_lockable() {
            return Err(AppError::BookingAlreadyProcessed(booking_id.to_string()));
        }

        // Release all seats
        for seat_id in &booking.seat_ids {
            self.seat_repo.release_seat(seat_id).await?;
        }

        // Mark lock as Released
        if let Some(ref lock_id) = booking.lock_id {
            self.seat_lock_repo
                .update_status(lock_id, LockStatus::Released)
                .await?;
        }

        // Mark booking as Cancelled
        let status_from = booking.status.to_string();
        let show_id = booking.show_id.clone();
        let seat_ids = booking.seat_ids.clone();
        let mut updated_booking = booking;
        updated_booking.status = BookingStatus::Cancelled;
        updated_booking.cancelled_at = Some(Utc::now());
        self.booking_repo.save(updated_booking).await?;

        self.save_audit_event(
            booking_id,
            &show_id,
            user_id,
            "cancel",
            Some(user_id.to_string()),
            Some(status_from),
            Some(BookingStatus::Cancelled.to_string()),
            Some("lock released by user".to_string()),
            Some(serde_json::json!({ "seat_ids": seat_ids })),
        )
        .await;

        tracing::info!(
            booking_id = %booking_id,
            user_id = %user_id,
            "lock released (user cancelled)"
        );

        Ok(())
    }

    /// Process expired locks — called by the background expiration task.
    pub async fn process_expired_locks(&self) -> Result<usize, AppError> {
        let expired_locks = self
            .seat_lock_repo
            .find_expired_locks(self.cfg.seat_lock.grace_period_seconds as i64)
            .await?;

        let mut processed = 0;
        for lock in expired_locks {
            self.expire_lock(&lock.lock_id).await?;
            processed += 1;
        }

        // Also handle expired bookings
        let expired_bookings = self
            .booking_repo
            .find_expired(self.cfg.seat_lock.grace_period_seconds as i64)
            .await?;

        for booking in expired_bookings {
            self.booking_repo
                .save(Booking {
                    status: BookingStatus::Expired,
                    cancelled_at: Some(Utc::now()),
                    ..booking.clone()
                })
                .await?;
            self.save_audit_event(
                &booking.booking_id,
                &booking.show_id,
                &booking.user_id,
                "expire",
                None,
                Some(booking.status.to_string()),
                Some(BookingStatus::Expired.to_string()),
                Some("booking expired after lock grace period".to_string()),
                Some(serde_json::json!({ "seat_ids": booking.seat_ids })),
            )
            .await;
            processed += 1;
        }

        if processed > 0 {
            tracing::info!(count = processed, "expired locks processed");
        }

        Ok(processed)
    }

    /// Expire a single lock and release its seats.
    async fn expire_lock(&self, lock_id: &str) -> Result<(), AppError> {
        let lock = self
            .seat_lock_repo
            .find_by_id(lock_id)
            .await?
            .ok_or_else(|| AppError::LockNotFound(lock_id.to_string()))?;

        // Release all seats held by this lock
        for seat_id in &lock.seat_ids {
            if let Ok(Some(seat)) = self.seat_repo.find_by_id(seat_id).await
                && seat.lock_id.as_ref() == Some(&lock_id.to_string())
            {
                self.seat_repo.release_seat(seat_id).await?;
            }
        }

        // Mark lock as Expired
        self.seat_lock_repo
            .update_status(lock_id, LockStatus::Expired)
            .await?;

        // Mark associated booking as Expired
        let bookings = self
            .booking_repo
            .find_by_status(BookingStatus::Pending)
            .await?;
        for mut booking in bookings {
            if booking.lock_id.as_ref() == Some(&lock_id.to_string()) {
                booking.status = BookingStatus::Expired;
                booking.cancelled_at = Some(Utc::now());
                self.booking_repo.save(booking).await?;
            }
        }

        tracing::warn!(lock_id = %lock_id, "lock expired and seats released");
        Ok(())
    }

    /// Force-release a locked seat (admin operation). Works regardless of who holds the lock.
    /// Returns an error if the seat is not in a Locked state.
    pub async fn admin_override_seat(
        &self,
        seat_id: &str,
        reason: &str,
    ) -> Result<OverrideResult, AppError> {
        let seat = self
            .seat_repo
            .find_by_id(seat_id)
            .await?
            .ok_or_else(|| AppError::SeatNotFound(seat_id.to_string()))?;

        if !matches!(seat.status, SeatStatus::Locked) {
            return Err(AppError::ValidationError(format!(
                "seat {} is not locked (current status: {})",
                seat_id, seat.status
            )));
        }

        let released_lock_id = seat.lock_id.clone();
        let seat_number = seat.seat_number.clone();

        // Acquire per-show mutex so no concurrent lock attempt races us
        let _guard = self.acquire_show_lock(&seat.show_id).await;

        self.seat_repo.release_seat(seat_id).await?;

        // Mark the associated SeatLock as Released if it exists
        if let Some(ref lock_id) = released_lock_id {
            if let Ok(Some(_)) = self.seat_lock_repo.find_by_id(lock_id).await {
                self.seat_lock_repo
                    .update_status(lock_id, LockStatus::Released)
                    .await?;
            }

            // Mark the associated Booking as Cancelled
            let bookings = self
                .booking_repo
                .find_by_status(BookingStatus::Pending)
                .await?;
            for mut booking in bookings {
                if booking.lock_id.as_deref() == Some(lock_id) {
                    booking.status = BookingStatus::Cancelled;
                    booking.cancelled_at = Some(Utc::now());
                    self.booking_repo.save(booking.clone()).await?;
                    self.save_audit_event(
                        &booking.booking_id,
                        &booking.show_id,
                        &booking.user_id,
                        "admin_override",
                        None,
                        Some(BookingStatus::Pending.to_string()),
                        Some(BookingStatus::Cancelled.to_string()),
                        Some(reason.to_string()),
                        Some(serde_json::json!({ "seat_id": seat_id, "lock_id": lock_id })),
                    )
                    .await;
                }
            }
        }

        tracing::warn!(
            seat_id = %seat_id,
            seat_number = %seat_number,
            lock_id = ?released_lock_id,
            reason = %reason,
            "admin force-released locked seat"
        );

        Ok(OverrideResult {
            seat_id: seat_id.to_string(),
            seat_number,
            previous_status: "locked".to_string(),
            new_status: "available".to_string(),
            released_lock_id,
        })
    }

    async fn rollback_locked_seats(&self, seat_ids: &[String]) {
        for seat_id in seat_ids {
            if let Err(e) = self.seat_repo.release_seat(seat_id).await {
                tracing::error!(seat_id = %seat_id, error = %e, "failed to roll back locked seat");
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn save_audit_event(
        &self,
        booking_id: &str,
        show_id: &str,
        user_id: &str,
        event_type: &str,
        actor_id: Option<String>,
        status_from: Option<String>,
        status_to: Option<String>,
        message: Option<String>,
        metadata: Option<serde_json::Value>,
    ) {
        let Some(repo) = &self.compensation_log_repo else {
            return;
        };

        let log = CompensationLog::audit_event(
            Uuid::new_v4().to_string(),
            booking_id.to_string(),
            show_id.to_string(),
            user_id.to_string(),
            event_type,
            actor_id,
            status_from,
            status_to,
            message,
            metadata,
        );
        if let Err(e) = repo.save(log).await {
            tracing::error!(booking_id = %booking_id, event_type = %event_type, error = %e, "failed to save audit log");
        }
    }

    async fn acquire_show_lock(&self, show_id: &str) -> ShowLockGuard {
        #[cfg(feature = "distributed-lock")]
        if let Some(redis_url) = &self.cfg.distributed_lock.redis_url {
            match RedisShowLockGuard::acquire(redis_url, show_id, &self.cfg).await {
                Ok(guard) => return ShowLockGuard::Redis(guard),
                Err(e) => tracing::warn!(
                    show_id = %show_id,
                    error = %e,
                    "redis show lock unavailable; falling back to local lock"
                ),
            }
        }

        let show_lock = self.get_show_lock(show_id).await;
        ShowLockGuard::Local(show_lock.write_owned().await)
    }

    /// Get or create the per-show Mutex guard.
    async fn get_show_lock(&self, show_id: &str) -> Arc<RwLock<()>> {
        {
            let r = self.show_locks.read().await;
            if let Some(lock) = r.get(show_id) {
                return Arc::clone(lock);
            }
        }

        let mut w = self.show_locks.write().await;
        Arc::clone(
            w.entry(show_id.to_string())
                .or_insert_with(|| Arc::new(RwLock::new(()))),
        )
    }
}

enum ShowLockGuard {
    Local(#[allow(dead_code)] OwnedRwLockWriteGuard<()>),
    #[cfg(feature = "distributed-lock")]
    Redis(#[allow(dead_code)] RedisShowLockGuard),
}

#[cfg(feature = "distributed-lock")]
struct RedisShowLockGuard {
    client: redis::Client,
    key: String,
    token: String,
    renew_task: tokio::task::JoinHandle<()>,
}

#[cfg(feature = "distributed-lock")]
impl RedisShowLockGuard {
    async fn acquire(redis_url: &str, show_id: &str, cfg: &AppConfig) -> Result<Self, AppError> {
        let client = redis::Client::open(redis_url)
            .map_err(|e| AppError::InternalError(format!("invalid redis url: {e}")))?;
        let key = format!("bms:show-lock:{show_id}");
        let token = Uuid::new_v4().to_string();
        let ttl_ms = cfg.distributed_lock.lock_ttl_ms.max(1);
        let deadline = tokio::time::Instant::now()
            + std::time::Duration::from_millis(cfg.distributed_lock.acquire_timeout_ms.max(1));

        loop {
            let mut conn = client
                .get_multiplexed_async_connection()
                .await
                .map_err(|e| AppError::InternalError(format!("redis connection failed: {e}")))?;
            let acquired: Option<String> = redis::cmd("SET")
                .arg(&key)
                .arg(&token)
                .arg("NX")
                .arg("PX")
                .arg(ttl_ms)
                .query_async(&mut conn)
                .await
                .map_err(|e| AppError::InternalError(format!("redis set nx failed: {e}")))?;

            if acquired.is_some() {
                let renew_client = client.clone();
                let renew_key = key.clone();
                let renew_token = token.clone();
                let interval_ms = cfg.distributed_lock.renewal_interval_ms.max(1).min(ttl_ms);
                let renew_task = tokio::spawn(async move {
                    let mut ticker =
                        tokio::time::interval(std::time::Duration::from_millis(interval_ms));
                    loop {
                        ticker.tick().await;
                        let Ok(mut conn) = renew_client.get_multiplexed_async_connection().await
                        else {
                            continue;
                        };
                        let _: redis::RedisResult<i64> = redis::Script::new(
                            "if redis.call('get', KEYS[1]) == ARGV[1] then return redis.call('pexpire', KEYS[1], ARGV[2]) else return 0 end",
                        )
                        .key(&renew_key)
                        .arg(&renew_token)
                        .arg(ttl_ms)
                        .invoke_async(&mut conn)
                        .await;
                    }
                });

                return Ok(Self {
                    client,
                    key,
                    token,
                    renew_task,
                });
            }

            if tokio::time::Instant::now() >= deadline {
                return Err(AppError::InternalError(
                    "redis show lock timed out".to_string(),
                ));
            }
            tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        }
    }
}

#[cfg(feature = "distributed-lock")]
impl Drop for RedisShowLockGuard {
    fn drop(&mut self) {
        self.renew_task.abort();
        let client = self.client.clone();
        let key = self.key.clone();
        let token = self.token.clone();
        tokio::spawn(async move {
            let Ok(mut conn) = client.get_multiplexed_async_connection().await else {
                return;
            };
            let _: redis::RedisResult<i64> = redis::Script::new(
                "if redis.call('get', KEYS[1]) == ARGV[1] then return redis.call('del', KEYS[1]) else return 0 end",
            )
            .key(key)
            .arg(token)
            .invoke_async(&mut conn)
            .await;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use domain::{Seat, SeatStatus, SeatType, Show};
    use repository_inmemory::{
        InMemoryBookingRepository, InMemorySeatLockRepository, InMemorySeatRepository,
        InMemoryShowRepository, InMemoryUserRepository,
    };

    fn make_test_show(id: &str) -> Show {
        Show::new(
            id.to_string(),
            "Test Movie".to_string(),
            "Test Theatre".to_string(),
            1,
            Utc.with_ymd_and_hms(2026, 5, 1, 14, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2026, 5, 1, 17, 0, 0).unwrap(),
            250.0,
            10,
        )
    }

    fn make_seat(id: &str, show_id: &str, status: SeatStatus) -> Seat {
        Seat {
            seat_id: id.to_string(),
            seat_number: id.to_string(),
            row_label: "A".to_string(),
            seat_type: SeatType::Standard,
            show_id: show_id.to_string(),
            status,
            booked_by: None,
            locked_by: None,
            locked_at: None,
            lock_expires_at: None,
            lock_id: None,
        }
    }

    fn make_cfg() -> AppConfig {
        AppConfig::default()
    }

    #[tokio::test]
    async fn test_lock_single_seat_success() {
        let show_repo = Arc::new(InMemoryShowRepository::new());
        let seat_repo = Arc::new(InMemorySeatRepository::new());
        let booking_repo = Arc::new(InMemoryBookingRepository::new());
        let seat_lock_repo = Arc::new(InMemorySeatLockRepository::new());
        let user_repo = Arc::new(InMemoryUserRepository::new());

        let show = make_test_show("show-1");
        show_repo.save(show.clone()).await.unwrap();

        let seat = make_seat("A1", "show-1", SeatStatus::Available);
        seat_repo.save(seat).await.unwrap();

        let user = domain::User::new(
            "user-1".to_string(),
            "Alice".to_string(),
            "alice@test.com".to_string(),
        );
        user_repo.save(user).await.unwrap();

        let svc = SeatLockingService::new(
            Arc::clone(&show_repo) as Arc<dyn ShowRepository>,
            Arc::clone(&seat_repo) as Arc<dyn SeatRepository>,
            Arc::clone(&booking_repo) as Arc<dyn BookingRepository>,
            Arc::clone(&seat_lock_repo) as Arc<dyn SeatLockRepository>,
            Arc::clone(&user_repo) as Arc<dyn UserRepository>,
            make_cfg(),
        );

        let result = svc
            .lock_seats("show-1", vec!["A1".to_string()], "user-1")
            .await
            .unwrap();

        assert_eq!(result.seat_ids, vec!["A1"]);
        assert_eq!(result.total_amount, 250.0); // Standard × 1.0

        let seat = seat_repo.find_by_id("A1").await.unwrap().unwrap();
        assert_eq!(seat.status, SeatStatus::Locked);
    }

    #[tokio::test]
    async fn test_lock_already_locked_seat_fails() {
        let show_repo = Arc::new(InMemoryShowRepository::new());
        let seat_repo = Arc::new(InMemorySeatRepository::new());
        let booking_repo = Arc::new(InMemoryBookingRepository::new());
        let seat_lock_repo = Arc::new(InMemorySeatLockRepository::new());
        let user_repo = Arc::new(InMemoryUserRepository::new());

        let show = make_test_show("show-1");
        show_repo.save(show.clone()).await.unwrap();

        // Seat is already locked by user-1
        let mut seat = make_seat("A1", "show-1", SeatStatus::Locked);
        seat.locked_by = Some(domain::User::new(
            "user-1".to_string(),
            "Bob".to_string(),
            "bob@test.com".to_string(),
        ));
        seat_repo.save(seat).await.unwrap();

        let user2 = domain::User::new(
            "user-2".to_string(),
            "Alice".to_string(),
            "alice@test.com".to_string(),
        );
        user_repo.save(user2).await.unwrap();

        let svc = SeatLockingService::new(
            Arc::clone(&show_repo) as Arc<dyn ShowRepository>,
            Arc::clone(&seat_repo) as Arc<dyn SeatRepository>,
            Arc::clone(&booking_repo) as Arc<dyn BookingRepository>,
            Arc::clone(&seat_lock_repo) as Arc<dyn SeatLockRepository>,
            Arc::clone(&user_repo) as Arc<dyn UserRepository>,
            make_cfg(),
        );

        let err = svc
            .lock_seats("show-1", vec!["A1".to_string()], "user-2")
            .await
            .unwrap_err();

        match err {
            AppError::SeatsUnavailable(ids) => assert_eq!(ids, vec!["A1"]),
            _ => panic!("expected SeatsUnavailable, got {err:?}"),
        }
    }

    #[tokio::test]
    async fn test_lock_multiple_seats_same_show() {
        let show_repo = Arc::new(InMemoryShowRepository::new());
        let seat_repo = Arc::new(InMemorySeatRepository::new());
        let booking_repo = Arc::new(InMemoryBookingRepository::new());
        let seat_lock_repo = Arc::new(InMemorySeatLockRepository::new());
        let user_repo = Arc::new(InMemoryUserRepository::new());

        let show = make_test_show("show-1");
        show_repo.save(show.clone()).await.unwrap();

        for i in 1..=5u32 {
            let seat = make_seat(&format!("A{i}"), "show-1", SeatStatus::Available);
            seat_repo.save(seat).await.unwrap();
        }

        let user = domain::User::new(
            "user-1".to_string(),
            "Alice".to_string(),
            "alice@test.com".to_string(),
        );
        user_repo.save(user).await.unwrap();

        let svc = SeatLockingService::new(
            Arc::clone(&show_repo) as Arc<dyn ShowRepository>,
            Arc::clone(&seat_repo) as Arc<dyn SeatRepository>,
            Arc::clone(&booking_repo) as Arc<dyn BookingRepository>,
            Arc::clone(&seat_lock_repo) as Arc<dyn SeatLockRepository>,
            Arc::clone(&user_repo) as Arc<dyn UserRepository>,
            make_cfg(),
        );

        let result = svc
            .lock_seats(
                "show-1",
                vec!["A1".to_string(), "A2".to_string(), "A3".to_string()],
                "user-1",
            )
            .await
            .unwrap();

        assert_eq!(result.seat_ids.len(), 3);
        assert_eq!(result.total_amount, 750.0); // 3 × 250

        for i in 1..=3u32 {
            let seat = seat_repo
                .find_by_id(&format!("A{i}"))
                .await
                .unwrap()
                .unwrap();
            assert_eq!(seat.status, SeatStatus::Locked);
        }
        // Unlocked seats remain available
        for i in 4..=5u32 {
            let seat = seat_repo
                .find_by_id(&format!("A{i}"))
                .await
                .unwrap()
                .unwrap();
            assert_eq!(seat.status, SeatStatus::Available);
        }
    }

    #[tokio::test]
    async fn test_concurrent_lock_same_seat_only_one_succeeds() {
        let show_repo = Arc::new(InMemoryShowRepository::new());
        let seat_repo = Arc::new(InMemorySeatRepository::new());
        let booking_repo = Arc::new(InMemoryBookingRepository::new());
        let seat_lock_repo = Arc::new(InMemorySeatLockRepository::new());
        let user_repo = Arc::new(InMemoryUserRepository::new());

        let show = make_test_show("show-1");
        show_repo.save(show.clone()).await.unwrap();

        let seat = make_seat("A1", "show-1", SeatStatus::Available);
        seat_repo.save(seat).await.unwrap();

        let user1 = domain::User::new(
            "user-1".to_string(),
            "Alice".to_string(),
            "alice@test.com".to_string(),
        );
        let user2 = domain::User::new(
            "user-2".to_string(),
            "Bob".to_string(),
            "bob@test.com".to_string(),
        );
        user_repo.save(user1).await.unwrap();
        user_repo.save(user2).await.unwrap();

        let svc = SeatLockingService::new(
            Arc::clone(&show_repo) as Arc<dyn ShowRepository>,
            Arc::clone(&seat_repo) as Arc<dyn SeatRepository>,
            Arc::clone(&booking_repo) as Arc<dyn BookingRepository>,
            Arc::clone(&seat_lock_repo) as Arc<dyn SeatLockRepository>,
            Arc::clone(&user_repo) as Arc<dyn UserRepository>,
            make_cfg(),
        );

        // Spawn 50 concurrent lock attempts for the same seat
        let mut handles = vec![];
        for i in 0..50u32 {
            let user_id = if i % 2 == 0 { "user-1" } else { "user-2" };
            let svc = svc.clone();
            handles.push(tokio::spawn(async move {
                svc.lock_seats("show-1", vec!["A1".to_string()], user_id)
                    .await
            }));
        }

        let results: Vec<Result<LockResult, AppError>> = futures::future::join_all(handles)
            .await
            .into_iter()
            .map(|r| r.unwrap())
            .collect();

        // Exactly ONE success
        let successes: Vec<_> = results.iter().filter(|r| r.is_ok()).collect();
        let failures: Vec<_> = results.iter().filter(|r| r.is_err()).collect();

        assert_eq!(
            successes.len(),
            1,
            "exactly one lock attempt should succeed"
        );
        assert_eq!(failures.len(), 49, "all other attempts should fail");
    }

    #[tokio::test]
    async fn test_lock_extension() {
        let show_repo = Arc::new(InMemoryShowRepository::new());
        let seat_repo = Arc::new(InMemorySeatRepository::new());
        let booking_repo = Arc::new(InMemoryBookingRepository::new());
        let seat_lock_repo = Arc::new(InMemorySeatLockRepository::new());
        let user_repo = Arc::new(InMemoryUserRepository::new());

        let show = make_test_show("show-1");
        show_repo.save(show.clone()).await.unwrap();

        let seat = make_seat("A1", "show-1", SeatStatus::Available);
        seat_repo.save(seat).await.unwrap();

        let user = domain::User::new(
            "user-1".to_string(),
            "Alice".to_string(),
            "alice@test.com".to_string(),
        );
        user_repo.save(user).await.unwrap();

        let svc = SeatLockingService::new(
            Arc::clone(&show_repo) as Arc<dyn ShowRepository>,
            Arc::clone(&seat_repo) as Arc<dyn SeatRepository>,
            Arc::clone(&booking_repo) as Arc<dyn BookingRepository>,
            Arc::clone(&seat_lock_repo) as Arc<dyn SeatLockRepository>,
            Arc::clone(&user_repo) as Arc<dyn UserRepository>,
            make_cfg(),
        );

        let lock_result = svc
            .lock_seats("show-1", vec!["A1".to_string()], "user-1")
            .await
            .unwrap();

        let first_expires = lock_result.expires_at;

        // Extend once
        let extended = svc
            .extend_lock(&lock_result.booking_id, "user-1")
            .await
            .unwrap();
        assert!(extended.expires_at > first_expires);

        // Extend again (max_extensions = 2 in default config)
        let extended2 = svc
            .extend_lock(&lock_result.booking_id, "user-1")
            .await
            .unwrap();
        assert!(extended2.expires_at > extended.expires_at);

        // Third extension should fail
        let err = svc
            .extend_lock(&lock_result.booking_id, "user-1")
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::LockMaxExtensionsReached));
    }

    #[tokio::test]
    async fn test_release_lock() {
        let show_repo = Arc::new(InMemoryShowRepository::new());
        let seat_repo = Arc::new(InMemorySeatRepository::new());
        let booking_repo = Arc::new(InMemoryBookingRepository::new());
        let seat_lock_repo = Arc::new(InMemorySeatLockRepository::new());
        let user_repo = Arc::new(InMemoryUserRepository::new());

        let show = make_test_show("show-1");
        show_repo.save(show.clone()).await.unwrap();

        let seat = make_seat("A1", "show-1", SeatStatus::Available);
        seat_repo.save(seat).await.unwrap();

        let user = domain::User::new(
            "user-1".to_string(),
            "Alice".to_string(),
            "alice@test.com".to_string(),
        );
        user_repo.save(user).await.unwrap();

        let svc = SeatLockingService::new(
            Arc::clone(&show_repo) as Arc<dyn ShowRepository>,
            Arc::clone(&seat_repo) as Arc<dyn SeatRepository>,
            Arc::clone(&booking_repo) as Arc<dyn BookingRepository>,
            Arc::clone(&seat_lock_repo) as Arc<dyn SeatLockRepository>,
            Arc::clone(&user_repo) as Arc<dyn UserRepository>,
            make_cfg(),
        );

        let lock_result = svc
            .lock_seats("show-1", vec!["A1".to_string()], "user-1")
            .await
            .unwrap();

        svc.release_lock(&lock_result.booking_id, "user-1")
            .await
            .unwrap();

        let seat = seat_repo.find_by_id("A1").await.unwrap().unwrap();
        assert_eq!(seat.status, SeatStatus::Available);
    }
}
