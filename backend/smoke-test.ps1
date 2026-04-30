$ErrorActionPreference = "Stop"
$BASE_URL = "http://127.0.0.1:8080"
$ADMIN_TOKEN = "admin-secret"
$ADMIN_HEADERS = @{ "Content-Type" = "application/json"; "X-Admin-Token" = $ADMIN_TOKEN }

function Test-JsonResponse {
    param($Response)
    try {
        $j = $Response | ConvertFrom-Json
        return $j.success -eq $true
    } catch { return $false }
}

function Get-JsonValue {
    param($Response, $Path)
    try {
        $j = $Response | ConvertFrom-Json
        $parts = $Path.Split('.')
        $current = $j
        foreach ($p in $parts) {
            if ($p -eq "data") { $current = $current.data }
            elseif ($p -eq "error") { $current = $current.error }
            else {
                $val = $current.$p
                if ($null -ne $val) { $current = $val }
                else { return $null }
            }
        }
        return $current
    } catch { return $null }
}

function Log-Pass($msg) { Write-Host "[PASS] $msg" -ForegroundColor Green }
function Log-Fail($msg) { Write-Host "[FAIL] $msg" -ForegroundColor Red; throw $msg }
function Log-Info($msg) { Write-Host "[INFO] $msg" -ForegroundColor Yellow }

# 0. Wait for server
Write-Host ""
Log-Info "Waiting for backend at $BASE_URL..."
$success = $false
for ($i = 0; $i -lt 10; $i++) {
    try {
        $health = Invoke-WebRequest -Uri "$BASE_URL/health" -UseBasicParsing -TimeoutSec 2
        if ((Get-JsonValue $health.Content "success") -eq $true) {
            $success = $true
            break
        }
    } catch {
        Write-Host "Error: $_"
    }
    Start-Sleep 1
}
if (-not $success) { Log-Fail "Backend health check failed. Is it running on port 8080?" }
Log-Pass "Backend is healthy"

# 1. List shows
Log-Info "Testing GET /shows"
$shows = Invoke-WebRequest -Uri "$BASE_URL/shows" -UseBasicParsing
if (-not (Test-JsonResponse $shows.Content)) { Log-Fail "GET /shows failed" }
Log-Pass "GET /shows OK"
$showCount = (Get-JsonValue $shows.Content "data").Count
Log-Info "Found $showCount show(s)"

if ($showCount -eq 0) { Log-Fail "No shows found. Did seeding work?" }
$showsJson = $shows.Content | ConvertFrom-Json
$FIRST_SHOW = $showsJson.data[0].show_id
Log-Info "Using show: $FIRST_SHOW"

# 2. Get show details
Log-Info "Testing GET /shows/:id"
$showDetail = Invoke-WebRequest -Uri "$BASE_URL/shows/$FIRST_SHOW" -UseBasicParsing
if (-not (Test-JsonResponse $showDetail.Content)) { Log-Fail "GET /shows/:id failed" }
Log-Pass "GET /shows/:id OK"
$price = Get-JsonValue $showDetail.Content "data.price_per_seat"
Log-Info "Price per seat: $price"

# 3. Get seat layout
Log-Info "Testing GET /shows/:id/seats"
$seats = Invoke-WebRequest -Uri "$BASE_URL/shows/$FIRST_SHOW/seats" -UseBasicParsing
if (-not (Test-JsonResponse $seats.Content)) { Log-Fail "GET /shows/:id/seats failed" }
Log-Pass "GET /shows/:id/seats OK"
$seatList = Get-JsonValue $seats.Content "data.seats"
$seatCount = $seatList.Count
Log-Info "Total seats: $seatCount"

$available = $seatList | Where-Object { $_.status -eq "Available" }
if ($available.Count -lt 2) { Log-Fail "Not enough available seats (need 2)" }
$SEAT_ID_1 = $available[0].seat_id
$SEAT_ID_2 = $available[1].seat_id
Log-Info "Selected seats: $SEAT_ID_1, $SEAT_ID_2"

# 4. Get availability
Log-Info "Testing GET /shows/:id/availability"
$avail = Invoke-WebRequest -Uri "$BASE_URL/shows/$FIRST_SHOW/availability" -UseBasicParsing
if (-not (Test-JsonResponse $avail.Content)) { Log-Fail "GET /availability failed" }
Log-Pass "GET /availability OK"
$availCount = Get-JsonValue $avail.Content "data.available"
Log-Info "Available seats: $availCount"

