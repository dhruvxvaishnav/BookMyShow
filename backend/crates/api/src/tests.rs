use axum::http::{HeaderName, HeaderValue};
use common::AppConfig;
use repository::{
    BookingRepository, CompensationLogRepository, PaymentRepository, QueueRepository,
    SeatLockRepository, SeatRepository, ShowRepository, UserRepository,
};
use repository_inmemory::{
    InMemoryBookingRepository, InMemoryCompensationLogRepository, InMemoryPaymentRepository,
    InMemoryQueueRepository, InMemorySeatLockRepository, InMemorySeatRepository,
    InMemoryShowRepository, InMemoryUserRepository,
};
use service::{
    BookingService, PaymentService, QueueService, SeatLockingService, ShowService,
    booking_service::BookingServiceTrait, payment_service::PaymentServiceTrait,
};
use std::sync::Arc;

/// Creates a fresh AppState with fresh in-memory repos.
async fn make_state() -> crate::AppState {
    let mut cfg = AppConfig::default();
    cfg.payment.mock_gateway_failure_rate = 0.0;

    let user_repo: Arc<dyn UserRepository> = Arc::new({
        let repo = InMemoryUserRepository::new();
        repo.seed_test_user().await;
        repo
    });
    let show_repo: Arc<dyn ShowRepository> = Arc::new(InMemoryShowRepository::new());
    let seat_repo: Arc<dyn SeatRepository> = Arc::new(InMemorySeatRepository::new());
    let booking_repo: Arc<dyn BookingRepository> = Arc::new(InMemoryBookingRepository::new());
    let payment_repo: Arc<dyn PaymentRepository> = Arc::new(InMemoryPaymentRepository::new());
    let seat_lock_repo: Arc<dyn SeatLockRepository> = Arc::new(InMemorySeatLockRepository::new());
    let queue_repo: Arc<dyn QueueRepository> = Arc::new(InMemoryQueueRepository::new());
    let compensation_log_repo: Arc<dyn CompensationLogRepository> =
        Arc::new(InMemoryCompensationLogRepository::new());
    let rate_limiter = crate::rate_limiter::RateLimiter::new();

    let seat_locking_svc = Arc::new(SeatLockingService::new(
        Arc::clone(&show_repo),
        Arc::clone(&seat_repo),
        Arc::clone(&booking_repo),
        Arc::clone(&seat_lock_repo),
        Arc::clone(&user_repo),
        cfg.clone(),
    ));
    let booking_svc = Arc::new(BookingService::new(
        Arc::clone(&booking_repo),
        Arc::clone(&seat_repo),
        Arc::clone(&payment_repo),
        Arc::clone(&compensation_log_repo),
        cfg.clone(),
    ));
    let payment_svc = Arc::new(PaymentService::new(
        Arc::clone(&payment_repo),
        Arc::clone(&booking_repo),
        Arc::clone(&booking_svc) as Arc<dyn BookingServiceTrait>,
        cfg.clone(),
    ));
    let show_svc = Arc::new(ShowService::new(
        Arc::clone(&show_repo),
        Arc::clone(&seat_repo),
        Arc::clone(&booking_repo),
        cfg.clone(),
    ));
    let queue_svc = Arc::new(QueueService::new(
        Arc::clone(&queue_repo),
        Arc::clone(&seat_repo),
        Arc::clone(&seat_lock_repo),
        Arc::clone(&seat_locking_svc),
        cfg.clone(),
    ));

    crate::AppState::new(
        seat_locking_svc,
        booking_svc as Arc<dyn BookingServiceTrait>,
        payment_svc as Arc<dyn PaymentServiceTrait>,
        show_svc,
        queue_svc,
        rate_limiter,
        cfg,
    )
}

fn user_hdr() -> (HeaderName, HeaderValue) {
    (
        HeaderName::from_static("x-user-id"),
        HeaderValue::from_static("user-001"),
    )
}

fn admin_hdr() -> (HeaderName, HeaderValue) {
    (
        HeaderName::from_static("x-admin-token"),
        HeaderValue::from_static("admin-secret"),
    )
}

/// Helper: create a show and return its show_id.
async fn create_show(client: &axum_test::TestServer, name: &str) -> String {
    let resp = client
        .post("/admin/shows")
        .add_header(&admin_hdr().0, &admin_hdr().1)
        .json(&serde_json::json!({
            "show_name": name,
            "theatre_name": "Test Theatre",
            "screen_number": 1,
            "start_time": 1748481600,
            "end_time": 1748492400,
            "price_per_seat": 250.0,
            "seat_layout": { "rows": [{ "row": "A", "seats": 5, "seat_type": "standard" }] }
        }))
        .await;
    let body: serde_json::Value = serde_json::from_str(resp.text().as_str()).unwrap();
    body["data"]["show_id"].as_str().unwrap().to_string()
}

