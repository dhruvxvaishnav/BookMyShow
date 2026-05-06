'use client';
import { useCallback, useEffect, useMemo, useState } from 'react';
import Link from 'next/link';
import { ChevronLeft, ChevronRight, Search, SlidersHorizontal } from 'lucide-react';
import Badge from '@/components/common/Badge';
import { getAdminBookings } from '@/api/admin';
import { useRequireAdmin } from '@/hooks/useRequireAuth';
import { getErrorMessage } from '@/utils/error';
import { formatDateTime, formatPrice, shortId } from '@/utils/format';
import type { Booking, BookingStatus } from '@/types/api';
import styles from './page.module.css';

const PAGE_SIZE = 10;
const STATUSES: Array<'All' | BookingStatus> = [
  'All',
  'pending',
  'payment_pending',
  'success',
  'success_partial',
  'expired',
  'cancelled',
];

const statusVariant: Record<BookingStatus, 'success' | 'error' | 'warning' | 'muted' | 'gold'> = {
  success: 'success',
  pending: 'warning',
  payment_pending: 'gold',
  expired: 'muted',
  cancelled: 'error',
  success_partial: 'warning',
  payment_failed: 'error',
  queued: 'muted',
};

export default function AdminBookingsPage() {
  const isAdmin = useRequireAdmin();
  const [bookings, setBookings] = useState<Booking[]>([]);
  const [status, setStatus] = useState<'All' | BookingStatus>('All');
  const [query, setQuery] = useState('');
  const [page, setPage] = useState(0);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState('');

  const load = useCallback(async () => {
    setIsLoading(true);
    setError('');
    try {
      setBookings(await getAdminBookings());
    } catch (err) {
      setError(getErrorMessage(err));
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => { load(); }, [load]);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    return bookings.filter((booking) => {
      const matchesStatus = status === 'All' || booking.status === status;
      const showName = booking.show?.name ?? '';
      const theatre = booking.show?.theatre_name ?? '';
      const matchesQuery = !q || [
        booking.booking_id,
        booking.user_id,
        booking.show_id,
        showName,
        theatre,
        booking.seat_ids.join(' '),
      ].some((value) => value.toLowerCase().includes(q));
      return matchesStatus && matchesQuery;
    });
  }, [bookings, query, status]);

  const totalPages = Math.max(1, Math.ceil(filtered.length / PAGE_SIZE));
  const currentPage = Math.min(page, totalPages - 1);
  const paged = filtered.slice(currentPage * PAGE_SIZE, (currentPage + 1) * PAGE_SIZE);

  useEffect(() => { setPage(0); }, [query, status]);

  if (!isAdmin) return null;

  return (
    <div className={styles.page}>
      <header className={styles.header}>
        <div>
          <p className="marquee-label">Admin</p>
          <h1 className={styles.title}>All Bookings</h1>
          <p className={styles.subtitle}>{filtered.length} records in the ledger</p>
        </div>
        <button className={styles.refreshBtn} onClick={load}>Refresh</button>
      </header>

      <section className={styles.filterBar} aria-label="Booking filters">
        <div className={styles.searchWrap}>
          <Search size={15} strokeWidth={1.5} className={styles.searchIcon} />
          <input
            className={styles.searchInput}
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder="Search booking, user, show, seat..."
          />
        </div>
        <div className={styles.selectWrap}>
          <SlidersHorizontal size={15} strokeWidth={1.5} />
          <select
            className={styles.select}
            value={status}
            onChange={(event) => setStatus(event.target.value as 'All' | BookingStatus)}
            aria-label="Filter by status"
          >
            {STATUSES.map((item) => (
              <option key={item} value={item}>{formatStatus(item)}</option>
            ))}
          </select>
        </div>
      </section>

      {error && (
        <div className={styles.error} role="alert">
          <span>{error}</span>
          <button onClick={load}>Retry</button>
        </div>
      )}

      <section className={styles.tableShell}>
        <div className={styles.tableHead}>
          <span>Booking ID</span>
          <span>User</span>
          <span>Show</span>
          <span>Seats</span>
          <span>Amount</span>
          <span>Status</span>
          <span>Date</span>
        </div>

        {isLoading ? (
          Array.from({ length: 6 }, (_, index) => (
            <div key={index} className={`${styles.tableRow} ${styles.loadingRow}`}>
              {Array.from({ length: 7 }, (_, cell) => <span key={cell} className={styles.shimmer} />)}
            </div>
          ))
        ) : paged.length === 0 ? (
          <div className={styles.empty}>No bookings match the current filters.</div>
        ) : (
          paged.map((booking) => (
            <Link
              key={booking.booking_id}
              href={`/admin/bookings/${booking.booking_id}`}
              className={styles.tableRow}
            >
              <span className={styles.mono}>{shortId(booking.booking_id)}</span>
              <span className={styles.truncate}>{shortId(booking.user_id)}</span>
              <span className={styles.showCell}>
                <strong>{booking.show?.name ?? shortId(booking.show_id)}</strong>
                <small>{booking.show?.theatre_name ?? 'Show reference'}</small>
              </span>
              <span className={styles.seats}>{seatLabel(booking)}</span>
              <span className={styles.amount}>{formatPrice(booking.total_amount)}</span>
              <Badge variant={statusVariant[booking.status]}>{formatStatus(booking.status)}</Badge>
              <span className={styles.date}>{formatBookingDate(booking.created_at)}</span>
            </Link>
          ))
        )}

        <footer className={styles.pagination}>
          <span>
            Page {currentPage + 1} of {totalPages}
          </span>
          <div className={styles.pageBtns}>
            <button
              disabled={currentPage === 0}
              onClick={() => setPage((value) => Math.max(0, value - 1))}
              aria-label="Previous page"
            >
              <ChevronLeft size={16} strokeWidth={1.5} />
            </button>
            <button
              disabled={currentPage >= totalPages - 1}
              onClick={() => setPage((value) => Math.min(totalPages - 1, value + 1))}
              aria-label="Next page"
            >
              <ChevronRight size={16} strokeWidth={1.5} />
            </button>
          </div>
        </footer>
      </section>
    </div>
  );
}

function seatLabel(booking: Booking) {
  if (booking.seats?.length) return booking.seats.map((seat) => seat.seat_number).join(', ');
  return `${booking.seat_ids.length} seat${booking.seat_ids.length === 1 ? '' : 's'}`;
}

function formatStatus(value: string) {
  return value.replace(/([a-z])([A-Z])/g, '$1 $2');
}

function formatBookingDate(value: string) {
  const numeric = Number(value);
  if (Number.isFinite(numeric)) return formatDateTime(numeric);
  const parsed = Date.parse(value);
  if (Number.isNaN(parsed)) return value;
  return formatDateTime(Math.floor(parsed / 1000));
}
