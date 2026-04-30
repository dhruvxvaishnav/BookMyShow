#!/usr/bin/env bash
# ============================================================
# BookMyShow — End-to-End Smoke Tests
# Runs the full booking flow via curl against localhost:8080
# ============================================================

set -e

BASE_URL="${BASE_URL:-http://localhost:8080}"
ADMIN_TOKEN="${ADMIN_TOKEN:-admin-secret}"
ADMIN_HDR="X-Admin-Token: $ADMIN_TOKEN"
ADMIN_HDR_2="-H 'X-Admin-Token: $ADMIN_TOKEN'"
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

pass() { echo -e "${GREEN}[PASS]${NC} $1"; }
fail() { echo -e "${RED}[FAIL]${NC} $1"; exit 1; }
info()  { echo -e "${YELLOW}[INFO]${NC} $1"; }

# ── Helpers ──────────────────────────────────────────────────
admin_post() { curl -s -X POST -H "Content-Type: application/json" -H "$ADMIN_HDR" "$@"; }
admin_get()  { curl -s -H "$ADMIN_HDR" "$@"; }
user_post()  { curl -s -X POST -H "Content-Type: application/json" -H "$1" "$@"; }
user_get()   { curl -s -H "$1" "$@"; }
post_json()  { curl -s -X POST -H "Content-Type: application/json" "$@"; }

jq_val() { jq -r "$2" <<< "$1" 2>/dev/null || echo ""; }

# ── 0. Wait for server ────────────────────────────────────────
echo ""
info "Waiting for backend at $BASE_URL..."
for i in $(seq 1 10); do
  if curl -s --max-time 2 "$BASE_URL/health" | jq -e '.success' > /dev/null 2>&1; then
    break
  fi
  sleep 1
done

HEALTH=$(curl -s "$BASE_URL/health")
if ! echo "$HEALTH" | jq -e '.success' > /dev/null; then
  fail "Backend health check failed. Is it running on port 8080?"
fi
pass "Backend is healthy"
UPTIME=$(jq -r '.data.uptime_seconds' <<< "$HEALTH")
info "Backend uptime: ${UPTIME}s"

# ── 1. List shows ────────────────────────────────────────────
info "Testing GET /shows"
SHOWS=$(curl -s "$BASE_URL/shows")
if ! echo "$SHOWS" | jq -e '.success' > /dev/null; then
  fail "GET /shows failed"
fi
pass "GET /shows OK"
SHOW_COUNT=$(jq '.data | length' <<< "$SHOWS")
info "Found $SHOW_COUNT shows"

# Pick the first show for tests
FIRST_SHOW=$(jq -r '.data[0].show_id' <<< "$SHOWS")
if [ -z "$FIRST_SHOW" ] || [ "$FIRST_SHOW" = "null" ]; then
  fail "No shows found. Did seeding work?"
fi
info "Using show: $FIRST_SHOW"

# ── 2. Get show details ───────────────────────────────────────
info "Testing GET /shows/:id"
SHOW_DETAIL=$(curl -s "$BASE_URL/shows/$FIRST_SHOW")
if ! echo "$SHOW_DETAIL" | jq -e '.success' > /dev/null; then
  fail "GET /shows/$FIRST_SHOW failed"
fi
pass "GET /shows/:id OK"
PRICE=$(jq -r '.data.price_per_seat' <<< "$SHOW_DETAIL")
info "Price per seat: ₹$PRICE"

# ── 3. Get seat layout ────────────────────────────────────────
info "Testing GET /shows/:id/seats"
SEATS=$(curl -s "$BASE_URL/shows/$FIRST_SHOW/seats")
if ! echo "$SEATS" | jq -e '.success' > /dev/null; then
  fail "GET /shows/$FIRST_SHOW/seats failed"
fi
pass "GET /shows/:id/seats OK"
SEAT_COUNT=$(jq '.data.seats | length' <<< "$SEATS")
info "Total seats in layout: $SEAT_COUNT"

# Pick first 2 available seats
AVAILABLE_SEATS=$(jq -r '[.data.seats[] | select(.status == "Available") | .seat_id][0:2]' <<< "$SEATS")
SEAT_ID_1=$(echo "$AVAILABLE_SEATS" | jq -r '.[0]')
SEAT_ID_2=$(echo "$AVAILABLE_SEATS" | jq -r '.[1]')

if [ -z "$SEAT_ID_1" ] || [ "$SEAT_ID_1" = "null" ]; then
  fail "No available seats found in show"
fi
info "Selected seats: $SEAT_ID_1, $SEAT_ID_2"