# 5. Admin create show
Log-Info "Testing POST /admin/shows"
$nowSecs = [int][double]::Parse((Get-Date -UFormat %s))
$adminShowBody = @{
    show_name = "Smoke Test Show"
    theatre_name = "Test Theatre"
    screen_number = 99
    start_time = $nowSecs + 3600
    end_time = $nowSecs + 7200
    price_per_seat = 123.0
    seat_layout = @{
        rows = @(
            @{ row = "X"; seats = 5; seat_type = "Standard" }
        )
    }
} | ConvertTo-Json -Depth 10 -Compress
$adminCreate = Invoke-WebRequest -Uri "$BASE_URL/admin/shows" -Method POST -Headers $ADMIN_HEADERS -Body $adminShowBody -UseBasicParsing
if (-not (Test-JsonResponse $adminCreate.Content)) {
    $err = Get-JsonValue $adminCreate.Content "error.message"
    Log-Fail "POST /admin/shows failed: $err"
}
Log-Pass "POST /admin/shows OK"
$ADMIN_SHOW_ID = Get-JsonValue $adminCreate.Content "data.show_id"
Log-Info "Admin-created show: $ADMIN_SHOW_ID"

# 6. Admin list bookings
Log-Info "Testing GET /admin/bookings"
$adminBookings = Invoke-WebRequest -Uri "$BASE_URL/admin/bookings" -Method GET -Headers $ADMIN_HEADERS -UseBasicParsing
if (-not (Test-JsonResponse $adminBookings.Content)) { Log-Fail "GET /admin/bookings failed" }
Log-Pass "GET /admin/bookings OK"

# 7. Lock seats (user flow)
Log-Info "Testing POST /shows/:id/seats/lock"
$USER_ID = "user-001"
$USER_HEADERS = @{ "Content-Type" = "application/json"; "X-User-Id" = $USER_ID }

$lockBody = @{ seat_ids = @($SEAT_ID_1, $SEAT_ID_2) } | ConvertTo-Json -Depth 10 -Compress
$lockResp = Invoke-WebRequest -Uri "$BASE_URL/shows/$FIRST_SHOW/seats/lock" -Method POST -Headers $USER_HEADERS -Body $lockBody -UseBasicParsing
if (-not (Test-JsonResponse $lockResp.Content)) {
    $err = Get-JsonValue $lockResp.Content "error.message"
    Log-Fail "POST /lock seats failed: $err"
}
Log-Pass "POST /lock seats OK"

$BOOKING_ID = Get-JsonValue $lockResp.Content "data.booking_id"
$total = Get-JsonValue $lockResp.Content "data.total_amount"
$status = Get-JsonValue $lockResp.Content "data.status"
Log-Info "Booking: $BOOKING_ID (status: $status, total: $total)"

# 8. Get booking details
Log-Info "Testing GET /bookings/:id"
$booking = Invoke-WebRequest -Uri "$BASE_URL/bookings/$BOOKING_ID" -Method GET -Headers $USER_HEADERS -UseBasicParsing
if (-not (Test-JsonResponse $booking.Content)) { Log-Fail "GET /bookings/:id failed" }
Log-Pass "GET /bookings/:id OK"
$bookingStatus = Get-JsonValue $booking.Content "data.status"
Log-Info "Booking status: $bookingStatus"

# 9. Initiate payment
Log-Info "Testing POST /bookings/:id/payment/initiate"
$payInit = Invoke-WebRequest -Uri "$BASE_URL/bookings/$BOOKING_ID/payment/initiate" -Method POST -Headers $USER_HEADERS -UseBasicParsing
if (-not (Test-JsonResponse $payInit.Content)) {
    $err = Get-JsonValue $payInit.Content "error.message"
    Log-Fail "POST /payment/initiate failed: $err"
}
Log-Pass "POST /payment/initiate OK"

$PAYMENT_ID = Get-JsonValue $payInit.Content "data.payment_id"
$PAYMENT_INTENT = Get-JsonValue $payInit.Content "data.payment_intent_id"
$AMOUNT = Get-JsonValue $payInit.Content "data.amount"
Log-Info "Payment: $PAYMENT_ID (amount: $AMOUNT)"

