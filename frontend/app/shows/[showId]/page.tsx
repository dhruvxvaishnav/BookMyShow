'use client';
import { useState, useEffect, useCallback, use, useRef } from 'react';
import { useRouter } from 'next/navigation';
import SeatGrid from '@/components/seats/SeatGrid';
import QueueStatusBanner from '@/components/booking/QueueStatusBanner';
import { SeatGridSkeleton } from '@/components/common/LoadingSkeleton';
import EmptyState from '@/components/common/EmptyState';
import { useToast } from '@/components/layout/Toast';
import { useUserId } from '@/hooks/useUserId';
import { getShow, getSeatLayout } from '@/api/shows';
import { lockSeats } from '@/api/bookings';
import { joinQueue } from '@/api/queue';
import { getConflictingSeats, getErrorMessage } from '@/utils/error';
import { ApiError } from '@/types/api';
import { formatPrice, formatTime, formatDate } from '@/utils/format';
import { Lock } from 'lucide-react';
import type { Show, Seat } from '@/types/api';
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
  const [srAnnouncement, setSrAnnouncement] = useState('');

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
      toast.showToast(`Seat ${seat.seat_number} is currently held by another user.`, 'warning');
      return;
    }

    setSelectedSeatIds((prev) => {
      if (prev.includes(seat.seat_id)) {
        setSrAnnouncement(`Seat ${seat.seat_number} deselected. ${prev.length - 1} seat${prev.length - 1 !== 1 ? 's' : ''} selected.`);
        return prev.filter((id) => id !== seat.seat_id);
      }
      if (prev.length >= 10) {
        toast.showToast('Maximum 10 seats per booking.', 'warning');
        setSrAnnouncement('Maximum 10 seats per booking reached.');
        return prev;
      }
      setSrAnnouncement(`Seat ${seat.seat_number} selected. ${prev.length + 1} seat${prev.length + 1 !== 1 ? 's' : ''} selected.`);
      return [...prev, seat.seat_id];
    });
    setConflictingSeats([]);
  };

  const selectedSeats = seats.filter((s) => selectedSeatIds.includes(s.seat_id));
  const total = selectedSeats.reduce((sum, s) => sum + s.price, 0);
  const count = selectedSeats.length;

  const breakdown = selectedSeats.reduce((acc, seat) => {
    if (!acc[seat.seat_type]) acc[seat.seat_type] = { count: 0, price: seat.price };
    acc[seat.seat_type].count += 1;
    return acc;
  }, {} as Record<string, { count: number; price: number }>);

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
        toast.showToast('Too many concurrent locks. Joining queue...', 'info');
        const entry = await joinQueue(showId, selectedSeatIds);
        setQueueId(entry.queue_id);
        setQueuePosition(entry.position);
        router.push(`/waiting-room?queueId=${encodeURIComponent(entry.queue_id)}&showId=${encodeURIComponent(showId)}`);
      } else {
        toast.showToast(getErrorMessage(err), 'error');
      }
    } finally {
      setIsLocking(false);
    }
  };

  if (isLoading) {
    return (
      <div className={styles.pageWrapper}>
        <div className={styles.headerBar}>
          <div className={styles.showTitle}>Select Seats</div>
          <div className={styles.showMeta}>
            <span className={styles.showMetaItem}>Loading seat layout…</span>
          </div>
        </div>
        <div className="container">
          <SeatGridSkeleton />
        </div>
      </div>
    );
  }

  if (error || !show) {
    return (
      <div className={styles.pageWrapper}>
        <div className={styles.headerBar}>
          <div className={styles.showTitle}>Select Seats</div>
        </div>
        <div className="container">
          <EmptyState
            title="Unable to load seats"
            description={error ?? 'Show not found.'}
            icon="clapperboard"
            action={<button onClick={loadShow}>Try again</button>}
          />
        </div>
      </div>
    );
  }

  return (
    <div className={styles.pageWrapper}>
      {/* Screen reader live region */}
      <div aria-live="polite" aria-atomic="true" className={styles.srOnly}>
        {srAnnouncement}
      </div>

      {/* Header bar */}
      <div className={styles.headerBar}>
        <h1 className={styles.showTitle}>{show.name}</h1>
        <div className={styles.showMeta}>
          <span className={styles.showMetaItem}>{formatDate(show.start_time)}</span>
          <span className={styles.showMetaDot} />
          <span className={styles.showMetaItem}>{formatTime(show.start_time)}</span>
          <span className={styles.showMetaDot} />
          <span className={styles.showMetaItem}>{show.theatre_name}</span>
          <span className={styles.showMetaDot} />
          <span className={styles.showMetaItem}>Screen {show.screen_number}</span>
        </div>
      </div>

      {/* Queue banner */}
      {queueId && (
        <div className={styles.queueBannerWrap}>
          <QueueStatusBanner
            position={queuePosition}
            status="Waiting"
            isProcessing={queueProcessing}
          />
        </div>
      )}

      {/* Seat grid */}
      <div className={styles.gridArea}>
        <SeatGrid
          seats={seats}
          selectedSeatIds={selectedSeatIds}
          lockedByYouSeatIds={[]}
          conflictingSeats={conflictingSeats}
          userId={userId}
          onSeatClick={handleSeatClick}
        />
      </div>

      {/* Sticky bottom bar */}
      <div className={styles.stickyBar} role="region" aria-label="Seat selection summary">
        {/* Left: selected seat chips */}
        <div className={styles.stickyBarLeft}>
          {count === 0 ? (
            <span className={styles.stickyEmpty}>Select seats to continue</span>
          ) : (
            <>
              <span className={styles.stickyBarLabel}>Selected:</span>
              {selectedSeats.map((s) => (
                <span key={s.seat_id} className={styles.seatChip}>{s.seat_number}</span>
              ))}
            </>
          )}
        </div>

        {/* Center: type breakdown */}
        <div className={styles.stickyBarCenter} aria-hidden="true">
          {Object.entries(breakdown).map(([type, data]) => (
            <span key={type} className={styles.breakdownChip}>
              <span className={styles.breakdownType}>{data.count}×&nbsp;{type}</span>
              {' '}{formatPrice(data.price * data.count)}
            </span>
          ))}
        </div>

        {/* Right: total + lock button */}
        <div className={styles.stickyBarRight}>
          <div className={styles.totalLabel}>
            <span className={styles.totalCaption}>Total</span>
            <span className={styles.totalAmount}>{count > 0 ? formatPrice(total) : '—'}</span>
          </div>
          <button
            className={styles.lockBtn}
            onClick={handleLock}
            disabled={count === 0 || isLocking}
            aria-busy={isLocking}
          >
            <Lock size={15} strokeWidth={2} />
            {isLocking ? 'Locking…' : `Lock Seats${count > 0 ? ` (${count})` : ''}`}
          </button>
        </div>
      </div>
    </div>
  );
}