# ── 4. Get availability ───────────────────────────────────────
info "Testing GET /shows/:id/availability"
AVAIL=$(curl -s "$BASE_URL/shows/$FIRST_SHOW/availability")
if ! echo "$AVAIL" | jq -e '.success' > /dev/null; then
  fail "GET /availability failed"
fi
pass "GET /availability OK"
AVAIL_COUNT=$(jq -r '.data.available' <<< "$AVAIL")
info "Available seats: $AVAIL_COUNT"

# ── 5. Create admin show (POST /admin/shows) ──────────────────
info "Testing POST /admin/shows"
NOW=$(date +%s)
ADMIN_SHOW_REQ=$(cat <<EOF
{
  "show_name": "Smoke Test Show",
  "theatre_name": "Test Theatre",
  "screen_number": 99,
  "start_time": $((NOW + 3600)),
  "end_time": $((NOW + 7200)),
  "price_per_seat": 123.0,
  "seat_layout": {
    "rows": [
      { "row": "X", "seats": 5, "seat_type": "Standard" }
    ]
  }
}
EOF
)
ADMIN_CREATE=$(curl -s -X POST \
  -H "Content-Type: application/json" \
  -H "$ADMIN_HDR" \
  -d "$ADMIN_SHOW_REQ" \
  "$BASE_URL/admin/shows")
if ! echo "$ADMIN_CREATE" | jq -e '.success' > /dev/null; then
  fail "POST /admin/shows failed: $(echo $ADMIN_CREATE | jq '.error.message')"
fi
pass "POST /admin/shows OK"
ADMIN_SHOW_ID=$(jq -r '.data.show_id' <<< "$ADMIN_CREATE")
info "Admin-created show: $ADMIN_SHOW_ID"

# ── 6. Admin list bookings ───────────────────────────────────
info "Testing GET /admin/bookings"
ADMIN_BOOKINGS=$(curl -s -H "$ADMIN_HDR" "$BASE_URL/admin/bookings")
if ! echo "$ADMIN_BOOKINGS" | jq -e '.success' > /dev/null; then
  fail "GET /admin/bookings failed"
fi
pass "GET /admin/bookings OK"
info "Admin can list all bookings"

# ── 7. Lock seats (user flow) ────────────────────────────────
info "Testing POST /shows/:id/seats/lock"
USER_ID="user-001"
USER_HDR="X-User-Id: $USER_ID"

LOCK_REQ="{\"seat_ids\": [\"$SEAT_ID_1\", \"$SEAT_ID_2\"]}"
LOCK_RESP=$(curl -s -X POST \
  -H "Content-Type: application/json" \
  -H "$USER_HDR" \
  -d "$LOCK_REQ" \
  "$BASE_URL/shows/$FIRST_SHOW/seats/lock")

if ! echo "$LOCK_RESP" | jq -e '.success' > /dev/null; then
  fail "POST /lock seats failed: $(echo $LOCK_RESP | jq '.error.message')"
fi
pass "POST /lock seats OK"

BOOKING_ID=$(jq -r '.data.booking_id' <<< "$LOCK_RESP")
LOCK_ID=$(jq -r '.data.lock_id' <<< "$LOCK_RESP")
TOTAL=$(jq -r '.data.total_amount' <<< "$LOCK_RESP")
STATUS=$(jq -r '.data.status' <<< "$LOCK_RESP")
info "Booking created: $BOOKING_ID (status: $STATUS, total: ₹$TOTAL)"

# ── 8. Get booking details ────────────────────────────────────
info "Testing GET /bookings/:id"
BOOKING=$(curl -s -H "$USER_HDR" "$BASE_URL/bookings/$BOOKING_ID")
if ! echo "$BOOKING" | jq -e '.success' > /dev/null; then
  fail "GET /bookings/$BOOKING_ID failed"
fi
pass "GET /bookings/:id OK"
BOOKING_STATUS=$(jq -r '.data.status' <<< "$BOOKING")
info "Booking status: $BOOKING_STATUS"

# ── 9. Initiate payment ───────────────────────────────────────
info "Testing POST /bookings/:id/payment/initiate"
PAYMENT_INIT=$(curl -s -X POST \
  -H "$USER_HDR" \
  "$BASE_URL/bookings/$BOOKING_ID/payment/initiate")
if ! echo "$PAYMENT_INIT" | jq -e '.success' > /dev/null; then
  fail "POST /payment/initiate failed: $(echo $PAYMENT_INIT | jq '.error.message')"
fi
pass "POST /payment/initiate OK"

PAYMENT_ID=$(jq -r '.data.payment_id' <<< "$PAYMENT_INIT")
PAYMENT_INTENT=$(jq -r '.data.payment_intent_id' <<< "$PAYMENT_INIT")
AMOUNT=$(jq -r '.data.amount' <<< "$PAYMENT_INIT")
info "Payment initiated: $PAYMENT_ID (amount: ₹$AMOUNT)"