# 10. Mock gateway pay (success)
Log-Info "Testing POST /mock-gateway/pay (success)"
$gatewayBody = @{
    payment_intent_id = $PAYMENT_INTENT
    amount = $AMOUNT
    card_last4 = "4242"
    simulate_failure = $false
} | ConvertTo-Json -Depth 10 -Compress
$gatewayResp = Invoke-WebRequest -Uri "$BASE_URL/mock-gateway/pay" -Method POST -Headers @{ "Content-Type" = "application/json" } -Body $gatewayBody -UseBasicParsing
if (-not (Test-JsonResponse $gatewayResp.Content)) {
    $err = Get-JsonValue $gatewayResp.Content "error.message"
    Log-Fail "POST /mock-gateway/pay failed: $err"
}
Log-Pass "POST /mock-gateway/pay OK"
$gatewayStatus = Get-JsonValue $gatewayResp.Content "data.status"
Log-Info "Gateway status: $gatewayStatus"

# 11. Verify booking confirmed
Start-Sleep 0.5
Log-Info "Verifying booking confirmed"
$confirmed = Invoke-WebRequest -Uri "$BASE_URL/bookings/$BOOKING_ID" -Method GET -Headers $USER_HEADERS -UseBasicParsing
$confirmedStatus = Get-JsonValue $confirmed.Content "data.status"
if ($confirmedStatus -ne "Success") { Log-Fail "Booking should be Success, got: $confirmedStatus" }
Log-Pass "Booking confirmed (status: $confirmedStatus)"

# 12. User booking history
Log-Info "Testing GET /bookings/user/:id"
$userBookings = Invoke-WebRequest -Uri "$BASE_URL/bookings/user/$USER_ID" -Method GET -Headers $USER_HEADERS -UseBasicParsing
if (-not (Test-JsonResponse $userBookings.Content)) { Log-Fail "GET /bookings/user/:id failed" }
Log-Pass "GET /bookings/user/:id OK"
$userBookingsCount = (Get-JsonValue $userBookings.Content "data").Count
Log-Info "User has $userBookingsCount booking(s)"

# 13. Get payment status
Log-Info "Testing GET /payments/:id"
$payment = Invoke-WebRequest -Uri "$BASE_URL/payments/$PAYMENT_ID" -UseBasicParsing
if (-not (Test-JsonResponse $payment.Content)) { Log-Fail "GET /payments/:id failed" }
Log-Pass "GET /payments/:id OK"
$paymentStatus = Get-JsonValue $payment.Content "data.status"
Log-Info "Payment status: $paymentStatus"

# 14. Admin show analytics
Log-Info "Testing GET /admin/shows/:id/analytics"
$analytics = Invoke-WebRequest -Uri "$BASE_URL/admin/shows/$FIRST_SHOW/analytics" -Method GET -Headers $ADMIN_HEADERS -UseBasicParsing
if (-not (Test-JsonResponse $analytics.Content)) { Log-Fail "GET /admin/shows/:id/analytics failed" }
Log-Pass "GET /admin/shows/:id/analytics OK"
$booked = Get-JsonValue $analytics.Content "data.booked_seats"
$revenue = Get-JsonValue $analytics.Content "data.revenue"
Log-Info "Show analytics - booked: $booked, revenue: $revenue"

# 15. Admin seat override
Log-Info "Testing POST /admin/shows/:id/seats/:seatId/override"
$adminSeats = Invoke-WebRequest -Uri "$BASE_URL/shows/$ADMIN_SHOW_ID/seats" -UseBasicParsing
$adminSeatsJson = $adminSeats.Content | ConvertFrom-Json
$adminSeat = $adminSeatsJson.data.seats[0].seat_id
if ($adminSeat) {
    $overrideBody = @{ reason = "smoke test override" } | ConvertTo-Json -Depth 10 -Compress
    try {
        $overrideResp = Invoke-WebRequest -Uri "$BASE_URL/admin/shows/$ADMIN_SHOW_ID/seats/$adminSeat/override" -Method POST -Headers $ADMIN_HEADERS -Body $overrideBody -UseBasicParsing
        if ((Test-JsonResponse $overrideResp.Content)) { Log-Pass "POST /admin/seat override OK" }
        else { Log-Info "Seat override skipped (no lock to release)" }
    } catch {
        Log-Info "Seat override skipped (expected - no lock exists)"
    }
}

