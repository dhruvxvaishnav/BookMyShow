use chrono::{Duration, Utc};
use common::{AppConfig, AppError};
use domain::{Booking, BookingStatus, LockStatus, SeatLock, SeatStatus};
use repository::{
    BookingRepository, SeatLockRepository, SeatRepository, ShowRepository, UserRepository,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
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
            show_locks: Arc::new(RwLock::new(HashMap::new())),
            cfg,
        }
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
            return Err(AppError::SeatNotFound((*missing.first().unwrap()).clone()));
        }

        // All seats must belong to the same show
        if !seats.iter().all(|s| s.show_id == show_id) {
            return Err(AppError::SeatsMustBelongToSameShow);
        }

        // ── Acquire per-show Mutex ────────────────────────────────────────────
        let show_lock = self.get_show_lock(show_id).await;
        let _guard = show_lock.write().await;

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

        // Lock all seats
        for seat in &current_seats {
            self.seat_repo
                .lock_seat(&seat.seat_id, user_id, &lock_id, expires_at)
                .await?;
        }

        // Create SeatLock record
        let seat_lock = SeatLock::new(
            lock_id.clone(),
            user_id.to_string(),
            show_id.to_string(),
            seat_ids.clone(),
            expires_at,
        );
        self.seat_lock_repo.save(seat_lock).await?;

        // Calculate total amount
        let show = self.show_repo.find_by_id(show_id).await?.unwrap();
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
        self.booking_repo.save(booking).await?;

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
        let updated_lock = self.seat_lock_repo.find_by_id(&lock_id).await?.unwrap();
        let mut updated_lock = updated_lock;
        updated_lock.expires_at = new_expires_at;
        self.seat_lock_repo.save(updated_lock.clone()).await?;

        // Update all locked seats' expiry timestamps
        let seat_ids_clone = booking.seat_ids.clone();
        let show_id_clone = booking.show_id.clone();
        for seat_id in &seat_ids_clone {
            let seat = self.seat_repo.find_by_id(seat_id).await?.unwrap();
            let mut updated_seat = seat;
            updated_seat.lock_expires_at = Some(new_expires_at);
            self.seat_repo.save(updated_seat).await?;
        }

        // Update booking expiry
        let mut updated_booking = booking;
        updated_booking.expires_at = new_expires_at;
        self.booking_repo.save(updated_booking.clone()).await?;

        tracing::info!(
            booking_id = %booking_id,
            lock_id = %lock_id,
            new_expires_at = %new_expires_at,
            extension_count = updated_lock.extended_count,
            "lock extended"
        );

        let show = self.show_repo.find_by_id(&show_id_clone).await?.unwrap();
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
        let mut updated_booking = booking;
        updated_booking.status = BookingStatus::Cancelled;
        updated_booking.cancelled_at = Some(Utc::now());
        self.booking_repo.save(updated_booking).await?;

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
                    ..booking
                })
                .await?;
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
        let show_lock = self.get_show_lock(&seat.show_id).await;
        let _guard = show_lock.write().await;

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
                    self.booking_repo.save(booking).await?;
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

    /// Get or create the per-show Mutex guard.
    async fn get_show_lock(&self, show_id: &str) -> Arc<RwLock<()>> {
        {
            let r = self.show_locks.read().await;
            if let Some(lock) = r.get(show_id) {
                return Arc::clone(lock);
            }
        }

        let mut w = self.show_locks.write().await;
        w.entry(show_id.to_string())
            .or_insert_with(|| Arc::new(RwLock::new(())));
        Arc::clone(w.get(show_id).unwrap())
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
