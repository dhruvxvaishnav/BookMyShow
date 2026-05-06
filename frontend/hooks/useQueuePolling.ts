'use client';
import { useState, useEffect, useRef, useCallback } from 'react';
import { QueueEntry } from '@/types/api';
import { getQueueStatus } from '@/api/queue';

export interface UseQueuePollingResult {
  queueEntry: QueueEntry | null;
  isLoading: boolean;
  error: string | null;
  stopPolling: () => void;
}

/**
 * Poll queue status every 1 second.
 * Returns when status is 'locked' (with booking_id) or 'conflict' (with conflict_seats).
 */
export function useQueuePolling(
  queueId: string | null
): UseQueuePollingResult {
  const [queueEntry, setQueueEntry] = useState<QueueEntry | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const stoppedRef = useRef(false);

  const stopPolling = useCallback(() => {
    stoppedRef.current = true;
    if (intervalRef.current) {
      clearInterval(intervalRef.current);
      intervalRef.current = null;
    }
  }, []);

  useEffect(() => {
    if (!queueId) return;
    stoppedRef.current = false;
    setIsLoading(true);

    const poll = async () => {
      if (stoppedRef.current) return;
      try {
        const entry = await getQueueStatus(queueId);
        setQueueEntry(entry);
        setError(null);
        // Stop polling on terminal states
        if (entry.status === 'locked' || entry.status === 'conflict') {
          stopPolling();
        }
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Queue error');
        stopPolling();
      } finally {
        setIsLoading(false);
      }
    };

    poll();
    intervalRef.current = setInterval(poll, 1000);

    return () => stopPolling();
  }, [queueId, stopPolling]);

  return { queueEntry, isLoading, error, stopPolling };
}