/// Helper: get N available seat IDs for a show.
async fn get_seat_ids(client: &axum_test::TestServer, show_id: &str, count: usize) -> Vec<String> {
    let resp = client.get(&format!("/shows/{}/seats", show_id)).await;
    let body: serde_json::Value = serde_json::from_str(resp.text().as_str()).unwrap();
    let seats = body["data"]["seats"].as_array().unwrap();
    seats
        .iter()
        .take(count)
        .map(|s| s["seat_id"].as_str().unwrap().to_string())
        .collect()
}

#[tokio::test]
async fn test_health_endpoint() {
    let app = crate::create_router(make_state().await);
    let client = axum_test::TestServer::new(app).unwrap();
    let resp = client.get("/health").await;
    resp.assert_status(axum::http::StatusCode::OK);
    let body: serde_json::Value = serde_json::from_str(resp.text().as_str()).unwrap();
    assert!(body["success"].as_bool().unwrap());
}

#[tokio::test]
async fn test_lock_and_confirm_booking_flow() {
    let app = crate::create_router(make_state().await);
    let client = axum_test::TestServer::new(app).unwrap();

    let show_id = create_show(&client, "Test Movie").await;
    let seat_ids = get_seat_ids(&client, &show_id, 2).await;

    // Lock seats
    let resp = client
        .post(&format!("/shows/{show_id}/seats/lock"))
        .add_header(&user_hdr().0, &user_hdr().1)
        .json(&serde_json::json!({ "seat_ids": seat_ids }))
        .await;
    resp.assert_status(axum::http::StatusCode::CREATED);
    let body: serde_json::Value = serde_json::from_str(resp.text().as_str()).unwrap();
    let booking_id = body["data"]["booking_id"].as_str().unwrap().to_string();

    // Initiate payment
    let resp = client
        .post(&format!("/bookings/{booking_id}/payment/initiate"))
        .add_header(&user_hdr().0, &user_hdr().1)
        .await;
    resp.assert_status(axum::http::StatusCode::OK);
    let body: serde_json::Value = serde_json::from_str(resp.text().as_str()).unwrap();
    let payment_intent_id = body["data"]["payment_intent_id"]
        .as_str()
        .unwrap()
        .to_string();
    let amount = body["data"]["amount"].as_f64().unwrap();

    let resp_gw = client
        .post("/mock-gateway/pay")
        .json(&serde_json::json!({
            "payment_intent_id": payment_intent_id,
            "amount": amount,
            "simulate_failure": false,
            "simulate_delay_ms": 0
        }))
        .await;
    resp_gw.assert_status(axum::http::StatusCode::OK);

    // Verify booking confirmed
    let resp = client
        .get(&format!("/bookings/{booking_id}"))
        .add_header(&user_hdr().0, &user_hdr().1)
        .await;
    resp.assert_status(axum::http::StatusCode::OK);
    let body: serde_json::Value = serde_json::from_str(resp.text().as_str()).unwrap();
    assert_eq!(
        body["data"]["status"].as_str().unwrap().to_lowercase(),
        "success"
    );
    assert!(!body["data"]["confirmed_at"].is_null());
}

#[tokio::test]
async fn test_lock_then_cancel() {
    let app = crate::create_router(make_state().await);
    let client = axum_test::TestServer::new(app).unwrap();

    let show_id = create_show(&client, "Cancel Test").await;
    let seat_ids = get_seat_ids(&client, &show_id, 1).await;

    let resp = client
        .post(&format!("/shows/{show_id}/seats/lock"))
        .add_header(&user_hdr().0, &user_hdr().1)
        .json(&serde_json::json!({ "seat_ids": seat_ids }))
        .await;
    resp.assert_status(axum::http::StatusCode::CREATED);
    let body: serde_json::Value = serde_json::from_str(resp.text().as_str()).unwrap();
    let booking_id = body["data"]["booking_id"].as_str().unwrap().to_string();

    let resp = client
        .post(&format!("/bookings/{booking_id}/cancel"))
        .add_header(&user_hdr().0, &user_hdr().1)
        .await;
    resp.assert_status(axum::http::StatusCode::OK);

    let resp = client
        .get(&format!("/bookings/{booking_id}"))
        .add_header(&user_hdr().0, &user_hdr().1)
        .await;
    resp.assert_status(axum::http::StatusCode::OK);
    let body: serde_json::Value = serde_json::from_str(resp.text().as_str()).unwrap();
    assert_eq!(
        body["data"]["status"].as_str().unwrap().to_lowercase(),
        "cancelled"
    );
}

