use std::collections::HashMap;
use crate::movie_seat_booking::model::booking::Booking;
use crate::movie_seat_booking::model::booking_status::BookingStatus;
use crate::movie_seat_booking::model::show::Show;
use crate::movie_seat_booking::model::user::User;
use super::booking_repository::BookingRepository;

pub struct InMemoryBookingRepository {
    booking_map: HashMap<String, Booking>,
}

impl InMemoryBookingRepository {
    pub fn new() -> Self {
        InMemoryBookingRepository { booking_map: HashMap::new() }
    }
}

impl BookingRepository for InMemoryBookingRepository {
    fn save(&mut self, booking: Booking) -> Booking {
        self.booking_map.insert(booking.booking_id.clone(), booking.clone());
        booking
    }

    fn find_by_id(&self, booking_id: &str) -> Option<Booking> {
        self.booking_map.get(booking_id).cloned()
    }

    fn find_by_user(&self, user: &User) -> Vec<Booking> {
        self.booking_map.values()
            .filter(|b| b.user.user_id == user.user_id)
            .cloned()
            .collect()
    }

    fn find_by_show(&self, show: &Show) -> Vec<Booking> {
        self.booking_map.values()
            .filter(|b| b.show.show_id == show.show_id)
            .cloned()
            .collect()
    }

    fn find_by_status(&self, status: &BookingStatus) -> Vec<Booking> {
        self.booking_map.values()
            .filter(|b| &b.status == status)
            .cloned()
            .collect()
    }

    fn find_by_payment_id(&self, payment_id: &str) -> Option<Booking> {
        self.booking_map.values()
            .find(|b| b.payment_id.as_deref() == Some(payment_id))
            .cloned()
    }
}