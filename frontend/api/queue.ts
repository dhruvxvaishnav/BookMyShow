import apiClient from './client';
import type { QueueEntry } from '@/types/api';

interface JoinQueueRequest { seat_ids: string[]; }

export async function joinQueue(showId: string, seatIds: string[]): Promise<QueueEntry> {
  const response = await apiClient.post<QueueEntry>(
    `/shows/${showId}/queue/join`,
    { seat_ids: seatIds } as JoinQueueRequest
  );
  return response.data;
}

export async function getQueueStatus(queueId: string): Promise<QueueEntry> {
  const response = await apiClient.get<QueueEntry>(`/queue/${queueId}/status`);
  return response.data;
}

export async function leaveQueue(queueId: string): Promise<void> {
  await apiClient.delete(`/queue/${queueId}`);
}