#[tokio::test]
async fn test_double_lock_rejected() {
    let app = crate::create_router(make_state().await);
    let client = axum_test::TestServer::new(app).unwrap();

    let show_id = create_show(&client, "Conflict Test").await;
    let seat_ids = get_seat_ids(&client, &show_id, 1).await;

    client
        .post(&format!("/shows/{show_id}/seats/lock"))
        .add_header(&user_hdr().0, &user_hdr().1)
        .json(&serde_json::json!({ "seat_ids": seat_ids.clone() }))
        .await;

    let resp = client
        .post(&format!("/shows/{show_id}/seats/lock"))
        .add_header(&user_hdr().0, &user_hdr().1)
        .json(&serde_json::json!({ "seat_ids": seat_ids }))
        .await;
    resp.assert_status(axum::http::StatusCode::CONFLICT);
    let body: serde_json::Value = serde_json::from_str(resp.text().as_str()).unwrap();
    assert!(!body["success"].as_bool().unwrap());
}

#[tokio::test]
async fn test_queue_join_and_status() {
    let app = crate::create_router(make_state().await);
    let client = axum_test::TestServer::new(app).unwrap();

    let show_id = create_show(&client, "Queue Test").await;
    let seat_ids = get_seat_ids(&client, &show_id, 1).await;

    let resp = client
        .post(&format!("/shows/{show_id}/queue/join"))
        .add_header(&user_hdr().0, &user_hdr().1)
        .json(&serde_json::json!({ "seat_ids": seat_ids }))
        .await;
    resp.assert_status(axum::http::StatusCode::ACCEPTED);
    let body: serde_json::Value = serde_json::from_str(resp.text().as_str()).unwrap();
    let queue_id = body["data"]["queue_id"].as_str().unwrap().to_string();
    assert_eq!(body["data"]["position"], 1);

    let resp = client.get(&format!("/queue/{queue_id}/status")).await;
    resp.assert_status(axum::http::StatusCode::OK);
    let body: serde_json::Value = serde_json::from_str(resp.text().as_str()).unwrap();
    assert_eq!(body["data"]["queue_id"], queue_id);
}

#[tokio::test]
async fn test_availability_endpoint() {
    let app = crate::create_router(make_state().await);
    let client = axum_test::TestServer::new(app).unwrap();

    let show_id = create_show(&client, "Avail Test").await;

    let resp = client.get(&format!("/shows/{show_id}/availability")).await;
    resp.assert_status(axum::http::StatusCode::OK);
    let body: serde_json::Value = serde_json::from_str(resp.text().as_str()).unwrap();
    assert_eq!(body["data"]["available"], 5);
    assert_eq!(body["data"]["locked"], 0);
    assert_eq!(body["data"]["booked"], 0);
}

#[tokio::test]
async fn test_payment_failure_releases_seats() {
    let app = crate::create_router(make_state().await);
    let client = axum_test::TestServer::new(app).unwrap();

    let show_id = create_show(&client, "Pay Fail Test").await;
    let seat_ids = get_seat_ids(&client, &show_id, 2).await;

    let resp = client
        .post(&format!("/shows/{show_id}/seats/lock"))
        .add_header(&user_hdr().0, &user_hdr().1)
        .json(&serde_json::json!({ "seat_ids": seat_ids }))
        .await;
    resp.assert_status(axum::http::StatusCode::CREATED);
    let body: serde_json::Value = serde_json::from_str(resp.text().as_str()).unwrap();
    let booking_id = body["data"]["booking_id"].as_str().unwrap().to_string();

    let resp = client
        .post(&format!("/bookings/{booking_id}/payment/initiate"))
        .add_header(&user_hdr().0, &user_hdr().1)
        .await;
    resp.assert_status(axum::http::StatusCode::OK);
    let body: serde_json::Value = serde_json::from_str(resp.text().as_str()).unwrap();
    let payment_intent_id = body["data"]["payment_intent_id"]
        .as_str()
        .unwrap()
        .to_string();
    let amount = body["data"]["amount"].as_f64().unwrap();

    let _resp_gw = client
        .post("/mock-gateway/pay")
        .json(&serde_json::json!({
            "payment_intent_id": payment_intent_id,
            "amount": amount,
            "simulate_failure": true,
            "simulate_delay_ms": 0
        }))
        .await;

    let resp = client
        .get(&format!("/bookings/{booking_id}"))
        .add_header(&user_hdr().0, &user_hdr().1)
        .await;
    resp.assert_status(axum::http::StatusCode::OK);
    let body: serde_json::Value = serde_json::from_str(resp.text().as_str()).unwrap();
    assert_eq!(
        body["data"]["status"].as_str().unwrap().to_lowercase(),
        "cancelled"
    );
}

