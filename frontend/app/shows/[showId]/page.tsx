'use client';
import { useState, useEffect, useCallback, use } from 'react';
import { useRouter } from 'next/navigation';
import PageHeader from '@/components/layout/PageHeader';
import SeatGrid from '@/components/seats/SeatGrid';
import SeatSelectionPanel from '@/components/booking/SeatSelectionPanel';
import QueueStatusBanner from '@/components/booking/QueueStatusBanner';
import { SeatGridSkeleton } from '@/components/common/LoadingSkeleton';
import EmptyState from '@/components/common/EmptyState';
import { useToast } from '@/components/layout/Toast';
import { useUserId } from '@/hooks/useUserId';
import { getShow, getSeatLayout } from '@/api/shows';
import { lockSeats, extendLock } from '@/api/bookings';
import { joinQueue } from '@/api/queue';
import { getConflictingSeats, getErrorMessage } from '@/utils/error';
import { ApiError } from '@/types/api';
import type { Show, Seat, SeatLayoutResponse } from '@/types/api';
import styles from './page.module.css';

interface PageProps { params: Promise<{ showId: string }> }

export default function SeatSelectionPage({ params }: PageProps) {
  const { showId } = use(params);
  const router = useRouter();
  const toast = useToast();
  const userId = useUserId();

  const [show, setShow] = useState<Show | null>(null);
  const [seats, setSeats] = useState<Seat[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedSeatIds, setSelectedSeatIds] = useState<string[]>([]);
  const [isLocking, setIsLocking] = useState(false);
  const [conflictingSeats, setConflictingSeats] = useState<string[]>([]);

  // Queue state
  const [queueId, setQueueId] = useState<string | null>(null);
  const [queuePosition, setQueuePosition] = useState<number | null>(null);
  const [queueProcessing, setQueueProcessing] = useState(false);

  const loadSeatLayout = useCallback(async () => {
    try {
      const layout = await getSeatLayout(showId);
      setSeats(layout.seats);
    } catch {
      // Non-fatal — keep existing seats
    }
  }, [showId]);

  const loadShow = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const [showData, layoutData] = await Promise.all([
        getShow(showId),
        getSeatLayout(showId),
      ]);
      setShow(showData);
      setSeats(layoutData.seats);
    } catch (err) {
      setError(getErrorMessage(err));
    } finally {
      setIsLoading(false);
    }
  }, [showId]);

  useEffect(() => { loadShow(); }, [loadShow]);

  // Poll seat layout every 5 seconds
  useEffect(() => {
    const interval = setInterval(loadSeatLayout, 5000);
    return () => clearInterval(interval);
  }, [loadSeatLayout]);

  const handleSeatClick = (seat: Seat) => {
    if (seat.status === 'Booked') {
      toast.showToast(`Seat ${seat.seat_number} is already booked.`, 'warning');
      return;
    }
    if (seat.status === 'Locked' &&
        !selectedSeatIds.includes(seat.seat_id) &&
        !seats.some(s => s.seat_id === seat.seat_id && s.status === 'Locked' && selectedSeatIds.includes(s.seat_id))) {
      // Check if it's locked by the current user
      toast.showToast(`Seat ${seat.seat_number} is currently held by another user.`, 'warning');
      return;
    }

    setSelectedSeatIds((prev) => {
      if (prev.includes(seat.seat_id)) {
        return prev.filter((id) => id !== seat.seat_id);
      }
      if (prev.length >= 10) {
        toast.showToast('Maximum 10 seats per booking.', 'warning');
        return prev;
      }
      return [...prev, seat.seat_id];
    });
    setConflictingSeats([]);
  };

  const selectedSeats = seats.filter((s) => selectedSeatIds.includes(s.seat_id));

  const handleLock = async () => {
    if (selectedSeatIds.length === 0) return;
    setIsLocking(true);
    setConflictingSeats([]);
    try {
      const booking = await lockSeats(showId, selectedSeatIds);
      toast.showToast('Seats locked successfully!', 'success');
      router.push(`/bookings/${booking.booking_id}`);
    } catch (err) {
      if (err instanceof ApiError && err.code === 'SEATS_UNAVAILABLE') {
        const conflicts = getConflictingSeats(err);
        setConflictingSeats(conflicts);
        toast.showToast(err.message, 'warning');
      } else if (err instanceof ApiError && err.code === 'RATE_LIMIT_EXCEEDED') {
        toast.showToast(getErrorMessage(err), 'warning');
      } else if (err instanceof ApiError && err.code === 'QUEUE_REQUIRED') {
        // Server returned queue entry — join queue
        toast.showToast('Too many concurrent locks. Joining queue...', 'info');
        const entry = await joinQueue(showId, selectedSeatIds);
        setQueueId(entry.queue_id);
        setQueuePosition(entry.position);
      } else {
        toast.showToast(getErrorMessage(err), 'error');
      }
    } finally {
      setIsLocking(false);
    }
  };

  if (isLoading) {
    return (
      <>
        <PageHeader
          title="Select Seats"
          backHref="/"
          subtitle="Loading seat layout..."
        />
        <div className="container">
          <SeatGridSkeleton />
        </div>
      </>
    );
  }

  if (error || !show) {
    return (
      <>
        <PageHeader title="Select Seats" backHref="/" />
        <div className="container">
          <EmptyState
            title="Unable to load seats"
            description={error ?? 'Show not found.'}
            icon="clapperboard"
            action={<button onClick={loadShow}>Try again</button>}
          />
        </div>
      </>
    );
  }

  return (
    <>
      <PageHeader
        title={show.name}
        subtitle={`${show.theatre_name} · Screen ${show.screen_number}`}
        backHref="/"
      />

      <div className="container">
        {queueId && (
          <div className={styles.queueBannerWrap}>
            <QueueStatusBanner
              position={queuePosition}
              status="Waiting"
              isProcessing={queueProcessing}
            />
          </div>
        )}

        <div className={styles.layout}>
          <div className={styles.gridWrap}>
            <SeatGrid
              seats={seats}
              selectedSeatIds={selectedSeatIds}
              lockedByYouSeatIds={[]}
              conflictingSeats={conflictingSeats}
              userId={userId}
              onSeatClick={handleSeatClick}
            />
          </div>

          <div className={styles.panelWrap}>
            <SeatSelectionPanel
              selectedSeats={selectedSeats}
              show={show}
              onLock={handleLock}
              isLocking={isLocking}
            />
          </div>
        </div>
      </div>
    </>
  );
}