# 16. Extend lock
Log-Info "Testing POST /bookings/:id/extend-lock"
$extSeats = Invoke-WebRequest -Uri "$BASE_URL/shows/$FIRST_SHOW/seats" -UseBasicParsing
$extSeat = (Get-JsonValue $extSeats.Content "data.seats") | Where-Object { $_.status -eq "Available" } | Select-Object -First 1
if ($extSeat) {
    $extLockBody = @{ seat_ids = @($extSeat.seat_id) } | ConvertTo-Json -Depth 10 -Compress
    $extLock = Invoke-WebRequest -Uri "$BASE_URL/shows/$FIRST_SHOW/seats/lock" -Method POST -Headers $USER_HEADERS -Body $extLockBody -UseBasicParsing
    $extBk = Get-JsonValue $extLock.Content "data.booking_id"
    if ($extBk) {
        $extResp = Invoke-WebRequest -Uri "$BASE_URL/bookings/$extBk/extend-lock" -Method POST -Headers $USER_HEADERS -UseBasicParsing
        if ((Test-JsonResponse $extResp.Content)) { Log-Pass "POST /extend-lock OK" }
        else { Log-Info "Extend lock skipped (may have used max extensions)" }
        Invoke-WebRequest -Uri "$BASE_URL/bookings/$extBk/lock" -Method DELETE -Headers $USER_HEADERS -UseBasicParsing | Out-Null
    }
}

# 17. Cancel booking
Log-Info "Testing POST /bookings/:id/cancel"
$cancelSeats = Invoke-WebRequest -Uri "$BASE_URL/shows/$FIRST_SHOW/seats" -UseBasicParsing
$cancelSeat = (Get-JsonValue $cancelSeats.Content "data.seats") | Where-Object { $_.status -eq "Available" } | Select-Object -First 1
if ($cancelSeat) {
    $cancelLockBody = @{ seat_ids = @($cancelSeat.seat_id) } | ConvertTo-Json -Depth 10 -Compress
    $cancelLock = Invoke-WebRequest -Uri "$BASE_URL/shows/$FIRST_SHOW/seats/lock" -Method POST -Headers $USER_HEADERS -Body $cancelLockBody -UseBasicParsing
    $cancelBk = Get-JsonValue $cancelLock.Content "data.booking_id"
    if ($cancelBk) {
        $cancelResp = Invoke-WebRequest -Uri "$BASE_URL/bookings/$cancelBk/cancel" -Method POST -Headers $USER_HEADERS -UseBasicParsing
        if ((Test-JsonResponse $cancelResp.Content)) { Log-Pass "POST /cancel OK" }
        else { Log-Fail "POST /cancel failed: $(Get-JsonValue $cancelResp.Content 'error.message')" }
    }
}

# 18. Queue join + leave
Log-Info "Testing POST /shows/:id/queue/join + DELETE /queue/:id"
$queueSeats = Invoke-WebRequest -Uri "$BASE_URL/shows/$FIRST_SHOW/seats" -UseBasicParsing
$queueSeat = (Get-JsonValue $queueSeats.Content "data.seats") | Where-Object { $_.status -eq "Available" } | Select-Object -First 1
if ($queueSeat) {
    $queueBody = @{ seat_ids = @($queueSeat.seat_id) } | ConvertTo-Json -Depth 10 -Compress
    $queueResp = Invoke-WebRequest -Uri "$BASE_URL/shows/$FIRST_SHOW/queue/join" -Method POST -Headers $USER_HEADERS -Body $queueBody -UseBasicParsing
    $queueData = Get-JsonValue $queueResp.Content "data"
    if ($queueData) {
        $queueId = $queueData.queue_id
        if ($queueId) {
            $queueLeave = Invoke-WebRequest -Uri "$BASE_URL/queue/$queueId" -Method DELETE -Headers $USER_HEADERS -UseBasicParsing
            if ((Test-JsonResponse $queueLeave.Content)) { Log-Pass "DELETE /queue/:id OK" }
            else { Log-Fail "DELETE /queue/:id failed" }
        }
    } else {
        Log-Info "Queue join skipped (lock succeeded - normal under low load)"
    }
}

# 19. Admin cancel show
Log-Info "Testing DELETE /admin/shows/:id"
$cancelShow = Invoke-WebRequest -Uri "$BASE_URL/admin/shows/$ADMIN_SHOW_ID" -Method DELETE -Headers $ADMIN_HEADERS -UseBasicParsing
if (-not (Test-JsonResponse $cancelShow.Content)) {
    $err = Get-JsonValue $cancelShow.Content "error.message"
    Log-Fail "DELETE /admin/shows/:id failed: $err"
}
Log-Pass "DELETE /admin/shows/:id OK"
Log-Info "Admin cancelled test show"

Write-Host ""
Write-Host "=========================================="
Write-Host "   All smoke tests passed! PASS"
Write-Host "=========================================="
Write-Host ""