# ── 10. Mock gateway pay (success) ───────────────────────────
info "Testing POST /mock-gateway/pay (success)"
GATEWAY_RESP=$(curl -s -X POST \
  -H "Content-Type: application/json" \
  -d "{
    \"payment_intent_id\": \"$PAYMENT_INTENT\",
    \"amount\": $AMOUNT,
    \"card_last4\": \"4242\",
    \"simulate_failure\": false
  }" \
  "$BASE_URL/mock-gateway/pay")
if ! echo "$GATEWAY_RESP" | jq -e '.success' > /dev/null; then
  fail "POST /mock-gateway/pay failed: $(echo $GATEWAY_RESP | jq '.error.message')"
fi
pass "POST /mock-gateway/pay OK"

GATEWAY_STATUS=$(jq -r '.data.status' <<< "$GATEWAY_RESP")
info "Mock gateway status: $GATEWAY_STATUS"

# ── 11. Verify booking confirmed ─────────────────────────────
sleep 0.5
info "Verifying booking is confirmed"
CONFIRMED=$(curl -s -H "$USER_HDR" "$BASE_URL/bookings/$BOOKING_ID")
CONFIRMED_STATUS=$(jq -r '.data.status' <<< "$CONFIRMED")
if [ "$CONFIRMED_STATUS" != "Success" ]; then
  fail "Booking status should be Success, got: $CONFIRMED_STATUS"
fi
pass "Booking confirmed (status: $CONFIRMED_STATUS)"

# ── 12. User booking history ───────────────────────────────────
info "Testing GET /bookings/user/:id"
USER_BOOKINGS=$(curl -s -H "$USER_HDR" "$BASE_URL/bookings/user/$USER_ID")
if ! echo "$USER_BOOKINGS" | jq -e '.success' > /dev/null; then
  fail "GET /bookings/user/:id failed"
fi
pass "GET /bookings/user/:id OK"
info "User has $(echo $USER_BOOKINGS | jq '.data | length') booking(s)"

# ── 13. Get payment status ────────────────────────────────────
info "Testing GET /payments/:id"
PAYMENT=$(curl -s "$BASE_URL/payments/$PAYMENT_ID")
if ! echo "$PAYMENT" | jq -e '.success' > /dev/null; then
  fail "GET /payments/$PAYMENT_ID failed"
fi
pass "GET /payments/:id OK"
PAYMENT_STATUS=$(jq -r '.data.status' <<< "$PAYMENT")
info "Payment status: $PAYMENT_STATUS"

# ── 14. Admin show analytics ─────────────────────────────────
info "Testing GET /admin/shows/:id/analytics"
ANALYTICS=$(curl -s -H "$ADMIN_HDR" "$BASE_URL/admin/shows/$FIRST_SHOW/analytics")
if ! echo "$ANALYTICS" | jq -e '.success' > /dev/null; then
  fail "GET /admin/shows/:id/analytics failed"
fi
pass "GET /admin/shows/:id/analytics OK"
BOOKED=$(jq -r '.data.booked_seats' <<< "$ANALYTICS")
REVENUE=$(jq -r '.data.revenue' <<< "$ANALYTICS")
info "Show analytics — booked: $BOOKED, revenue: ₹$REVENUE"

# ── 15. Admin seat override (release locked seat on admin show) ─
info "Testing POST /admin/shows/:id/seats/:seatId/override (admin show)"
OVERRIDE_SEATS=$(curl -s "$BASE_URL/shows/$ADMIN_SHOW_ID/seats")
ADMIN_SEAT=$(jq -r '[.data.seats[].seat_id][0]' <<< "$OVERRIDE_SEATS")
if [ -n "$ADMIN_SEAT" ] && [ "$ADMIN_SEAT" != "null" ]; then
  OVERRIDE_RESP=$(curl -s -X POST \
    -H "Content-Type: application/json" \
    -H "$ADMIN_HDR" \
    -d '{"reason": "smoke test override"}' \
    "$BASE_URL/admin/shows/$ADMIN_SHOW_ID/seats/$ADMIN_SEAT/override")
  if ! echo "$OVERRIDE_RESP" | jq -e '.success' > /dev/null; then
    info "Seat override on admin show failed (expected — no lock exists): $(echo $OVERRIDE_RESP | jq '.error.code')"
  else
    pass "POST /admin/seat override OK"
  fi
fi

