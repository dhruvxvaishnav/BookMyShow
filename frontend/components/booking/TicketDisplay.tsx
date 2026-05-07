'use client';
import { QRCodeSVG } from 'qrcode.react';
import type { Booking } from '@/types/api';
import { formatDateTime, formatPrice, formatBookingId } from '@/utils/format';
import styles from './TicketDisplay.module.css';

interface TicketDisplayProps {
  booking: Booking;
}

export default function TicketDisplay({ booking }: TicketDisplayProps) {
  const show = booking.show;
  const seats = booking.seats ?? [];
  const ticketCount = booking.seat_numbers?.length || seats.length || booking.seat_ids.length;
  const movieName = show?.movie?.title ?? booking.show_name ?? show?.name ?? 'Movie';
  const venueName = show?.venue?.name ?? show?.theatre_name ?? 'Venue unavailable';
  const screenNumber = show?.screen_number ?? '-';
  const seatLabels = seats.length > 0
    ? seats.map((seat) => seat.seat_number)
    : booking.seat_numbers && booking.seat_numbers.length > 0
      ? booking.seat_numbers
      : booking.seat_ids.map((seatId) => seatId.slice(0, 8));
  const sortedSeatLabels = [...seatLabels].sort((a, b) =>
    a.localeCompare(b, undefined, { numeric: true })
  );

  return (
    <div className={styles.ticket}>
      <div className={styles.accentStripe} />

      <div className={styles.mainStub}>
        <div className={styles.header}>
          <p className={styles.kicker}>{formatAdmitLabel(ticketCount)}</p>
          <h2 className={styles.movieName}>{movieName}</h2>
        </div>

        <div className={styles.ticketGrid}>
          <div>
            <span className={styles.sectionLabel}>Venue</span>
            <p>{venueName} · Screen {screenNumber}</p>
          </div>
          <div>
            <span className={styles.sectionLabel}>Show Time</span>
            <p className={styles.mono}>{show ? formatDateTime(show.start_time) : '-'}</p>
          </div>
        </div>

        <div className={styles.seatSection}>
          <span className={styles.sectionLabel}>Seats</span>
          <div className={styles.seatChips}>
            {sortedSeatLabels.map((seat) => (
              <span key={seat} className={styles.seatChip}>{seat}</span>
            ))}
          </div>
        </div>

        <div className={styles.detailFooter}>
          <div>
            <span className={styles.sectionLabel}>Amount Paid</span>
            <p className={styles.amount}>{formatPrice(booking.total_amount)}</p>
          </div>
          <div>
            <span className={styles.sectionLabel}>Booking Ref</span>
            <p className={styles.bookingId}>{formatBookingId(booking.booking_id)}</p>
          </div>
        </div>
      </div>

      <div className={styles.perforation} aria-hidden="true" />

      <div className={styles.qrStub}>
        <QRCodeSVG
          value={booking.booking_id}
          size={112}
          bgColor="transparent"
          fgColor="#F5A623"
          level="M"
        />
        <span className={styles.qrLabel}>Entry Code</span>
        <span className={styles.qrRef}>{formatBookingId(booking.booking_id)}</span>
        {booking.payment_id && (
          <span className={styles.paymentId}>{booking.payment_id.slice(0, 12)}...</span>
        )}
      </div>
    </div>
  );
}

function formatAdmitLabel(count: number) {
  const words = ['Zero', 'One', 'Two', 'Three', 'Four', 'Five', 'Six', 'Seven', 'Eight', 'Nine', 'Ten'];
  return `Admit ${words[count] ?? count}`;
}
