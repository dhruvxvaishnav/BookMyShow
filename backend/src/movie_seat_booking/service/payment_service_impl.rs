use uuid::Uuid;
use crate::movie_seat_booking::model::payment::Payment;
use crate::movie_seat_booking::repository::booking_repository::BookingRepository;
use crate::movie_seat_booking::repository::payment_repository::PaymentRepository;
use crate::movie_seat_booking::model::payment_status::PaymentStatus;
use super::payment_service::PaymentService;
use super::seat_booking_service::SeatBookingService;


pub struct PaymentServiceImpl{
    booking_repository: Box<dyn BookingRepository>,
    payment_repository: Box<dyn PaymentRepository>,
    seat_booking_service: Box<dyn SeatBookingService>
}

impl PaymentServiceImpl{
    pub fn new(
        booking_repository: Box<dyn BookingRepository>,
        payment_repository: Box<dyn PaymentRepository>,
        seat_booking_service: Box<dyn SeatBookingService>
    ) -> Self {
        PaymentServiceImpl {
            booking_repository,
            payment_repository,
            seat_booking_service,
        }
    }
}

impl PaymentService for PaymentServiceImpl {
    fn initiate_payment(&mut self, payment_intent_id: &str, booking_id: &str) {
        let payment_id = Uuid::new_v4().to_string();
        let mut booking = self.booking_repository.find_by_id(booking_id).expect("Booking not found");
        let user_id = booking.user.user_id.clone();
        let payment = Payment::new(
            payment_id.clone(),
            payment_intent_id.to_string(),
            user_id,
            100.0,
            booking_id.to_string(),
        );
        booking.payment_id= Some(payment_id);
        self.booking_repository.save(booking);
        self.payment_repository.save(payment);

        //We are mimicking the call to gateway and waiting for the callback
    }
    fn payment_callback(&mut self, payment_intent_id: &str, status: &str) {
        let mut payment = self.payment_repository.find_by_payment_intent_id(payment_intent_id).expect("payment not found");
        payment.status = PaymentStatus::from_str(status);
        self.payment_repository.save(payment.clone());
    //     If success - confirm booking
    //     If failed - cancel booking
        if payment.status == PaymentStatus::Success {
            self.seat_booking_service.confirm_booking(&payment.booking_id, &payment.payment_id);
        } else {
            self.seat_booking_service.mark_booking_failed(&payment.booking_id, Some(&payment.payment_id));
        }
    }
}
