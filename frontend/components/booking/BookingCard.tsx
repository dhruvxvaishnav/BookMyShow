'use client';
import Link from 'next/link';
import { QRCodeSVG } from 'qrcode.react';
import type { Booking } from '@/types/api';
import Badge from '@/components/common/Badge';
import { formatDateTime, formatPrice, formatSeatList } from '@/utils/format';
import styles from './BookingCard.module.css';

interface BookingCardProps {
  booking: Booking;
}

const statusVariant: Record<string, 'success' | 'error' | 'warning' | 'muted' | 'gold' | 'info'> = {
  success: 'success',
  pending: 'warning',
  payment_pending: 'gold',
  expired: 'muted',
  cancelled: 'error',
  success_partial: 'warning',
  payment_failed: 'error',
  queued: 'info',
};

export default function BookingCard({ booking }: BookingCardProps) {
  const variant = statusVariant[booking.status] ?? 'muted';
  const isConfirmed = booking.status === 'success';
  const posterUrl = booking.show?.movie?.poster_url ?? null;

  return (
    <div className={styles.card}>
      <div className={styles.top}>
        {posterUrl && (
          <div className={styles.poster} aria-hidden="true">
            <img src={posterUrl} alt="" className={styles.posterImg} />
          </div>
        )}

        <div className={styles.info}>
          {booking.show ? (
            <>
              <h3 className={styles.showName}>{booking.show.name}</h3>
              <p className={styles.theatre}>
                {booking.show.venue?.name ?? booking.show.theatre_name} · Screen {booking.show.screen_number}
              </p>
              {booking.show.venue?.address && (
                <p className={styles.address}>{booking.show.venue.address}</p>
              )}
              <p className={styles.time}>{formatDateTime(booking.show.start_time)}</p>
            </>
          ) : booking.show_name ? (
            <h3 className={styles.showName}>{booking.show_name}</h3>
          ) : (
            <h3 className={styles.showName}>Show #{booking.show_id.slice(0, 8)}</h3>
          )}
          {booking.seat_numbers && booking.seat_numbers.length > 0 && (
            <p className={styles.theatre}>Seats: {booking.seat_numbers.join(', ')}</p>
          )}
        </div>
        <div className={styles.rightCol}>
          <Badge variant={variant}>{booking.status}</Badge>
          {isConfirmed && (
            <div className={styles.qrWrap} title="Scan at entry">
              <QRCodeSVG
                value={booking.booking_id}
                size={64}
                bgColor="transparent"
                fgColor="#F5A623"
                level="M"
              />
            </div>
          )}
        </div>
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
        <div className={styles.actions}>
          {isConfirmed && (
            <Link href={`/bookings/${booking.booking_id}/confirmed`} className={styles.downloadBtn}>
              Download Ticket
            </Link>
          )}
          <Link href={`/bookings/${booking.booking_id}`} className={styles.viewBtn}>
            View Details
          </Link>
        </div>
      </div>
    </div>
  );
}