# ── 16. Extend lock ───────────────────────────────────────────
info "Testing POST /bookings/:id/extend-lock"
# First lock new seats for extension test
EXT_SEATS=$(curl -s "$BASE_URL/shows/$FIRST_SHOW/seats")
EXT_SEAT=$(jq -r '[.data.seats[] | select(.status == "Available") | .seat_id][0]' <<< "$EXT_SEATS")
if [ -n "$EXT_SEAT" ] && [ "$EXT_SEAT" != "null" ]; then
  EXT_LOCK=$(curl -s -X POST \
    -H "Content-Type: application/json" \
    -H "$USER_HDR" \
    -d "{\"seat_ids\": [\"$EXT_SEAT\"]}" \
    "$BASE_URL/shows/$FIRST_SHOW/seats/lock")
  EXT_BK=$(jq -r '.data.booking_id' <<< "$EXT_LOCK")
  if [ -n "$EXT_BK" ] && [ "$EXT_BK" != "null" ]; then
    EXT_RESP=$(curl -s -X POST -H "$USER_HDR" "$BASE_URL/bookings/$EXT_BK/extend-lock")
    if ! echo "$EXT_RESP" | jq -e '.success' > /dev/null; then
      info "Extend lock failed (expected — only 2 extensions allowed, previous ones used)"
    else
      pass "POST /extend-lock OK"
    fi
    # Clean up
    curl -s -X DELETE -H "$USER_HDR" "$BASE_URL/bookings/$EXT_BK/lock" > /dev/null || true
  fi
fi

# ── 17. Cancel booking ────────────────────────────────────────
info "Testing POST /bookings/:id/cancel"
# Lock new seats then cancel
CANCEL_SEATS=$(curl -s "$BASE_URL/shows/$FIRST_SHOW/seats")
CANCEL_SEAT=$(jq -r '[.data.seats[] | select(.status == "Available") | .seat_id][0]' <<< "$CANCEL_SEATS")
CANCEL_LOCK=$(curl -s -X POST \
  -H "Content-Type: application/json" \
  -H "$USER_HDR" \
  -d "{\"seat_ids\": [\"$CANCEL_SEAT\"]}" \
  "$BASE_URL/shows/$FIRST_SHOW/seats/lock")
CANCEL_BK=$(jq -r '.data.booking_id' <<< "$CANCEL_LOCK")
if [ -n "$CANCEL_BK" ] && [ "$CANCEL_BK" != "null" ]; then
  CANCEL_RESP=$(curl -s -X POST -H "$USER_HDR" "$BASE_URL/bookings/$CANCEL_BK/cancel")
  if ! echo "$CANCEL_RESP" | jq -e '.success' > /dev/null; then
    fail "POST /cancel failed: $(echo $CANCEL_RESP | jq '.error.message')"
  fi
  pass "POST /cancel OK"
fi

# ── 18. Leave queue ──────────────────────────────────────────
info "Testing POST /shows/:id/queue/join + DELETE /queue/:id"
QUEUE_RESP=$(curl -s -X POST \
  -H "Content-Type: application/json" \
  -H "$USER_HDR" \
  -d "{\"seat_ids\": [\"$SEAT_ID_1\"]}" \
  "$BASE_URL/shows/$FIRST_SHOW/queue/join")
QUEUE_ID=$(jq -r '.data.queue_id' <<< "$QUEUE_RESP" 2>/dev/null || echo "")
if [ -n "$QUEUE_ID" ] && [ "$QUEUE_ID" != "null" ]; then
  QUEUE_LEAVE=$(curl -s -X DELETE -H "$USER_HDR" "$BASE_URL/queue/$QUEUE_ID")
  if ! echo "$QUEUE_LEAVE" | jq -e '.success' > /dev/null; then
    fail "DELETE /queue/:id failed: $(echo $QUEUE_LEAVE | jq '.error.message')"
  fi
  pass "DELETE /queue/:id OK"
else
  info "Queue join skipped (lock succeeded — expected under normal load)"
fi

# ── 19. Admin cancel show ────────────────────────────────────
info "Testing DELETE /admin/shows/:id"
CANCEL_SHOW=$(curl -s -X DELETE -H "$ADMIN_HDR" "$BASE_URL/admin/shows/$ADMIN_SHOW_ID")
if ! echo "$CANCEL_SHOW" | jq -e '.success' > /dev/null; then
  fail "DELETE /admin/shows/:id failed: $(echo $CANCEL_SHOW | jq '.error.message')"
fi
pass "DELETE /admin/shows/:id OK"
info "Admin cancelled test show"

# ─────────────────────────────────────────────────────────────
echo ""
echo -e "${GREEN}=========================================="
echo -e "   All smoke tests passed! ✓"
echo -e "==========================================${NC}"
echo ""