use axum::{
    Json,
    extract::{Path, Query, State},
    http::HeaderMap,
};
use common::error::ApiResponse;
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::{decode_token, encode_access_token, encode_refresh_token};
use crate::dto::*;
use crate::state::AppState;

// ─── Auth helpers ─────────────────────────────────────────────────────────────

/// Extract user_id from JWT Bearer token (preferred) or X-User-Id header (fallback for tests).
fn get_user_id(headers: &HeaderMap, jwt_secret: &str) -> Result<String, common::AppError> {
    if let Some(auth_val) = headers.get("Authorization")
        && let Ok(auth_str) = auth_val.to_str()
        && let Some(token) = auth_str.strip_prefix("Bearer ")
    {
        let claims = decode_token(token, jwt_secret)?;
        return Ok(claims.sub);
    }
    // Fallback: X-User-Id header (keeps tests and legacy clients working)
    headers
        .get("X-User-Id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .ok_or(common::AppError::Unauthorized)
}

fn get_admin_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get("X-Admin-Token")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

/// Validate admin access: JWT with role=admin (preferred) or static env-var token (fallback for tests).
fn require_admin(token: Option<String>, jwt_secret: &str) -> Result<(), common::AppError> {
    let token = token.ok_or(common::AppError::Unauthorized)?;

    // Try JWT first
    if let Ok(claims) = decode_token(&token, jwt_secret) {
        if claims.role == "admin" {
            return Ok(());
        }
        return Err(common::AppError::Unauthorized);
    }

    // Fallback to static token (for tests / backward compat)
    let expected = std::env::var("ADMIN_TOKEN").unwrap_or_else(|_| "admin-secret".to_string());
    if token == expected {
        return Ok(());
    }

    Err(common::AppError::Unauthorized)
}

// ─── Auth handlers ────────────────────────────────────────────────────────────

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<
    (axum::http::StatusCode, Json<ApiResponse<AuthResponse>>),
    crate::impl_from_response::ApiError,
> {
    // Validate inputs
    let email = req.email.trim().to_lowercase();
    if email.is_empty() || !email.contains('@') {
        return Err(common::AppError::ValidationError("Invalid email".to_string()).into());
    }
    if req.password.len() < 8 {
        return Err(common::AppError::ValidationError(
            "Password must be at least 8 characters".to_string(),
        )
        .into());
    }
    let user_name = req.user_name.trim().to_string();
    if user_name.is_empty() {
        return Err(common::AppError::ValidationError("Name is required".to_string()).into());
    }

    // Check email uniqueness
    if state.user_repo.find_by_email(&email).await?.is_some() {
        return Err(common::AppError::EmailAlreadyExists.into());
    }

    // Hash password (blocking)
    let pw = req.password.clone();
    let hash = tokio::task::spawn_blocking(move || bcrypt::hash(&pw, 12))
        .await
        .map_err(|e| common::AppError::InternalError(e.to_string()))?
        .map_err(|e| common::AppError::InternalError(e.to_string()))?;

    let user_id = Uuid::new_v4().to_string();
    let user =
        domain::User::new_with_password(user_id.clone(), user_name.clone(), email.clone(), hash);
    state.user_repo.save(user).await?;

    let secret = &state.cfg.jwt.secret;
    let access_token = encode_access_token(
        &user_id,
        &email,
        &user_name,
        "user",
        secret,
        state.cfg.jwt.access_token_expiry_secs,
    )?;
    let refresh_token = encode_refresh_token(
        &user_id,
        &email,
        &user_name,
        "user",
        secret,
        state.cfg.jwt.refresh_token_expiry_secs,
    )?;

    Ok((
        axum::http::StatusCode::CREATED,
        Json(ApiResponse::ok(AuthResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: state.cfg.jwt.access_token_expiry_secs,
            user_id,
            email,
            user_name,
            role: "user".to_string(),
        })),
    ))
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<ApiResponse<AuthResponse>>, crate::impl_from_response::ApiError> {
    let email = req.email.trim().to_lowercase();

    let user = state
        .user_repo
        .find_by_email(&email)
        .await?
        .ok_or(common::AppError::InvalidCredentials)?;

    let hash = user
        .password_hash
        .clone()
        .ok_or(common::AppError::InvalidCredentials)?;

    let pw = req.password.clone();
    let valid = tokio::task::spawn_blocking(move || bcrypt::verify(&pw, &hash))
        .await
        .map_err(|e| common::AppError::InternalError(e.to_string()))?
        .map_err(|_| common::AppError::InvalidCredentials)?;

    if !valid {
        return Err(common::AppError::InvalidCredentials.into());
    }

    let secret = &state.cfg.jwt.secret;
    let role = user.role.to_string();
    let access_token = encode_access_token(
        &user.user_id,
        &user.email,
        &user.user_name,
        &role,
        secret,
        state.cfg.jwt.access_token_expiry_secs,
    )?;
    let refresh_token = encode_refresh_token(
        &user.user_id,
        &user.email,
        &user.user_name,
        &role,
        secret,
        state.cfg.jwt.refresh_token_expiry_secs,
    )?;

    Ok(Json(ApiResponse::ok(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: state.cfg.jwt.access_token_expiry_secs,
        user_id: user.user_id,
        email: user.email,
        user_name: user.user_name,
        role,
    })))
}

