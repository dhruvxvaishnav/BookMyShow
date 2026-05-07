'use client';
import { useState, useEffect, useCallback, use } from 'react';
import { useRouter } from 'next/navigation';
import { CreditCard, X, AlertTriangle } from 'lucide-react';
import PageHeader from '@/components/layout/PageHeader';
import LockTimer from '@/components/booking/LockTimer';
import Modal from '@/components/layout/Modal';
import Button from '@/components/forms/Button';
import Badge from '@/components/common/Badge';
import { BookingSkeleton } from '@/components/common/LoadingSkeleton';
import EmptyState from '@/components/common/EmptyState';
import { useToast } from '@/components/layout/Toast';
import { getBooking, cancelBooking } from '@/api/bookings';
import { useRequireAuth } from '@/hooks/useRequireAuth';
import { getErrorMessage } from '@/utils/error';
import { formatPrice, formatDateTime } from '@/utils/format';
import type { Booking, BookingStatus } from '@/types/api';
import styles from './page.module.css';

interface PageProps { params: Promise<{ bookingId: string }> }

const statusBadge: Record<BookingStatus, 'success' | 'error' | 'warning' | 'info' | 'muted' | 'gold'> = {
  success: 'success',
  pending: 'warning',
  payment_pending: 'gold',
  expired: 'muted',
  cancelled: 'error',
  success_partial: 'warning',
  payment_failed: 'error',
  queued: 'info',
};

