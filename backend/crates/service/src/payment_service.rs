use async_trait::async_trait;
use chrono::Utc;
use common::AppConfig;
use domain::{BookingStatus, CompensationLog, Payment, PaymentStatus};
use repository::{BookingRepository, CompensationLogRepository, PaymentRepository, UserRepository};
use std::sync::Arc;
use uuid::Uuid;

use super::booking_service::BookingServiceTrait;
use super::payment::{MockPaymentRequest, MockPaymentResponse, PaymentInitiated};
use crate::email_service::EmailServiceTrait;

#[async_trait]
pub trait PaymentServiceTrait: Send + Sync {
    /// Initiate a payment for a booking. Creates a Payment record and returns
    /// a payment_intent_id. In production this would call a real gateway;
    /// here we simulate it with a mock.
    async fn initiate_payment(
        &self,
        booking_id: &str,
        user_id: &str,
        idempotency_key: Option<String>,
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
    async fn get_payment(&self, payment_id: &str) -> Result<Option<Payment>, common::AppError>;

    /// Trigger a mock gateway payment (simulates the gateway processing).
    async fn mock_gateway_pay(
        &self,
        req: MockPaymentRequest,
    ) -> Result<MockPaymentResponse, common::AppError>;

    /// Issue a refund for a payment.
    async fn refund_payment(&self, payment_id: &str) -> Result<(), common::AppError>;

    /// Background task to process expired pending payments.
    async fn process_expired_payments(&self) -> Result<(), common::AppError>;
}

pub struct PaymentService {
    payment_repo: Arc<dyn PaymentRepository>,
    booking_repo: Arc<dyn BookingRepository>,
    user_repo: Arc<dyn UserRepository>,
    booking_svc: Arc<dyn BookingServiceTrait>,
    email_svc: Arc<dyn EmailServiceTrait>,
    compensation_log_repo: Option<Arc<dyn CompensationLogRepository>>,
    cfg: AppConfig,
    http_client: reqwest::Client,
}

impl PaymentService {
    pub fn new(
        payment_repo: Arc<dyn PaymentRepository>,
        booking_repo: Arc<dyn BookingRepository>,
        user_repo: Arc<dyn UserRepository>,
        booking_svc: Arc<dyn BookingServiceTrait>,
        email_svc: Arc<dyn EmailServiceTrait>,
        cfg: AppConfig,
    ) -> Self {
        Self {
            payment_repo,
            booking_repo,
            user_repo,
            booking_svc,
            email_svc,
            compensation_log_repo: None,
            cfg,
            http_client: reqwest::Client::new(),
        }
    }

