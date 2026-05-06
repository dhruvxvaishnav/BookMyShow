pub mod pg_booking_repository;
pub mod pg_compensation_log_repository;
pub mod pg_movie_repository;
pub mod pg_payment_repository;
pub mod pg_queue_repository;
pub mod pg_seat_lock_repository;
pub mod pg_seat_repository;
pub mod pg_show_repository;
pub mod pg_user_repository;
pub mod pg_venue_repository;

pub use pg_booking_repository::PgBookingRepository;
pub use pg_compensation_log_repository::PgCompensationLogRepository;
pub use pg_movie_repository::PgMovieRepository;
pub use pg_payment_repository::PgPaymentRepository;
pub use pg_queue_repository::PgQueueRepository;
pub use pg_seat_lock_repository::PgSeatLockRepository;
pub use pg_seat_repository::PgSeatRepository;
pub use pg_show_repository::PgShowRepository;
pub use pg_user_repository::PgUserRepository;
pub use pg_venue_repository::PgVenueRepository;

fn db_err(e: sqlx::Error) -> common::AppError {
    common::AppError::RepositoryError(e.to_string())
}
