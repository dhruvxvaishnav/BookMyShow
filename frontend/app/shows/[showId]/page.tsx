'use client';
import { useState, useEffect, useCallback, use } from 'react';
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
import { getShowExperience, SHOW_EXPERIENCE_LABELS } from '@/utils/showExperience';
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
  const [ticketCount, setTicketCount] = useState<number | null>(null);
  const [isTicketPickerOpen, setIsTicketPickerOpen] = useState(true);
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
    if (!ticketCount) {
      setIsTicketPickerOpen(true);
      toast.showToast('Choose how many tickets you want first.', 'info');
      return;
    }

    if (seat.status === 'booked') {
      toast.showToast(`Seat ${seat.seat_number} is already booked.`, 'warning');
      return;
    }
    if (seat.status === 'locked' &&
        !selectedSeatIds.includes(seat.seat_id) &&
        !seats.some(s => s.seat_id === seat.seat_id && s.status === 'locked' && selectedSeatIds.includes(s.seat_id))) {
      toast.showToast(`Seat ${seat.seat_number} is currently held by another user.`, 'warning');
      return;
    }

    setSelectedSeatIds((prev) => {
      if (prev.includes(seat.seat_id)) {
        const next = prev.filter((id) => id !== seat.seat_id);
        setSrAnnouncement(`Seat ${seat.seat_number} deselected. ${next.length} of ${ticketCount} selected.`);
        return next;
      }

      const replaceSelection = prev.length >= ticketCount;
      const baseSelection = replaceSelection ? [] : prev;
      const remaining = replaceSelection ? ticketCount : ticketCount - prev.length;
      const autoSelectedSeatIds = getAutoSelectedSeatIds(seat, seats, baseSelection, remaining);

      if (autoSelectedSeatIds.length === 0) {
        toast.showToast('No adjacent seats available from this spot.', 'warning');
        return prev;
      }

      const next = [...baseSelection, ...autoSelectedSeatIds].slice(0, ticketCount);
      const remainingAfterSelection = ticketCount - next.length;
      setSrAnnouncement(
        remainingAfterSelection > 0
          ? `${next.length} of ${ticketCount} selected. Choose ${remainingAfterSelection} more.`
          : `${ticketCount} seats selected.`
      );
      return next;
    });
    setConflictingSeats([]);
  };

  const handleTicketCountSelect = (count: number) => {
    setTicketCount(count);
    setSelectedSeatIds((prev) => prev.slice(0, count));
    setIsTicketPickerOpen(false);
    setSrAnnouncement(`${count} ticket${count !== 1 ? 's' : ''} selected.`);
  };

  const selectedSeats = seats.filter((s) => selectedSeatIds.includes(s.seat_id));
  const total = selectedSeats.reduce((sum, s) => sum + s.price, 0);
  const count = selectedSeats.length;
  const experience = show ? getShowExperience(show) : 'normal';
  const remainingTickets = ticketCount ? Math.max(ticketCount - count, 0) : 0;

  const breakdown = selectedSeats.reduce((acc, seat) => {
    if (!acc[seat.seat_type]) acc[seat.seat_type] = { count: 0, price: seat.price };
    acc[seat.seat_type].count += 1;
    return acc;
  }, {} as Record<string, { count: number; price: number }>);

  const handleLock = async () => {
    if (!ticketCount) {
      setIsTicketPickerOpen(true);
      return;
    }
    if (selectedSeatIds.length !== ticketCount) return;
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
          <span className={`${styles.showMetaItem} ${styles.showFormat}`}>{SHOW_EXPERIENCE_LABELS[experience]}</span>
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

      {isTicketPickerOpen && (
        <div className={styles.ticketPickerBackdrop} role="presentation">
          <section
            className={styles.ticketPicker}
            role="dialog"
            aria-modal="true"
            aria-labelledby="ticket-picker-title"
          >
            <span className={styles.ticketPickerEyebrow}>Tickets</span>
            <h2 id="ticket-picker-title" className={styles.ticketPickerTitle}>How many seats?</h2>
            <div className={styles.ticketCountGrid} role="list" aria-label="Choose ticket count">
              {Array.from({ length: 10 }, (_, index) => index + 1).map((value) => (
                <button
                  key={value}
                  className={`${styles.ticketCountButton} ${ticketCount === value ? styles.ticketCountButtonActive : ''}`}
                  onClick={() => handleTicketCountSelect(value)}
                  aria-pressed={ticketCount === value}
                >
                  {value}
                </button>
              ))}
            </div>
          </section>
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
          experience={experience}
        />
      </div>

      {/* Sticky bottom bar */}
      <div className={styles.stickyBar} role="region" aria-label="Seat selection summary">
        {/* Left: selected seat chips */}
        <div className={styles.stickyBarLeft}>
          <button
            className={styles.ticketCountControl}
            onClick={() => setIsTicketPickerOpen(true)}
            aria-label="Change ticket count"
          >
            {ticketCount ? `${ticketCount} ticket${ticketCount !== 1 ? 's' : ''}` : 'Choose tickets'}
          </button>
          {count === 0 ? (
            <span className={styles.stickyEmpty}>
              {ticketCount ? `Select ${ticketCount} seat${ticketCount !== 1 ? 's' : ''}` : 'Choose tickets to continue'}
            </span>
          ) : (
            <>
              <span className={styles.stickyBarLabel}>
                {remainingTickets > 0 ? `${remainingTickets} more` : 'Selected:'}
              </span>
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
              <span className={styles.breakdownType}>{data.count}×&nbsp;{formatSeatType(type)}</span>
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
            disabled={!ticketCount || count !== ticketCount || isLocking}
            aria-busy={isLocking}
          >
            <Lock size={15} strokeWidth={2} />
            {isLocking ? 'Locking…' : `Lock Seats${ticketCount ? ` (${count}/${ticketCount})` : ''}`}
          </button>
        </div>
      </div>
    </div>
  );
}

function formatSeatType(type: string) {
  if (type === 'premium') return 'Comfort';
  return type.charAt(0).toUpperCase() + type.slice(1);
}

function getAutoSelectedSeatIds(
  clickedSeat: Seat,
  seats: Seat[],
  currentSelection: string[],
  limit: number
) {
  const currentSelectionSet = new Set(currentSelection);
  const rowSeats = seats
    .filter((seat) => seat.row_label === clickedSeat.row_label)
    .sort((a, b) => a.seat_number.localeCompare(b.seat_number, undefined, { numeric: true }));
  const clickedIndex = rowSeats.findIndex((seat) => seat.seat_id === clickedSeat.seat_id);
  if (clickedIndex < 0) return [];

  const selected: Seat[] = [];

  for (let index = clickedIndex; index < rowSeats.length && selected.length < limit; index += 1) {
    const seat = rowSeats[index];
    if (!isAutoSelectableSeat(seat, currentSelectionSet)) break;
    selected.push(seat);
  }

  for (let index = clickedIndex - 1; index >= 0 && selected.length < limit; index -= 1) {
    const seat = rowSeats[index];
    if (!isAutoSelectableSeat(seat, currentSelectionSet)) break;
    selected.unshift(seat);
  }

  return selected.map((seat) => seat.seat_id);
}

function isAutoSelectableSeat(seat: Seat, currentSelectionSet: Set<string>) {
  return seat.status === 'available' && !currentSelectionSet.has(seat.seat_id);
}
