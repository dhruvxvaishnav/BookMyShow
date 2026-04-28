use async_trait::async_trait;
use chrono::Utc;
use common::AppConfig;
use domain::{BookingStatus, Payment, PaymentStatus};
use repository::{BookingRepository, PaymentRepository};
use std::sync::Arc;
use uuid::Uuid;

use super::payment::{PaymentInitiated, MockPaymentRequest, MockPaymentResponse};
use super::booking_service::BookingServiceTrait;

#[async_trait]
pub trait PaymentServiceTrait: Send + Sync {
    /// Initiate a payment for a booking. Creates a Payment record and returns
    /// a payment_intent_id. In production this would call a real gateway;
    /// here we simulate it with a mock.
    async fn initiate_payment(
        &self,
        booking_id: &str,
        user_id: &str,
    ) -> Result<PaymentInitiated, common::AppError>;

    /// Handle payment gateway callback. This is called by the gateway (or mock)
    /// after payment processing completes.
    async fn payment_callback(
        &self,
        payment_intent_id: &str,
        status: &str,
        gateway_response: Option<&str>,
    ) -> Result<(), common::AppError>;

    /// Get payment status.
    async fn get_payment(
        &self,
        payment_id: &str,
    ) -> Result<Option<Payment>, common::AppError>;

    /// Trigger a mock gateway payment (simulates the gateway processing).
    async fn mock_gateway_pay(
        &self,
        req: MockPaymentRequest,
    ) -> Result<MockPaymentResponse, common::AppError>;
}

pub struct PaymentService {
    payment_repo: Arc<dyn PaymentRepository>,
    booking_repo: Arc<dyn BookingRepository>,
    booking_svc: Arc<dyn BookingServiceTrait>,
    cfg: AppConfig,
}

impl PaymentService {
    pub fn new(
        payment_repo: Arc<dyn PaymentRepository>,
        booking_repo: Arc<dyn BookingRepository>,
        booking_svc: Arc<dyn BookingServiceTrait>,
        cfg: AppConfig,
    ) -> Self {
        Self {
            payment_repo,
            booking_repo,
            booking_svc,
            cfg,
        }
    }
}

#[async_trait]
impl PaymentServiceTrait for PaymentService {
    async fn initiate_payment(
        &self,
        booking_id: &str,
        user_id: &str,
    ) -> Result<PaymentInitiated, common::AppError> {
        let booking = self
            .booking_repo
            .find_by_id(booking_id)
            .await?
            .ok_or_else(|| common::AppError::BookingNotFound(booking_id.to_string()))?;

        // Check idempotency: if payment already exists for this booking, return it
        if let Some(existing) = self.payment_repo.find_by_booking(booking_id).await? {
            return Ok(PaymentInitiated {
                payment_id: existing.payment_id,
                payment_intent_id: existing.payment_intent_id,
                amount: existing.amount,
                gateway_name: existing.gateway_name,
            });
        }

        if !booking.can_pay() {
            return Err(common::AppError::BookingAlreadyProcessed(booking_id.to_string()));
        }

        if booking.user_id != user_id {
            return Err(common::AppError::LockNotOwnedByUser);
        }

        let payment_intent_id = Uuid::new_v4().to_string();
        let payment_id = Uuid::new_v4().to_string();

        let payment = Payment::new(
            payment_id.clone(),
            payment_intent_id.clone(),
            booking_id.to_string(),
            user_id.to_string(),
            booking.total_amount,
            "mock".to_string(),
        );

        self.payment_repo.save(payment).await?;

        // Update booking status — clone needed fields before moving
        let booking_total = booking.total_amount;
        let mut updated_booking = booking;
        updated_booking.status = BookingStatus::PaymentPending;
        updated_booking.payment_id = Some(payment_id.clone());
        self.booking_repo.save(updated_booking).await?;

        tracing::info!(
            payment_id = %payment_id,
            payment_intent_id = %payment_intent_id,
            booking_id = %booking_id,
            amount = booking_total,
            "payment initiated"
        );

        Ok(PaymentInitiated {
            payment_id,
            payment_intent_id,
            amount: booking_total,
            gateway_name: "mock".to_string(),
        })
    }

    async fn payment_callback(
        &self,
        payment_intent_id: &str,
        status: &str,
        gateway_response: Option<&str>,
    ) -> Result<(), common::AppError> {
        let payment = self
            .payment_repo
            .find_by_payment_intent_id(payment_intent_id)
            .await?
            .ok_or_else(|| common::AppError::PaymentNotFound(payment_intent_id.to_string()))?;

        // Idempotency: ignore if already processed
        if !matches!(payment.status, PaymentStatus::Pending) {
            tracing::warn!(
                payment_id = %payment.payment_id,
                status = ?payment.status,
                "duplicate payment callback ignored"
            );
            return Ok(());
        }

        let parsed_status = match status {
            "SUCCESS" => PaymentStatus::Success,
            "FAILED" => PaymentStatus::Failed,
            _ => PaymentStatus::Failed,
        };

        let mut updated_payment = payment.clone();
        updated_payment.status = parsed_status;

        if parsed_status == PaymentStatus::Success {
            updated_payment.confirmed_at = Some(Utc::now());
        } else {
            updated_payment.failed_at = Some(Utc::now());
        }

        if let Some(resp) = gateway_response {
            updated_payment.gateway_response = Some(resp.to_string());
        }

        self.payment_repo.save(updated_payment.clone()).await?;

        let booking_id = updated_payment.booking_id.clone();
        let payment_id = updated_payment.payment_id.clone();

        if updated_payment.status == PaymentStatus::Success {
            // Confirm the booking
            if let Err(e) = self
                .booking_svc
                .confirm_booking(&booking_id, &payment_id)
                .await
            {
                tracing::error!(
                    booking_id = %booking_id,
                    error = %e,
                    "failed to confirm booking after payment success"
                );
                return Err(e);
            }
        } else {
            // Payment failed — cancel the booking and release seats
            if let Err(e) = self
                .booking_svc
                .cancel_booking(&booking_id, &updated_payment.user_id)
                .await
            {
                tracing::error!(
                    booking_id = %booking_id,
                    error = %e,
                    "failed to cancel booking after payment failure"
                );
            }
        }

        tracing::info!(
            payment_id = %payment_id,
            booking_id = %booking_id,
            status = ?updated_payment.status,
            "payment callback processed"
        );

        Ok(())
    }

    async fn get_payment(&self, payment_id: &str) -> Result<Option<Payment>, common::AppError> {
        self.payment_repo.find_by_id(payment_id).await
    }

    async fn mock_gateway_pay(
        &self,
        req: MockPaymentRequest,
    ) -> Result<MockPaymentResponse, common::AppError> {
        // Simulate gateway delay
        if req.simulate_delay_ms > 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(req.simulate_delay_ms)).await;
        }

        let status = if req.simulate_failure {
            "FAILED".to_string()
        } else {
            // Use configured failure rate
            let failure_rate = self.cfg.payment.mock_gateway_failure_rate;
            if rand_simple() < failure_rate {
                "FAILED".to_string()
            } else {
                "SUCCESS".to_string()
            }
        };

        Ok(MockPaymentResponse {
            status,
            gateway_reference: format!("GW-{}", Uuid::new_v4()),
        })
    }
}

/// Simple deterministic pseudo-random for tests / mock gateway.
fn rand_simple() -> f64 {
    use rand::Rng;
    rand::thread_rng().gen_range(0.0..1.0)
}
