import { ApiError } from '@/types/api';

/**
 * Maps API error codes to user-friendly messages.
 */
export const ERROR_MESSAGES: Record<string, string> = {
  SEATS_UNAVAILABLE:            'One or more seats are no longer available. They have been highlighted.',
  MAX_SEATS_EXCEEDED:           'You can select a maximum of 10 seats per booking.',
  LOCK_EXPIRED:                  'Your lock has expired. Please select seats again.',
  LOCK_MAX_EXTENSIONS_REACHED:   "You've reached the maximum number of lock extensions.",
  RATE_LIMIT_EXCEEDED:           'Too many requests. Please wait a moment.',
  SHOW_NOT_FOUND:                'This show is no longer available.',
  BOOKING_NOT_FOUND:             'Booking not found.',
  VALIDATION_ERROR:              'Please check the form and try again.',
  UNAUTHORIZED:                  'You are not authorized to perform this action.',
  INTERNAL_ERROR:                'Something went wrong on our end. Please try again in a moment.',
  NETWORK_ERROR:                 'Unable to connect. Check your internet connection.',
  TIMEOUT:                       'Request timed out. Please try again.',
};

export type ToastVariant = 'success' | 'error' | 'warning' | 'info';

/**
 * Get user-friendly message from an ApiError.
 */
export function getErrorMessage(error: unknown): string {
  if (error instanceof ApiError) {
    return ERROR_MESSAGES[error.code] ?? error.message;
  }
  if (error instanceof Error) {
    if (error.message.includes('Network Error')) return ERROR_MESSAGES.NETWORK_ERROR;
    if (error.message.includes('timeout')) return ERROR_MESSAGES.TIMEOUT;
    return error.message;
  }
  return 'An unexpected error occurred.';
}

/**
 * Determine toast variant from error code.
 */
export function getErrorVariant(code: string): ToastVariant {
  switch (code) {
    case 'LOCK_EXPIRED':
    case 'SEATS_UNAVAILABLE':
    case 'RATE_LIMIT_EXCEEDED':
      return 'warning';
    case 'UNAUTHORIZED':
    case 'BOOKING_NOT_FOUND':
    case 'SHOW_NOT_FOUND':
      return 'error';
    default:
      return 'error';
  }
}

/**
 * Extract conflicting seat IDs from ApiError details.
 */
export function getConflictingSeats(error: ApiError): string[] {
  if (error.details && Array.isArray(error.details.conflict_seats)) {
    return error.details.conflict_seats as string[];
  }
  if (error.details && Array.isArray(error.details.seat_ids)) {
    return error.details.seat_ids as string[];
  }
  return [];
}