export default function BookingDetailsPage({ params }: PageProps) {
  const isAuthed = useRequireAuth();
  const { bookingId } = use(params);
  const router = useRouter();
  const toast = useToast();

  const [booking, setBooking] = useState<Booking | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [isCancelling, setIsCancelling] = useState(false);
  const [showCancelModal, setShowCancelModal] = useState(false);
  const [showExpiredModal, setShowExpiredModal] = useState(false);

  const loadBooking = useCallback(async () => {
    try {
      const data = await getBooking(bookingId);
      setBooking(data);
      if (data.status === 'expired') setShowExpiredModal(true);
    } catch (err) {
      setError(getErrorMessage(err));
    } finally {
      setIsLoading(false);
    }
  }, [bookingId]);

  useEffect(() => { loadBooking(); }, [loadBooking]);

  // Poll every 10 seconds
  useEffect(() => {
    if (!booking) return;
    const interval = setInterval(async () => {
      try {
        const data = await getBooking(bookingId);
        setBooking(data);
        if (data.status === 'expired') {
          setShowExpiredModal(true);
          clearInterval(interval);
        }
      } catch { /* silently fail */ }
    }, 10000);
    return () => clearInterval(interval);
  }, [booking, bookingId]);

  if (!isAuthed) return null;

  const handleCancel = async () => {
    setIsCancelling(true);
    try {
      await cancelBooking(bookingId);
      toast.showToast('Booking cancelled.', 'info');
      router.push('/');
    } catch (err) {
      toast.showToast(getErrorMessage(err), 'error');
    } finally {
      setIsCancelling(false);
      setShowCancelModal(false);
    }
  };

  if (isLoading) {
    return (
      <>
        <PageHeader title="Your Booking" />
        <div className="container"><BookingSkeleton /></div>
      </>
    );
  }

  if (error || !booking) {
    return (
      <>
        <PageHeader title="Your Booking" backHref="/" />
        <div className="container">
          <EmptyState
            title="Booking not found"
            description={error ?? 'This booking could not be found.'}
            icon="film"
            action={<button onClick={loadBooking}>Retry</button>}
          />
        </div>
      </>
    );
  }

  const isPending = booking.status === 'pending' || booking.status === 'payment_pending';
  const expired = booking.status === 'expired' || booking.status === 'cancelled';
  const badgeVariant = statusBadge[booking.status] ?? 'muted';

  return (
    <>
      <div className={`container ${styles.page}`}>
        <div className={styles.card}>
          {/* Card header */}
          <div className={styles.cardHeader}>
            <div>
              <div className={styles.bookingIdLabel}>Booking Reference</div>
              <div className={styles.bookingId}>{bookingId.slice(0, 16).toUpperCase()}</div>
              {booking.show_name && (
                <div style={{ fontSize: '0.75rem', color: 'var(--antique-gold)', marginTop: 4 }}>
                  {booking.show_name}
                </div>
              )}
            </div>
            <Badge variant={badgeVariant}>{booking.status}</Badge>
          </div>

          {/* Lock Timer */}
          {isPending && (
            <div className={styles.timerSection}>
              <LockTimer
                expiresAt={booking.expires_at}
                label="Lock expires in"
                sublabel={`${booking.seat_ids.length} seat${booking.seat_ids.length !== 1 ? 's' : ''} · ${formatPrice(booking.total_amount)}`}
              />
            </div>
          )}

          {/* Show details */}
          {booking.show && (
            <div className={styles.showSection}>
              <h2 className={styles.showName}>{booking.show.name}</h2>
              <p className={styles.showMeta}>
                {booking.show.theatre_name} · Screen {booking.show.screen_number}
              </p>
              <p className={styles.showTime}>{formatDateTime(booking.show.start_time)}</p>
            </div>
          )}

          <div className={styles.divider} />

          {/* Seats */}
          <div className={styles.seatsSection}>
            <span className={styles.seatsLabel}>Your Seats</span>
            <div className={styles.seatList}>
              {(booking.seat_numbers && booking.seat_numbers.length > 0
                ? booking.seat_numbers
                : booking.seat_ids.map((id) => id.slice(0, 8))
              ).map((label) => (
                <span key={label} className={styles.seatChip}>{label}</span>
              ))}
            </div>
          </div>

          <div className={styles.divider} />

          {/* Amount */}
          <div className={styles.amountSection}>
            <span>Total Amount</span>
            <span className={styles.amount}>{formatPrice(booking.total_amount)}</span>
          </div>

          {/* Actions */}
          {isPending && (
            <div className={styles.actions}>
              <Button
                variant="primary"
                onClick={() => router.push(`/bookings/${bookingId}/payment`)}
                leftIcon={<CreditCard size={16} strokeWidth={1.5} />}
              >
                Proceed to Payment
              </Button>
              <Button
                variant="danger"
                onClick={() => setShowCancelModal(true)}
                leftIcon={<X size={16} strokeWidth={1.5} />}
              >
                Cancel
              </Button>
            </div>
          )}

          {expired && (
            <div className={styles.actions}>
              <Button variant="primary" onClick={() => router.push('/')}>
                Browse Shows
              </Button>
            </div>
          )}
        </div>
      </div>

      {/* Cancel confirmation modal */}
      <Modal
        isOpen={showCancelModal}
        onClose={() => setShowCancelModal(false)}
        title="Cancel Booking?"
      >
        <div className={styles.modalBody}>
          <p>Are you sure you want to cancel this booking? Your seats will be released back to the pool.</p>
          <div className={styles.modalActions}>
            <Button variant="secondary" onClick={() => setShowCancelModal(false)}>
              Keep Booking
            </Button>
            <Button variant="danger" isLoading={isCancelling} onClick={handleCancel}>
              Yes, Cancel
            </Button>
          </div>
        </div>
      </Modal>

      {/* Expired modal */}
      <Modal
        isOpen={showExpiredModal}
        onClose={() => { setShowExpiredModal(false); router.push('/'); }}
        title="Lock Expired"
      >
        <div className={styles.modalBody}>
          <div className={styles.expiredIcon}>
            <AlertTriangle size={40} strokeWidth={1} />
          </div>
          <p>Your lock has expired and the seats have been released. Please select seats again.</p>
          <div className={styles.modalActions}>
            <Button variant="primary" onClick={() => router.push('/')}>
              Browse Shows
            </Button>
          </div>
        </div>
      </Modal>
    </>
  );
}
