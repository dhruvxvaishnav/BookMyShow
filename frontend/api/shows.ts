import apiClient from './client';
import type { Show, ShowAvailability, SeatLayoutResponse } from '@/types/api';

export async function getShows(): Promise<Show[]> {
  const response = await apiClient.get<Show[]>('/shows');
  return response.data;
}

export async function getShow(showId: string): Promise<Show> {
  const response = await apiClient.get<Show>(`/shows/${showId}`);
  return response.data;
}

export async function getShowAvailability(showId: string): Promise<ShowAvailability> {
  const response = await apiClient.get<ShowAvailability>(`/shows/${showId}/availability`);
  return response.data;
}

export async function getSeatLayout(showId: string, page = 1, limit = 500): Promise<SeatLayoutResponse> {
  const response = await apiClient.get<SeatLayoutResponse>(
    `/shows/${showId}/seats`,
    { params: { page, limit } }
  );
  return response.data;
}
