use async_trait::async_trait;
use common::AppConfig;

#[async_trait]
pub trait EmailServiceTrait: Send + Sync {
    async fn send_booking_confirmation(
        &self,
        to_email: &str,
        booking_id: &str,
        show_name: &str,
        seat_numbers: &[String],
        total_amount: f64,
    ) -> Result<(), common::AppError>;

    async fn send_payment_failed(
        &self,
        to_email: &str,
        booking_id: &str,
    ) -> Result<(), common::AppError>;

    async fn send_booking_cancelled(
        &self,
        to_email: &str,
        booking_id: &str,
    ) -> Result<(), common::AppError>;

    async fn send_lock_expiry_warning(
        &self,
        to_email: &str,
        booking_id: &str,
    ) -> Result<(), common::AppError>;
}

pub struct EmailService {
    cfg: AppConfig,
    http_client: reqwest::Client,
}

impl EmailService {
    pub fn new(cfg: AppConfig) -> Self {
        Self {
            cfg,
            http_client: reqwest::Client::new(),
        }
    }

    async fn send_email(
        &self,
        to: &str,
        subject: &str,
        html_body: &str,
    ) -> Result<(), common::AppError> {
        let api_key = match &self.cfg.email.api_key {
            Some(k) => k,
            None => {
                tracing::info!(
                    to = %to,
                    subject = %subject,
                    "Email not sent (no API key configured). Content: {}", html_body
                );
                return Ok(());
            }
        };

        let from_address = &self.cfg.email.from_address;

        let payload = serde_json::json!({
            "from": from_address,
            "to": [to],
            "subject": subject,
            "html": html_body
        });

        let res = self
            .http_client
            .post("https://api.resend.com/emails")
            .bearer_auth(api_key)
            .json(&payload)
            .send()
            .await;

        match res {
            Ok(response) if response.status().is_success() => {
                tracing::info!(to = %to, subject = %subject, "email sent successfully");
                Ok(())
            }
            Ok(response) => {
                tracing::error!("failed to send email: {:?}", response.text().await);
                Err(common::AppError::InternalError(
                    "Failed to send email".to_string(),
                ))
            }
            Err(e) => {
                tracing::error!("email network error: {}", e);
                Err(common::AppError::InternalError(
                    "Email service unavailable".to_string(),
                ))
            }
        }
    }
}

#[async_trait]
impl EmailServiceTrait for EmailService {
    async fn send_booking_confirmation(
        &self,
        to_email: &str,
        booking_id: &str,
        show_name: &str,
        seat_numbers: &[String],
        total_amount: f64,
    ) -> Result<(), common::AppError> {
        let subject = format!("Booking Confirmed: {}", show_name);
        let html_body = format!(
            "<h1>Your Booking is Confirmed!</h1>
             <p><strong>Show:</strong> {}</p>
             <p><strong>Booking ID:</strong> {}</p>
             <p><strong>Seats:</strong> {}</p>
             <p><strong>Total Amount:</strong> ₹{:.2}</p>
             <p>Thank you for choosing BookMyShow.</p>",
            show_name,
            booking_id,
            seat_numbers.join(", "),
            total_amount
        );
        self.send_email(to_email, &subject, &html_body).await
    }

    async fn send_payment_failed(
        &self,
        to_email: &str,
        booking_id: &str,
    ) -> Result<(), common::AppError> {
        let subject = "Payment Failed for Booking".to_string();
        let html_body = format!(
            "<h1>Payment Failed</h1>
             <p>Your payment for Booking ID <strong>{}</strong> has failed.</p>
             <p>Please try again before your seat lock expires.</p>",
            booking_id
        );
        self.send_email(to_email, &subject, &html_body).await
    }

    async fn send_booking_cancelled(
        &self,
        to_email: &str,
        booking_id: &str,
    ) -> Result<(), common::AppError> {
        let subject = "Booking Cancelled".to_string();
        let html_body = format!(
            "<h1>Booking Cancelled</h1>
             <p>Your Booking ID <strong>{}</strong> has been cancelled.</p>
             <p>If you have made a payment, it will be refunded shortly.</p>",
            booking_id
        );
        self.send_email(to_email, &subject, &html_body).await
    }

    async fn send_lock_expiry_warning(
        &self,
        to_email: &str,
        booking_id: &str,
    ) -> Result<(), common::AppError> {
        let subject = "Action Required: Seat Lock Expiring Soon".to_string();
        let html_body = format!(
            "<h1>Seat Lock Expiring</h1>
             <p>Your seat lock for Booking ID <strong>{}</strong> will expire in less than 5 minutes.</p>
             <p>Please complete your payment immediately.</p>",
            booking_id
        );
        self.send_email(to_email, &subject, &html_body).await
    }
}
