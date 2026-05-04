import apiClient from './client';
import type { AdminShowAnalytics, CreateShowRequest, Booking, Payment, AuditLog } from '@/types/api';
import type { Seat } from '@/types/api';

export async function createShow(data: CreateShowRequest): Promise<void> {
  await apiClient.post('/admin/shows', data);
}

export async function cancelShow(showId: string): Promise<void> {
  await apiClient.delete(`/admin/shows/${showId}`);
}

export async function getShowAnalytics(showId: string): Promise<AdminShowAnalytics> {
  const response = await apiClient.get<AdminShowAnalytics>(`/admin/shows/${showId}/analytics`);
  return response.data;
}

export async function forceReleaseSeat(showId: string, seatId: string): Promise<void> {
  await apiClient.post(`/admin/shows/${showId}/seats/${seatId}/override`);
}

export async function getAdminBookings(): Promise<Booking[]> {
  const response = await apiClient.get<Booking[]>('/admin/bookings');
  return response.data;
}

export async function getAdminBooking(bookingId: string): Promise<Booking> {
  const response = await apiClient.get<Booking>(`/admin/bookings/${bookingId}`);
  return response.data;
}

export async function getAuditLogs(filters?: { bookingId?: string; userId?: string }): Promise<AuditLog[]> {
  const params = new URLSearchParams();
  if (filters?.bookingId) params.set('booking_id', filters.bookingId);
  if (filters?.userId) params.set('user_id', filters.userId);
  const suffix = params.toString() ? `?${params.toString()}` : '';
  const response = await apiClient.get<AuditLog[]>(`/admin/audit${suffix}`);
  return response.data;
}

export async function issueRefund(paymentId: string): Promise<void> {
  await apiClient.post(`/admin/payments/${paymentId}/refund`);
}

export async function getAdminSeatLayout(showId: string): Promise<Seat[]> {
  const response = await apiClient.get<{ seats: Seat[] }>(`/shows/${showId}/seats`);
  return response.data.seats;
}
