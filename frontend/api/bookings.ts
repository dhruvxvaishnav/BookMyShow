import apiClient from './client';
import type { Booking } from '@/types/api';

interface LockSeatsRequest { seat_ids: string[]; }
interface LockSeatsResponse extends Booking {}

export async function lockSeats(showId: string, seatIds: string[]): Promise<LockSeatsResponse> {
  const response = await apiClient.post<LockSeatsResponse>(
    `/shows/${showId}/seats/lock`,
    { seat_ids: seatIds } as LockSeatsRequest
  );
  return response.data;
}

export async function extendLock(bookingId: string): Promise<Booking> {
  const response = await apiClient.post<Booking>(`/bookings/${bookingId}/extend-lock`);
  return response.data;
}

export async function releaseLock(bookingId: string): Promise<void> {
  await apiClient.delete(`/bookings/${bookingId}/lock`);
}

export async function getBooking(bookingId: string): Promise<Booking> {
  const response = await apiClient.get<Booking>(`/bookings/${bookingId}`);
  return response.data;
}

export async function cancelBooking(bookingId: string): Promise<Booking> {
  const response = await apiClient.post<Booking>(`/bookings/${bookingId}/cancel`);
  return response.data;
}

export async function getUserBookings(userId: string): Promise<Booking[]> {
  const response = await apiClient.get<Booking[]>(`/bookings/user/${userId}`);
  return response.data;
}