    pub fn with_audit_log_repo(mut self, repo: Arc<dyn CompensationLogRepository>) -> Self {
        self.compensation_log_repo = Some(repo);
        self
    }
}

#[async_trait]
impl PaymentServiceTrait for PaymentService {
    async fn initiate_payment(
        &self,
        booking_id: &str,
        user_id: &str,
        idempotency_key: Option<String>,
    ) -> Result<PaymentInitiated, common::AppError> {
        // If idempotency key is provided, check if payment already exists for this key
        if let Some(key) = &idempotency_key
            && let Some(existing) = self.payment_repo.find_by_idempotency_key(key).await?
        {
            return Ok(PaymentInitiated {
                payment_id: existing.payment_id,
                payment_intent_id: existing.payment_intent_id,
                amount: existing.amount,
                gateway_name: existing.gateway_name,
                client_secret: None,
            });
        }

        let booking = self
            .booking_repo
            .find_by_id(booking_id)
            .await?
            .ok_or_else(|| common::AppError::BookingNotFound(booking_id.to_string()))?;

        // Check legacy idempotency: if payment already exists for this booking, return it
        if let Some(existing) = self.payment_repo.find_by_booking(booking_id).await? {
            return Ok(PaymentInitiated {
                payment_id: existing.payment_id,
                payment_intent_id: existing.payment_intent_id,
                amount: existing.amount,
                gateway_name: existing.gateway_name,
                client_secret: None,
            });
        }

        if !booking.can_pay() {
            return Err(common::AppError::BookingAlreadyProcessed(
                booking_id.to_string(),
            ));
        }

        if booking.user_id != user_id {
            return Err(common::AppError::LockNotOwnedByUser);
        }

        let mut payment_intent_id = Uuid::new_v4().to_string();
        let payment_id = Uuid::new_v4().to_string();
        let mut gateway_name = "mock".to_string();
        let mut client_secret = None;

        if let Some(stripe_key) = &self.cfg.payment.stripe_secret_key {
            gateway_name = "stripe".to_string();
            let amount_cents = (booking.total_amount * 100.0).round() as u64;

            let mut req = self
                .http_client
                .post("https://api.stripe.com/v1/payment_intents")
                .bearer_auth(stripe_key)
                .form(&[
                    ("amount", amount_cents.to_string()),
                    ("currency", "inr".to_string()),
                    ("metadata[booking_id]", booking_id.to_string()),
                    ("metadata[payment_id]", payment_id.clone()),
                ]);

            if let Some(key) = &idempotency_key {
                req = req.header("Idempotency-Key", key);
            }

            let res = req.send().await.map_err(|e| {
                tracing::error!("stripe error: {}", e);
                common::AppError::InternalError("Failed to contact payment gateway".to_string())
            })?;

            let body: serde_json::Value = res.json().await.map_err(|_| {
                common::AppError::InternalError("Invalid gateway response".to_string())
            })?;

            if let Some(id) = body.get("id").and_then(|i: &serde_json::Value| i.as_str()) {
                payment_intent_id = id.to_string();
                client_secret = body
                    .get("client_secret")
                    .and_then(|cs: &serde_json::Value| cs.as_str())
                    .map(|s| s.to_string());
            } else {
                tracing::error!("stripe error response: {:?}", body);
                return Err(common::AppError::InternalError(
                    "Failed to initiate payment".to_string(),
                ));
            }
        }

        let payment = Payment::new(
            payment_id.clone(),
            payment_intent_id.clone(),
            booking_id.to_string(),
            user_id.to_string(),
            booking.total_amount,
            gateway_name.clone(),
            idempotency_key,
        );

        self.payment_repo.save(payment).await?;

        // Update booking status — clone needed fields before moving
        let booking_total = booking.total_amount;
        let booking_show_id = booking.show_id.clone();
        let booking_status = booking.status.to_string();
        let mut updated_booking = booking;
        updated_booking.status = BookingStatus::PaymentPending;
        updated_booking.payment_id = Some(payment_id.clone());
        self.booking_repo.save(updated_booking).await?;

        self.save_audit_event(
            booking_id,
            &booking_show_id,
            user_id,
            "pay",
            Some(user_id.to_string()),
            Some(booking_status),
            Some(BookingStatus::PaymentPending.to_string()),
            Some("payment initiated".to_string()),
            Some(serde_json::json!({
                "payment_id": payment_id.clone(),
                "payment_intent_id": payment_intent_id.clone(),
                "gateway": gateway_name.clone()
            })),
        )
        .await;

        tracing::info!(
            payment_id = %payment_id,
            payment_intent_id = %payment_intent_id,
            booking_id = %booking_id,
            amount = booking_total,
            gateway = %gateway_name,
            "payment initiated"
        );

        Ok(PaymentInitiated {
            payment_id,
            payment_intent_id,
            amount: booking_total,
            gateway_name,
            client_secret,
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
                self.save_audit_event(
                    &booking_id,
                    "",
                    &updated_payment.user_id,
                    "payment_callback_compensation_required",
                    None,
                    Some(PaymentStatus::Pending.to_string()),
                    Some(PaymentStatus::Success.to_string()),
                    Some("payment succeeded but booking confirmation failed".to_string()),
                    Some(serde_json::json!({
                        "payment_id": payment_id.clone(),
                        "payment_intent_id": payment_intent_id,
                        "error": e.to_string()
                    })),
                )
                .await;
                return Err(e);
            }
            self.save_audit_event(
                &booking_id,
                "",
                &updated_payment.user_id,
                "payment_callback",
                None,
                Some(PaymentStatus::Pending.to_string()),
                Some(PaymentStatus::Success.to_string()),
                Some("payment callback marked payment successful".to_string()),
                Some(serde_json::json!({
                    "payment_id": payment_id.clone(),
                    "payment_intent_id": payment_intent_id
                })),
            )
            .await;
        } else {
            if let Ok(Some(user)) = self.user_repo.find_by_id(&updated_payment.user_id).await {
                let _ = self
                    .email_svc
                    .send_payment_failed(&user.email, &booking_id)
                    .await;
            }

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
            self.save_audit_event(
                &booking_id,
                "",
                &updated_payment.user_id,
                "payment_callback",
                None,
                Some(PaymentStatus::Pending.to_string()),
                Some(PaymentStatus::Failed.to_string()),
                Some("payment callback marked payment failed".to_string()),
                Some(serde_json::json!({
                    "payment_id": payment_id.clone(),
                    "payment_intent_id": payment_intent_id
                })),
            )
            .await;
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

    async fn refund_payment(&self, payment_id: &str) -> Result<(), common::AppError> {
        let mut payment = self
            .payment_repo
            .find_by_id(payment_id)
            .await?
            .ok_or_else(|| common::AppError::PaymentNotFound(payment_id.to_string()))?;

        if payment.status != PaymentStatus::Success {
            return Err(common::AppError::ValidationError(
                "Only successful payments can be refunded".to_string(),
            ));
        }

        payment.status = PaymentStatus::Refunded;
        self.payment_repo.save(payment.clone()).await?;

        // Release seats if booking exists
        if let Some(booking) = self.booking_repo.find_by_payment_id(payment_id).await? {
            // This is simplified. In a real system, we might want a BookingService
            // method to handle refunds properly.
            let _ = self
                .booking_svc
                .cancel_booking(&booking.booking_id, &booking.user_id)
                .await;
        }

        tracing::info!(payment_id = %payment_id, "payment refunded");

        self.save_audit_event(
            &payment.booking_id,
            "",
            &payment.user_id,
            "refund",
            None,
            Some(PaymentStatus::Success.to_string()),
            Some(PaymentStatus::Refunded.to_string()),
            Some("payment refunded".to_string()),
            Some(serde_json::json!({ "payment_id": payment.payment_id })),
        )
        .await;
        Ok(())
    }

    async fn process_expired_payments(&self) -> Result<(), common::AppError> {
        let timeout_secs = self.cfg.payment.timeout_seconds as i64;
        let before = Utc::now() - chrono::Duration::seconds(timeout_secs);

        let expired = self.payment_repo.find_expired_pending(before).await?;
        for mut payment in expired {
            payment.status = PaymentStatus::Failed;
            payment.failed_at = Some(Utc::now());
            self.payment_repo.save(payment.clone()).await?;

            tracing::info!(payment_id = %payment.payment_id, "payment expired and marked as failed");

            // Also trigger cancellation of booking if applicable
            if let Some(booking) = self.booking_repo.find_by_id(&payment.booking_id).await? {
                let _ = self
                    .booking_svc
                    .cancel_booking(&booking.booking_id, &booking.user_id)
                    .await;
            }

            self.save_audit_event(
                &payment.booking_id,
                "",
                &payment.user_id,
                "payment_timeout",
                None,
                Some(PaymentStatus::Pending.to_string()),
                Some(PaymentStatus::Failed.to_string()),
                Some("pending payment expired".to_string()),
                Some(serde_json::json!({ "payment_id": payment.payment_id })),
            )
            .await;
        }
        Ok(())
    }
}

impl PaymentService {
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

        let resolved_show_id = if show_id.is_empty() {
            self.booking_repo
                .find_by_id(booking_id)
                .await
                .ok()
                .flatten()
                .map(|b| b.show_id)
                .unwrap_or_default()
        } else {
            show_id.to_string()
        };

        let log = CompensationLog::audit_event(
            Uuid::new_v4().to_string(),
            booking_id.to_string(),
            resolved_show_id,
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
}

/// Simple deterministic pseudo-random for tests / mock gateway.
fn rand_simple() -> f64 {
    use rand::Rng;
    rand::thread_rng().gen_range(0.0..1.0)
}
