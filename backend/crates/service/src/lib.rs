pub mod booking;
pub mod payment;
pub mod queue;
pub mod seat_locking;
pub mod show;

pub mod booking_service;
pub mod email_service;
pub mod movie_service;
pub mod payment_service;
pub mod queue_service;
pub mod seat_locking_service;
pub mod show_service;
pub mod venue_service;

pub use booking_service::BookingService;
pub use email_service::{EmailService, EmailServiceTrait};
pub use movie_service::MovieService;
pub use payment_service::PaymentService;
pub use queue_service::QueueService;
pub use seat_locking_service::SeatLockingService;
pub use show_service::ShowService;
pub use venue_service::VenueService;