pub async fn refresh_token(
    State(state): State<AppState>,
    Json(req): Json<RefreshTokenRequest>,
) -> Result<Json<ApiResponse<AuthResponse>>, crate::impl_from_response::ApiError> {
    let claims = decode_token(&req.refresh_token, &state.cfg.jwt.secret)?;

    if claims.token_type != "refresh" {
        return Err(common::AppError::Unauthorized.into());
    }

    let secret = &state.cfg.jwt.secret;
    let access_token = encode_access_token(
        &claims.sub,
        &claims.email,
        &claims.user_name,
        &claims.role,
        secret,
        state.cfg.jwt.access_token_expiry_secs,
    )?;
    let new_refresh = encode_refresh_token(
        &claims.sub,
        &claims.email,
        &claims.user_name,
        &claims.role,
        secret,
        state.cfg.jwt.refresh_token_expiry_secs,
    )?;

    Ok(Json(ApiResponse::ok(AuthResponse {
        access_token,
        refresh_token: new_refresh,
        token_type: "Bearer".to_string(),
        expires_in: state.cfg.jwt.access_token_expiry_secs,
        user_id: claims.sub,
        email: claims.email,
        user_name: claims.user_name,
        role: claims.role,
    })))
}

pub async fn admin_login(
    State(state): State<AppState>,
    Json(req): Json<AdminLoginRequest>,
) -> Result<Json<ApiResponse<AuthResponse>>, crate::impl_from_response::ApiError> {
    let email = req.email.trim().to_lowercase();

    let user = state
        .user_repo
        .find_by_email(&email)
        .await?
        .ok_or(common::AppError::InvalidCredentials)?;

    if user.role != domain::UserRole::Admin {
        return Err(common::AppError::Unauthorized.into());
    }

    let hash = user
        .password_hash
        .clone()
        .ok_or(common::AppError::InvalidCredentials)?;

    let pw = req.password.clone();
    let valid = tokio::task::spawn_blocking(move || bcrypt::verify(&pw, &hash))
        .await
        .map_err(|e| common::AppError::InternalError(e.to_string()))?
        .map_err(|_| common::AppError::InvalidCredentials)?;

    if !valid {
        return Err(common::AppError::InvalidCredentials.into());
    }

    let secret = &state.cfg.jwt.secret;
    let access_token = encode_access_token(
        &user.user_id,
        &user.email,
        &user.user_name,
        "admin",
        secret,
        state.cfg.jwt.access_token_expiry_secs,
    )?;
    let refresh_token = encode_refresh_token(
        &user.user_id,
        &user.email,
        &user.user_name,
        "admin",
        secret,
        state.cfg.jwt.refresh_token_expiry_secs,
    )?;

    Ok(Json(ApiResponse::ok(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: state.cfg.jwt.access_token_expiry_secs,
        user_id: user.user_id,
        email: user.email,
        user_name: user.user_name,
        role: "admin".to_string(),
    })))
}

// ─── Health ────────────────────────────────────────────────────────────────────

