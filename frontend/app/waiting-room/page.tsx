'use client';
import { useEffect, useMemo, useState } from 'react';
import { useRouter } from 'next/navigation';
import { Clock, DoorOpen, Film, Ticket } from 'lucide-react';
import Button from '@/components/forms/Button';
import { useQueuePolling } from '@/hooks/useQueuePolling';
import { leaveQueue } from '@/api/queue';
import { getShow } from '@/api/shows';
import { useRequireAuth } from '@/hooks/useRequireAuth';
import { useToast } from '@/components/layout/Toast';
import { formatDateTime } from '@/utils/format';
import type { Show } from '@/types/api';
import styles from './page.module.css';

export default function WaitingRoomPage() {
  const isAuthed = useRequireAuth();
  const router = useRouter();
  const toast = useToast();
  const [queueId, setQueueId] = useState<string | null>(null);
  const [showId, setShowId] = useState<string | null>(null);
  const [show, setShow] = useState<Show | null>(null);
  const [isLeaving, setIsLeaving] = useState(false);
  const { queueEntry, isLoading, error, stopPolling } = useQueuePolling(queueId);

  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    setQueueId(params.get('queueId'));
    setShowId(params.get('showId'));
  }, []);

  useEffect(() => {
    if (!showId) return;
    getShow(showId).then(setShow).catch(() => setShow(null));
  }, [showId]);

  useEffect(() => {
    if (!queueEntry) return;
    if (queueEntry.status === 'locked' && queueEntry.booking_id) {
      toast.showToast('Your seats are ready.', 'success');
      router.replace(`/bookings/${queueEntry.booking_id}`);
    }
    if (queueEntry.status === 'conflict') {
      toast.showToast('Selected seats are no longer available.', 'warning');
    }
  }, [queueEntry, router, toast]);

  const position = queueEntry?.position ?? null;
  const status = queueEntry?.status ?? 'waiting';

  const statusText = useMemo(() => {
    if (!queueId) return 'No queue entry found';
    if (status === 'processing') return 'Preparing your lock';
    if (status === 'locked') return 'Seats ready';
    if (status === 'conflict') return 'Seat conflict detected';
    if (status === 'expired') return 'Queue entry expired';
    return 'Waiting for your turn';
  }, [queueId, status]);

  if (!isAuthed) return null;

  const handleLeave = async () => {
    if (!queueId) {
      router.push(showId ? `/shows/${showId}` : '/movies');
      return;
    }
    setIsLeaving(true);
    try {
      stopPolling();
      await leaveQueue(queueId);
      toast.showToast('You left the queue.', 'info');
      router.push(showId ? `/shows/${showId}` : '/movies');
    } catch {
      router.push(showId ? `/shows/${showId}` : '/movies');
    }
  };

  return (
    <main className={styles.page}>
      <section className={styles.panel}>
        <div className={styles.goldBar} />
        <div className={styles.content}>
          <div className={styles.iconWrap}>
            <Clock size={42} strokeWidth={1.2} />
          </div>

          <p className="marquee-label">Waiting Room</p>
          <h1 className={styles.title}>{statusText}</h1>

          {show && (
            <div className={styles.showCard}>
              <Film size={16} strokeWidth={1.5} />
              <div>
                <strong>{show.name}</strong>
                <span>{show.theatre_name} / Screen {show.screen_number} / {formatDateTime(show.start_time)}</span>
              </div>
            </div>
          )}

          <div className={styles.positionPanel} aria-live="polite">
            <span className={styles.positionLabel}>Queue Position</span>
            <span className={styles.positionValue}>{position ?? (isLoading ? '...' : '-')}</span>
            <span className={styles.statusBadge}>{status}</span>
          </div>

          <div className={styles.progressTrack} aria-hidden="true">
            <div
              className={styles.progressFill}
              style={{ width: `${position ? Math.max(12, 100 - Math.min(position, 20) * 4) : 20}%` }}
            />
          </div>

          {error && <p className={styles.error}>{error}</p>}
          {status === 'conflict' && (
            <p className={styles.error}>
              Seats affected: {queueEntry?.conflict_seats?.join(', ') || 'selected seats'}
            </p>
          )}

          <div className={styles.actions}>
            {status === 'conflict' ? (
              <Button
                variant="primary"
                onClick={() => router.push(showId ? `/shows/${showId}` : '/movies')}
                leftIcon={<Ticket size={16} strokeWidth={1.5} />}
              >
                Select Again
              </Button>
            ) : (
              <Button
                variant="danger"
                onClick={handleLeave}
                isLoading={isLeaving}
                leftIcon={<DoorOpen size={16} strokeWidth={1.5} />}
              >
                Leave Queue
              </Button>
            )}
          </div>
        </div>
      </section>
    </main>
  );
}
