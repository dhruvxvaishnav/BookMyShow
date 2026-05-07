import client from './client';
import type { Venue } from '@/types/api';

export async function getVenues(): Promise<Venue[]> {
  const res = await client.get<Venue[]>('/venues');
  return res.data;
}