static START_TIME: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();

pub async fn health() -> Json<ApiResponse<HealthResponse>> {
    let uptime = START_TIME
        .get_or_init(std::time::Instant::now)
        .elapsed()
        .as_secs();
    Json(ApiResponse::ok(HealthResponse {
        status: "ok".to_string(),
        uptime_seconds: uptime,
    }))
}

// ─── Show handlers ─────────────────────────────────────────────────────────────

pub async fn list_shows(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<ShowResponse>>>, crate::impl_from_response::ApiError> {
    let shows = state.show_svc.list_shows().await?;

    let responses: Vec<ShowResponse> = shows
        .into_iter()
        .map(|s| ShowResponse {
            show_id: s.show_id,
            show_name: s.show_name,
            theatre_name: s.theatre_name,
            screen_number: s.screen_number,
            start_time: s.start_time.timestamp(),
            end_time: s.end_time.timestamp(),
            price_per_seat: s.price_per_seat,
            total_seats: s.total_seats,
        })
        .collect();

    Ok(Json(ApiResponse::ok(responses)))
}

pub async fn get_show(
    State(state): State<AppState>,
    Path(show_id): Path<String>,
) -> Result<Json<ApiResponse<ShowResponse>>, crate::impl_from_response::ApiError> {
    let show = state
        .show_svc
        .get_show(&show_id)
        .await?
        .ok_or_else(|| common::AppError::ShowNotFound(show_id.clone()))?;

    Ok(Json(ApiResponse::ok(ShowResponse {
        show_id: show.show_id,
        show_name: show.show_name,
        theatre_name: show.theatre_name,
        screen_number: show.screen_number,
        start_time: show.start_time.timestamp(),
        end_time: show.end_time.timestamp(),
        price_per_seat: show.price_per_seat,
        total_seats: show.total_seats,
    })))
}

#[derive(Deserialize)]
pub struct SeatPageQuery {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_page() -> u32 {
    1
}
fn default_limit() -> u32 {
    100
}

pub async fn get_seat_layout(
    State(state): State<AppState>,
    Path(show_id): Path<String>,
    Query(query): Query<SeatPageQuery>,
) -> Result<Json<ApiResponse<SeatLayoutPageResponse>>, crate::impl_from_response::ApiError> {
    let show = state
        .show_svc
        .get_show(&show_id)
        .await?
        .ok_or_else(|| common::AppError::ShowNotFound(show_id.clone()))?;

    let all_seats = state.show_svc.get_seat_layout(&show_id).await?;

    let offset = (query.page.saturating_sub(1)) * query.limit;
    let seats: Vec<SeatResponse> = all_seats
        .into_iter()
        .skip(offset as usize)
        .take(query.limit as usize)
        .map(|s| {
            let price = s.effective_price(show.price_per_seat);
            SeatResponse {
                seat_id: s.seat_id,
                seat_number: s.seat_number,
                row_label: s.row_label,
                seat_type: s.seat_type.to_string(),
                status: s.status.to_string(),
                lock_expires_at: s.lock_expires_at.map(|dt| dt.timestamp()),
                price,
            }
        })
        .collect();

    Ok(Json(ApiResponse::ok(SeatLayoutPageResponse {
        show_id,
        seats,
        page: query.page,
        limit: query.limit,
    })))
}

pub async fn get_availability(
    State(state): State<AppState>,
    Path(show_id): Path<String>,
) -> Result<Json<ApiResponse<AvailabilityResponse>>, crate::impl_from_response::ApiError> {
    let avail = state.show_svc.get_show_availability(&show_id).await?;
    Ok(Json(ApiResponse::ok(AvailabilityResponse {
        show_id: avail.show_id,
        available: avail.available,
        locked: avail.locked,
        booked: avail.booked,
    })))
}

// ─── Seat lock handlers ────────────────────────────────────────────────────────

pub async fn lock_seats(
    State(state): State<AppState>,
    Path(show_id): Path<String>,
    headers: HeaderMap,
    Json(req): Json<LockSeatsRequest>,
) -> Result<
    (axum::http::StatusCode, Json<ApiResponse<LockSeatsResponse>>),
    crate::impl_from_response::ApiError,
> {
    let user_id = get_user_id(&headers, &state.cfg.jwt.secret)?;

    let rate_key = format!("lock:{}", user_id);
    let lock_limit = state.cfg.rate_limit.lock_requests_per_min;
    if state
        .rate_limiter
        .check(&rate_key, lock_limit)
        .await
        .is_err()
    {
        return Err(common::AppError::RateLimitExceeded.into());
    }

    let result = state
        .seat_locking_svc
        .lock_seats(&show_id, req.seat_ids, &user_id)
        .await?;

    let response = LockSeatsResponse {
        booking_id: result.booking_id,
        lock_id: result.lock_id,
        show_id: result.show_id,
        seat_ids: result.seat_ids,
        total_amount: result.total_amount,
        expires_at: result.expires_at.timestamp(),
        status: result.status,
    };

    Ok((
        axum::http::StatusCode::CREATED,
        Json(ApiResponse::ok(response)),
    ))
}

pub async fn extend_lock(
    State(state): State<AppState>,
    Path(booking_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<LockSeatsResponse>>, crate::impl_from_response::ApiError> {
    let user_id = get_user_id(&headers, &state.cfg.jwt.secret)?;

    let rate_key = format!("lock:{}", user_id);
    let lock_limit = state.cfg.rate_limit.lock_requests_per_min;
    if state
        .rate_limiter
        .check(&rate_key, lock_limit)
        .await
        .is_err()
    {
        return Err(common::AppError::RateLimitExceeded.into());
    }

    let result = state
        .seat_locking_svc
        .extend_lock(&booking_id, &user_id)
        .await?;

    Ok(Json(ApiResponse::ok(LockSeatsResponse {
        booking_id: result.booking_id,
        lock_id: result.lock_id,
        show_id: result.show_id,
        seat_ids: result.seat_ids,
        total_amount: result.total_amount,
        expires_at: result.expires_at.timestamp(),
        status: result.status,
    })))
}

pub async fn release_lock(
    State(state): State<AppState>,
    Path(booking_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<()>>, crate::impl_from_response::ApiError> {
    let user_id = get_user_id(&headers, &state.cfg.jwt.secret)?;

    state
        .seat_locking_svc
        .release_lock(&booking_id, &user_id)
        .await?;

    Ok(Json(ApiResponse::ok(())))
}

// ─── Booking handlers ─────────────────────────────────────────────────────────

pub async fn get_booking(
    State(state): State<AppState>,
    Path(booking_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<BookingResponse>>, crate::impl_from_response::ApiError> {
    let _user_id = get_user_id(&headers, &state.cfg.jwt.secret)?;

    let booking = state
        .booking_svc
        .get_booking(&booking_id)
        .await?
        .ok_or_else(|| common::AppError::BookingNotFound(booking_id.clone()))?;

    Ok(Json(ApiResponse::ok(BookingResponse {
        booking_id: booking.booking_id,
        user_id: booking.user_id,
        show_id: booking.show_id,
        seat_ids: booking.seat_ids,
        status: booking.status.to_string(),
        total_amount: booking.total_amount,
        payment_id: booking.payment_id,
        created_at: booking.created_at.timestamp(),
        expires_at: booking.expires_at.timestamp(),
        confirmed_at: booking.confirmed_at.map(|dt| dt.timestamp()),
        cancelled_at: booking.cancelled_at.map(|dt| dt.timestamp()),
    })))
}

pub async fn cancel_booking(
    State(state): State<AppState>,
    Path(booking_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<()>>, crate::impl_from_response::ApiError> {
    let user_id = get_user_id(&headers, &state.cfg.jwt.secret)?;

    state
        .booking_svc
        .cancel_booking(&booking_id, &user_id)
        .await?;

    Ok(Json(ApiResponse::ok(())))
}

pub async fn get_user_bookings(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<Vec<BookingResponse>>>, crate::impl_from_response::ApiError> {
    let _ = get_user_id(&headers, &state.cfg.jwt.secret)?;

    let bookings = state.booking_svc.get_user_bookings(&user_id).await?;

    let responses: Vec<BookingResponse> = bookings
        .into_iter()
        .map(|b| BookingResponse {
            booking_id: b.booking_id,
            user_id: b.user_id,
            show_id: b.show_id,
            seat_ids: b.seat_ids,
            status: b.status.to_string(),
            total_amount: b.total_amount,
            payment_id: b.payment_id,
            created_at: b.created_at.timestamp(),
            expires_at: b.expires_at.timestamp(),
            confirmed_at: b.confirmed_at.map(|dt| dt.timestamp()),
            cancelled_at: b.cancelled_at.map(|dt| dt.timestamp()),
        })
        .collect();

    Ok(Json(ApiResponse::ok(responses)))
}

// ─── Payment handlers ─────────────────────────────────────────────────────────

pub async fn initiate_payment(
    State(state): State<AppState>,
    Path(booking_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<PaymentInitiatedResponse>>, crate::impl_from_response::ApiError> {
    let user_id = get_user_id(&headers, &state.cfg.jwt.secret)?;
    let idempotency_key = headers
        .get("Idempotency-Key")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let rate_key = format!("payment:{}", user_id);
    let payment_limit = state.cfg.rate_limit.payment_requests_per_min;
    if state
        .rate_limiter
        .check(&rate_key, payment_limit)
        .await
        .is_err()
    {
        return Err(common::AppError::RateLimitExceeded.into());
    }

    let result = state
        .payment_svc
        .initiate_payment(&booking_id, &user_id, idempotency_key)
        .await?;

    Ok(Json(ApiResponse::ok(PaymentInitiatedResponse {
        payment_id: result.payment_id,
        payment_intent_id: result.payment_intent_id,
        amount: result.amount,
        gateway_name: result.gateway_name,
        status: "pending".to_string(),
        client_secret: result.client_secret,
    })))
}

#[derive(Deserialize)]
pub struct PaymentCallbackQuery {
    pub status: String,
}

pub async fn payment_callback(
    State(state): State<AppState>,
    Path(payment_intent_id): Path<String>,
    Query(query): Query<PaymentCallbackQuery>,
) -> Result<Json<ApiResponse<()>>, crate::impl_from_response::ApiError> {
    state
        .payment_svc
        .payment_callback(&payment_intent_id, &query.status, None)
        .await?;

    Ok(Json(ApiResponse::ok(())))
}

pub async fn stripe_webhook(
    State(state): State<AppState>,
    body: axum::body::Bytes,
) -> Result<Json<ApiResponse<()>>, crate::impl_from_response::ApiError> {
    // Parse the payload as JSON
    let payload: serde_json::Value = serde_json::from_slice(&body)
        .map_err(|_| common::AppError::ValidationError("Invalid JSON payload".to_string()))?;

    // TODO: Verify Stripe-Signature header using stripe_webhook_secret

    if let Some(type_str) = payload.get("type").and_then(|t| t.as_str())
        && (type_str == "payment_intent.succeeded" || type_str == "payment_intent.payment_failed")
    {
        let status = if type_str == "payment_intent.succeeded" {
            "SUCCESS"
        } else {
            "FAILED"
        };

        if let Some(data) = payload.get("data")
            && let Some(object) = data.get("object")
            && let Some(payment_intent_id) = object.get("id").and_then(|i| i.as_str())
        {
            // Call the existing payment callback logic
            let _ = state
                .payment_svc
                .payment_callback(payment_intent_id, status, None)
                .await;
        }
    }

    Ok(Json(ApiResponse::ok(())))
}

pub async fn get_payment(
    State(state): State<AppState>,
    Path(payment_id): Path<String>,
) -> Result<Json<ApiResponse<PaymentResponse>>, crate::impl_from_response::ApiError> {
    let payment = state
        .payment_svc
        .get_payment(&payment_id)
        .await?
        .ok_or_else(|| common::AppError::PaymentNotFound(payment_id.clone()))?;

    Ok(Json(ApiResponse::ok(PaymentResponse {
        payment_id: payment.payment_id,
        booking_id: payment.booking_id,
        amount: payment.amount,
        currency: payment.currency,
        status: payment.status.to_string(),
        gateway_name: payment.gateway_name,
        created_at: payment.created_at.timestamp(),
    })))
}

// ─── Mock gateway ─────────────────────────────────────────────────────────────

pub async fn mock_gateway_pay(
    State(state): State<AppState>,
    Json(req): Json<service::payment::MockPaymentRequest>,
) -> Result<
    Json<ApiResponse<service::payment::MockPaymentResponse>>,
    crate::impl_from_response::ApiError,
> {
    let payment_intent_id = req.payment_intent_id.clone();
    let response = state.payment_svc.mock_gateway_pay(req).await?;

    state
        .payment_svc
        .payment_callback(
            &payment_intent_id,
            &response.status,
            Some(&serde_json::to_string(&response).unwrap_or_default()),
        )
        .await?;

    Ok(Json(ApiResponse::ok(response)))
}

// ─── Queue handlers ────────────────────────────────────────────────────────────

pub async fn leave_queue(
    State(state): State<AppState>,
    Path(queue_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<()>>, crate::impl_from_response::ApiError> {
    let user_id = get_user_id(&headers, &state.cfg.jwt.secret)?;

    state.queue_svc.leave_queue(&queue_id, &user_id).await?;

    Ok(Json(ApiResponse::ok(())))
}

pub async fn join_queue(
    State(state): State<AppState>,
    Path(show_id): Path<String>,
    headers: HeaderMap,
    Json(req): Json<JoinQueueRequest>,
) -> Result<
    (axum::http::StatusCode, Json<ApiResponse<QueueJoinResponse>>),
    crate::impl_from_response::ApiError,
> {
    let user_id = get_user_id(&headers, &state.cfg.jwt.secret)?;

    let result = state
        .queue_svc
        .join_queue(&show_id, &user_id, req.seat_ids)
        .await?;

    Ok((
        axum::http::StatusCode::ACCEPTED,
        Json(ApiResponse::ok(QueueJoinResponse {
            queue_id: result.queue_id,
            show_id: result.show_id,
            position: result.position,
            status: result.status,
        })),
    ))
}

pub async fn get_queue_status(
    State(state): State<AppState>,
    Path(queue_id): Path<String>,
) -> Result<Json<ApiResponse<QueueStatusResponse>>, crate::impl_from_response::ApiError> {
    let result = state
        .queue_svc
        .get_queue_status(&queue_id)
        .await?
        .ok_or_else(|| common::AppError::QueueEntryNotFound(queue_id.clone()))?;

    Ok(Json(ApiResponse::ok(QueueStatusResponse {
        queue_id: result.queue_id,
        status: result.status,
        position: result.position,
        booking_id: result.booking_id,
        lock_id: result.lock_id,
        conflict_seats: result.conflict_seats,
    })))
}

// ─── Admin handlers ────────────────────────────────────────────────────────────

pub async fn create_show(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateShowRequestDto>,
) -> Result<
    (axum::http::StatusCode, Json<ApiResponse<ShowResponse>>),
    crate::impl_from_response::ApiError,
> {
    require_admin(get_admin_token(&headers), &state.cfg.jwt.secret)?;

    let svc_req = service::show::CreateShowRequest {
        show_name: req.show_name.clone(),
        theatre_name: req.theatre_name.clone(),
        screen_number: req.screen_number,
        start_time: chrono::DateTime::from_timestamp(req.start_time, 0)
            .ok_or_else(|| common::AppError::ValidationError("invalid start_time".to_string()))?,
        end_time: chrono::DateTime::from_timestamp(req.end_time, 0)
            .ok_or_else(|| common::AppError::ValidationError("invalid end_time".to_string()))?,
        price_per_seat: req.price_per_seat,
        seat_layout: service::show::SeatLayoutRequest {
            rows: req
                .seat_layout
                .rows
                .into_iter()
                .map(|r| service::show::RowConfig {
                    row: r.row,
                    seats: r.seats,
                    seat_type: r.seat_type,
                })
                .collect(),
        },
    };

    let (show, seats) = state.show_svc.create_show(svc_req).await?;

    tracing::info!(show_id = %show.show_id, seat_count = seats.len(), "admin created show");

    Ok((
        axum::http::StatusCode::CREATED,
        Json(ApiResponse::ok(ShowResponse {
            show_id: show.show_id,
            show_name: show.show_name,
            theatre_name: show.theatre_name,
            screen_number: show.screen_number,
            start_time: show.start_time.timestamp(),
            end_time: show.end_time.timestamp(),
            price_per_seat: show.price_per_seat,
            total_seats: show.total_seats,
        })),
    ))
}

pub async fn cancel_show(
    State(state): State<AppState>,
    Path(show_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<()>>, crate::impl_from_response::ApiError> {
    require_admin(get_admin_token(&headers), &state.cfg.jwt.secret)?;

    state.show_svc.cancel_show(&show_id).await?;

    let bookings = state.booking_svc.get_show_bookings(&show_id).await?;
    for booking in bookings {
        if matches!(
            booking.status,
            domain::BookingStatus::Success | domain::BookingStatus::SuccessPartial
        ) && let Some(payment_id) = &booking.payment_id
        {
            let _ = state.payment_svc.refund_payment(payment_id).await;
        }
        let _ = state
            .booking_svc
            .cancel_booking(&booking.booking_id, &booking.user_id)
            .await;
    }

    Ok(Json(ApiResponse::ok(())))
}

pub async fn admin_list_bookings(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<Vec<BookingResponse>>>, crate::impl_from_response::ApiError> {
    require_admin(get_admin_token(&headers), &state.cfg.jwt.secret)?;

    let bookings = state.booking_svc.get_all_bookings().await?;

    let responses: Vec<BookingResponse> = bookings
        .into_iter()
        .map(|b| BookingResponse {
            booking_id: b.booking_id,
            user_id: b.user_id,
            show_id: b.show_id,
            seat_ids: b.seat_ids,
            status: b.status.to_string(),
            total_amount: b.total_amount,
            payment_id: b.payment_id,
            created_at: b.created_at.timestamp(),
            expires_at: b.expires_at.timestamp(),
            confirmed_at: b.confirmed_at.map(|dt| dt.timestamp()),
            cancelled_at: b.cancelled_at.map(|dt| dt.timestamp()),
        })
        .collect();

    Ok(Json(ApiResponse::ok(responses)))
}

pub async fn admin_show_analytics(
    State(state): State<AppState>,
    Path(show_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<ShowAnalyticsResponse>>, crate::impl_from_response::ApiError> {
    require_admin(get_admin_token(&headers), &state.cfg.jwt.secret)?;

    let analytics = state.show_svc.get_show_analytics(&show_id).await?;

    Ok(Json(ApiResponse::ok(ShowAnalyticsResponse {
        show_id: analytics.show_id,
        show_name: analytics.show_name,
        total_seats: analytics.total_seats,
        available_seats: analytics.available_seats,
        locked_seats: analytics.locked_seats,
        booked_seats: analytics.booked_seats,
        occupancy_rate: analytics.occupancy_rate,
        revenue: analytics.revenue,
    })))
}

pub async fn admin_override_seat(
    State(state): State<AppState>,
    Path((_show_id, seat_id)): Path<(String, String)>,
    headers: HeaderMap,
    Json(req): Json<SeatOverrideRequest>,
) -> Result<Json<ApiResponse<SeatOverrideResponse>>, crate::impl_from_response::ApiError> {
    require_admin(get_admin_token(&headers), &state.cfg.jwt.secret)?;

    let reason = if req.reason.is_empty() {
        "admin override".to_string()
    } else {
        req.reason
    };

    let result = state
        .seat_locking_svc
        .admin_override_seat(&seat_id, &reason)
        .await?;

    Ok(Json(ApiResponse::ok(SeatOverrideResponse {
        seat_id: result.seat_id,
        seat_number: result.seat_number,
        previous_status: result.previous_status,
        new_status: result.new_status,
        released_lock_id: result.released_lock_id,
    })))
}

pub async fn refund_payment(
    State(state): State<AppState>,
    Path(payment_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<()>>, crate::impl_from_response::ApiError> {
    require_admin(get_admin_token(&headers), &state.cfg.jwt.secret)?;

    state.payment_svc.refund_payment(&payment_id).await?;

    Ok(Json(ApiResponse::ok(())))
}
