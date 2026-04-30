// ── API Response Envelope ────────────────────────────────
export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: ApiErrorDetail;
  timestamp: string;
}

export interface ApiErrorDetail {
  code: string;
  message: string;
  details?: Record<string, unknown>;
}

// ── ApiError thrown by axios interceptor ────────────────
export class ApiError extends Error {
  constructor(
    public readonly code: string,
    public readonly message: string,
    public readonly details?: Record<string, unknown>,
    public readonly status?: number
  ) {
    super(message);
    this.name = 'ApiError';
  }
}

// ── Show ──────────────────────────────────────────────────
export interface Show {
  show_id: string;
  name: string;
  theatre_name: string;
  screen_number: number;
  start_time: number; // Unix timestamp seconds
  end_time: number;
  price_per_seat: number;
  created_at: string;
}

export interface ShowAvailability {
  show_id: string;
  total_seats: number;
  booked_seats: number;
  locked_seats: number;
  available_seats: number;
  occupancy_percent: number;
}

export interface Seat {
  seat_id: string;
  seat_number: string;
  row_label: string;
  seat_type: 'Standard' | 'Premium' | 'Recliner';
  status: 'Available' | 'Locked' | 'Booked';
  lock_expires_at: number | null; // Unix timestamp seconds
  price: number;
}

export interface SeatLayoutResponse {
  show_id: string;
  seats: Seat[];
  page: number;
  limit: number;
}

// ── Booking ──────────────────────────────────────────────
export type BookingStatus = 'Pending' | 'PaymentPending' | 'Success' | 'Expired' | 'Cancelled' | 'PartialSuccess';

export interface Booking {
  booking_id: string;
  lock_id: string;
  show_id: string;
  seat_ids: string[];
  total_amount: number;
  expires_at: number; // Unix timestamp seconds
  status: BookingStatus;
  user_id: string;
  created_at: string;
  show?: Show;
  seats?: Seat[];
  payment_id?: string;
}

// ── Payment ───────────────────────────────────────────────
export type PaymentStatus = 'pending' | 'completed' | 'failed' | 'refunded';

export interface Payment {
  payment_id: string;
  payment_intent_id: string;
  booking_id: string;
  amount: number;
  gateway_name: string;
  status: PaymentStatus;
  card_last4?: string;
  created_at: string;
}

export interface PaymentInitiateResponse {
  payment_id: string;
  payment_intent_id: string;
  amount: number;
  gateway_name: string;
  status: 'pending';
}

export interface MockGatewayPayRequest {
  payment_intent_id: string;
  amount: number;
  card_last4: string;
  simulate_failure: boolean;
}

// ── Queue ────────────────────────────────────────────────
export type QueueStatus = 'Waiting' | 'Processing' | 'Locked' | 'Conflict' | 'Expired';

export interface QueueEntry {
  queue_id: string;
  status: QueueStatus;
  position: number;
  booking_id?: string;
  lock_id?: string;
  conflict_seats: string[] | null;
}

// ── Admin ─────────────────────────────────────────────────
export interface AdminShowAnalytics {
  show_id: string;
  name: string;
  total_seats: number;
  available: number;
  locked: number;
  booked: number;
  occupancy_percent: number;
  revenue: number;
  cancelled_bookings: number;
}

export interface CreateShowRequest {
  show_name: string;
  theatre_name: string;
  screen_number: number;
  start_time: number; // Unix timestamp seconds
  end_time: number;
  price_per_seat: number;
  seat_layout: { rows: RowConfig[] };
}

export interface RowConfig {
  row: string;
  seats: number;
  seat_type: 'Standard' | 'Premium' | 'Recliner';
}

// ── BookingStore ──────────────────────────────────────────
export interface BookingState {
  booking: Booking | null;
  payment: PaymentInitiateResponse | null;
  extensionsUsed: number;
  maxExtensions: number;
}