#[tokio::test]
async fn test_payment_idempotency() {
    let app = crate::create_router(make_state().await);
    let client = axum_test::TestServer::new(app).unwrap();

    let show_id = create_show(&client, "Idempotency Test").await;
    let seat_ids = get_seat_ids(&client, &show_id, 1).await;

    let resp = client
        .post(&format!("/shows/{show_id}/seats/lock"))
        .add_header(&user_hdr().0, &user_hdr().1)
        .json(&serde_json::json!({ "seat_ids": seat_ids }))
        .await;
    resp.assert_status(axum::http::StatusCode::CREATED);
    let body: serde_json::Value = serde_json::from_str(resp.text().as_str()).unwrap();
    let booking_id = body["data"]["booking_id"].as_str().unwrap().to_string();

    let key = "idemp-key-123";

    // Request 1
    let resp1 = client
        .post(&format!("/bookings/{booking_id}/payment/initiate"))
        .add_header(&user_hdr().0, &user_hdr().1)
        .add_header(
            axum::http::HeaderName::from_static("idempotency-key"),
            axum::http::HeaderValue::from_static(key),
        )
        .await;
    resp1.assert_status(axum::http::StatusCode::OK);
    let b1: serde_json::Value = serde_json::from_str(resp1.text().as_str()).unwrap();
    let pid1 = b1["data"]["payment_id"].as_str().unwrap().to_string();

    // Request 2 (same key)
    let resp2 = client
        .post(&format!("/bookings/{booking_id}/payment/initiate"))
        .add_header(&user_hdr().0, &user_hdr().1)
        .add_header(
            axum::http::HeaderName::from_static("idempotency-key"),
            axum::http::HeaderValue::from_static(key),
        )
        .await;
    resp2.assert_status(axum::http::StatusCode::OK);
    let b2: serde_json::Value = serde_json::from_str(resp2.text().as_str()).unwrap();
    let pid2 = b2["data"]["payment_id"].as_str().unwrap().to_string();

    // Must be same payment ID
    assert_eq!(pid1, pid2);
}

#[tokio::test]
async fn test_admin_cancel_show() {
    let app = crate::create_router(make_state().await);
    let client = axum_test::TestServer::new(app).unwrap();

    let show_id = create_show(&client, "Cancel Show Test").await;

    // Delete show
    let resp = client
        .delete(&format!("/admin/shows/{show_id}"))
        .add_header(&admin_hdr().0, &admin_hdr().1)
        .await;
    resp.assert_status(axum::http::StatusCode::OK);

    // Verify show is gone
    let resp = client.get(&format!("/shows/{show_id}")).await;
    resp.assert_status(axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_admin_refund_payment() {
    let app = crate::create_router(make_state().await);
    let client = axum_test::TestServer::new(app).unwrap();

    let show_id = create_show(&client, "Refund Test").await;
    let seat_ids = get_seat_ids(&client, &show_id, 2).await;

    let resp = client
        .post(&format!("/shows/{show_id}/seats/lock"))
        .add_header(&user_hdr().0, &user_hdr().1)
        .json(&serde_json::json!({ "seat_ids": seat_ids }))
        .await;
    let body: serde_json::Value = serde_json::from_str(resp.text().as_str()).unwrap();
    let booking_id = body["data"]["booking_id"].as_str().unwrap().to_string();

    let resp = client
        .post(&format!("/bookings/{booking_id}/payment/initiate"))
        .add_header(&user_hdr().0, &user_hdr().1)
        .await;
    let body: serde_json::Value = serde_json::from_str(resp.text().as_str()).unwrap();
    let payment_id = body["data"]["payment_id"].as_str().unwrap().to_string();
    let payment_intent_id = body["data"]["payment_intent_id"]
        .as_str()
        .unwrap()
        .to_string();
    let amount = body["data"]["amount"].as_f64().unwrap();

    // Pay
    let resp = client
        .post("/mock-gateway/pay")
        .json(&serde_json::json!({
            "payment_intent_id": payment_intent_id,
            "amount": amount,
            "simulate_failure": false,
            "simulate_delay_ms": 0
        }))
        .await;
    resp.assert_status(axum::http::StatusCode::OK);

    // Refund
    let resp = client
        .post(&format!("/payments/{payment_id}/refund"))
        .add_header(&admin_hdr().0, &admin_hdr().1)
        .await;
    resp.assert_status(axum::http::StatusCode::OK);

    // Verify refund
    let resp = client
        .get(&format!("/payments/{payment_id}"))
        .add_header(&user_hdr().0, &user_hdr().1)
        .await;
    resp.assert_status(axum::http::StatusCode::OK);
    let body: serde_json::Value = serde_json::from_str(resp.text().as_str()).unwrap();
    assert_eq!(
        body["data"]["status"].as_str().unwrap().to_lowercase(),
        "refunded"
    );
}
