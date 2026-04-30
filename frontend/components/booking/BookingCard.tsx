'use client';
import Link from 'next/link';
import type { Booking } from '@/types/api';
import Badge from '@/components/common/Badge';
import { formatDateTime, formatPrice, formatSeatList } from '@/utils/format';
import styles from './BookingCard.module.css';

interface BookingCardProps {
  booking: Booking;
}

const statusVariant: Record<string, 'success' | 'error' | 'warning' | 'muted' | 'gold' | 'info'> = {
  Success: 'success',
  Pending: 'warning',
  PaymentPending: 'gold',
  Expired: 'muted',
  Cancelled: 'error',
  PartialSuccess: 'warning',
};

export default function BookingCard({ booking }: BookingCardProps) {
  const variant = statusVariant[booking.status] ?? 'muted';

  return (
    <div className={styles.card}>
      <div className={styles.top}>
        <div className={styles.info}>
          {booking.show ? (
            <>
              <h3 className={styles.showName}>{booking.show.name}</h3>
              <p className={styles.theatre}>
                {booking.show.theatre_name} · Screen {booking.show.screen_number}
              </p>
              <p className={styles.time}>{formatDateTime(booking.show.start_time)}</p>
            </>
          ) : (
            <h3 className={styles.showName}>Show #{booking.show_id.slice(0, 8)}</h3>
          )}
        </div>
        <Badge variant={variant}>{booking.status}</Badge>
      </div>

      <div className={styles.seats}>
        <span className={styles.seatLabel}>Seats:</span>
        <span className={styles.seatList}>
          {booking.seats && booking.seats.length > 0
            ? formatSeatList(booking.seats.map((s) => s.seat_number))
            : `${booking.seat_ids.length} seat${booking.seat_ids.length !== 1 ? 's' : ''}`}
        </span>
      </div>

      <div className={styles.bottom}>
        <span className={styles.amount}>{formatPrice(booking.total_amount)}</span>
        <Link href={`/bookings/${booking.booking_id}`} className={styles.viewBtn}>
          View Details
        </Link>
      </div>
    </div>
  );
}
