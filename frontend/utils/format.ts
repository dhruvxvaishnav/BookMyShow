import { format, formatDistanceToNow } from 'date-fns';

/**
 * Format a Unix timestamp (seconds) to a readable time string.
 * e.g. 1746032400 → "2:00 PM"
 */
export function formatTime(unixSeconds: number): string {
  return format(new Date(unixSeconds * 1000), 'h:mm a');
}

/**
 * Format a Unix timestamp (seconds) to a full date string.
 * e.g. 1746032400 → "Fri, May 1, 2026"
 */
export function formatDate(unixSeconds: number): string {
  return format(new Date(unixSeconds * 1000), 'EEE, MMM d, yyyy');
}

/**
 * Format a Unix timestamp to date + time.
 * e.g. 1746032400 → "Fri, May 1 · 2:00 PM"
 */
export function formatDateTime(unixSeconds: number): string {
  return format(new Date(unixSeconds * 1000), "EEE, MMM d · h:mm a");
}

/**
 * Format seconds remaining into MM:SS
 */
export function formatCountdown(seconds: number): string {
  if (seconds <= 0) return '00:00';
  const m = Math.floor(seconds / 60);
  const s = seconds % 60;
  return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
}

/**
 * Format a price amount with ₹ symbol and commas.
 */
export function formatPrice(amount: number): string {
  return `₹${amount.toLocaleString('en-IN', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
}

/**
 * Format a seat number list for display.
 * e.g. ["A1", "A2", "A3"] → "A1, A2, A3"
 */
export function formatSeatList(seats: string[]): string {
  return seats.join(', ');
}

/**
 * Format a list with "and" conjunction.
 * e.g. ["A1", "A2", "A3"] → "A1, A2 and A3"
 */
export function formatListWithAnd(items: string[]): string {
  if (items.length === 0) return '';
  if (items.length === 1) return items[0];
  if (items.length === 2) return `${items[0]} and ${items[1]}`;
  return `${items.slice(0, -1).join(', ')} and ${items[items.length - 1]}`;
}

/**
 * Get a relative time string from a Unix timestamp.
 */
export function formatRelativeTime(unixSeconds: number): string {
  return formatDistanceToNow(new Date(unixSeconds * 1000), { addSuffix: true });
}

/**
 * Generate a short booking ID display string.
 * e.g. "BMS-AB12CD34"
 */
export function formatBookingId(id: string): string {
  const short = id.replace(/-/g, '').slice(0, 8).toUpperCase();
  return `BMS-${short}`;
}

/**
 * Truncate a UUID for display.
 * e.g. "abcd1234-abcd-1234-abcd-123456789abc" → "abcd1234"
 */
export function shortId(id: string): string {
  return id.split('-')[0];
}
