'use client';
import type { Booking } from '@/types/api';
import { formatDateTime, formatPrice, formatSeatList, formatBookingId } from '@/utils/format';
import styles from './TicketDisplay.module.css';

interface TicketDisplayProps {
  booking: Booking;
}

export default function TicketDisplay({ booking }: TicketDisplayProps) {
  const show = booking.show;
  const seats = booking.seats ?? [];

  return (
    <div className={styles.ticket}>
      {/* Header */}
      <div className={styles.header}>
        <div className={styles.star} />
        <h2 className={styles.title}>Booking Confirmed!</h2>
        <div className={styles.star} />
      </div>

      {/* Movie info */}
      <div className={styles.movie}>
        <h3 className={styles.movieName}>{show?.name ?? 'Show'}</h3>
        <p className={styles.movieDetail}>
          {show?.theatre_name} · Screen {show?.screen_number}
        </p>
        <p className={styles.movieTime}>
          {show ? formatDateTime(show.start_time) : ''}
        </p>
      </div>

      <div className={styles.dashes}>
        <span className={styles.dashCircle} />
        <span className={styles.dashLine} />
        <span className={styles.dashCircle} />
      </div>

      {/* Seats */}
      <div className={styles.section}>
        <span className={styles.sectionLabel}>Seats</span>
        <span className={styles.seatList}>
          {seats.length > 0 ? formatSeatList(seats.map((s) => s.seat_number)) : formatSeatList(booking.seat_ids)}
        </span>
      </div>

      <div className={styles.dashes}>
        <span className={styles.dashCircle} />
        <span className={styles.dashLine} />
        <span className={styles.dashCircle} />
      </div>

      {/* Payment details */}
      <div className={styles.details}>
        <div className={styles.detailRow}>
          <span>Amount Paid</span>
          <span className={styles.amount}>{formatPrice(booking.total_amount)}</span>
        </div>
        <div className={styles.detailRow}>
          <span>Booking ID</span>
          <span className={styles.bookingId}>{formatBookingId(booking.booking_id)}</span>
        </div>
        {booking.payment_id && (
          <div className={styles.detailRow}>
            <span>Payment ID</span>
            <span className={styles.paymentId}>{booking.payment_id.slice(0, 12)}...</span>
          </div>
        )}
      </div>

      <div className={styles.footer}>
        <p>Thank you for booking with BookMyShow</p>
      </div>
    </div>
  );
}
