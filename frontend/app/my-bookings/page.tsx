'use client';
import { useState, useEffect, useCallback } from 'react';
import { ChevronLeft, ChevronRight } from 'lucide-react';
import Link from 'next/link';
import BookingCard from '@/components/booking/BookingCard';
import EmptyState from '@/components/common/EmptyState';
import { BookingSkeleton } from '@/components/common/LoadingSkeleton';
import { useUserId } from '@/hooks/useUserId';
import { useRequireAuth } from '@/hooks/useRequireAuth';
import { getUserBookings } from '@/api/bookings';
import { getErrorMessage } from '@/utils/error';
import type { Booking } from '@/types/api';
import styles from './page.module.css';

type FilterTab = 'Upcoming' | 'Past';

const TABS: FilterTab[] = ['Upcoming', 'Past'];
const PAGE_SIZE = 9;

function isUpcoming(booking: Booking): boolean {
  if (booking.show?.start_time) {
    return booking.show.start_time > Date.now() / 1000;
  }
  return booking.status === 'pending' || booking.status === 'payment_pending';
}

export default function MyBookingsPage() {
  const isAuthed = useRequireAuth();
  const userId = useUserId();
  const [bookings, setBookings] = useState<Booking[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<FilterTab>('Upcoming');
  const [page, setPage] = useState(1);

  const loadBookings = useCallback(async () => {
    if (!userId) return;
    setIsLoading(true);
    setError(null);
    try {
      const data = await getUserBookings(userId);
      setBookings(data);
    } catch (err) {
      setError(getErrorMessage(err));
    } finally {
      setIsLoading(false);
    }
  }, [userId]);

  useEffect(() => { loadBookings(); }, [loadBookings]);

  // Reset to page 1 when tab changes
  useEffect(() => { setPage(1); }, [activeTab]);

  if (!isAuthed) return null;

  const filtered = bookings.filter((b) =>
    activeTab === 'Upcoming' ? isUpcoming(b) : !isUpcoming(b)
  );
  const sorted = [...filtered].sort(
    (a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
  );

  const totalPages = Math.max(1, Math.ceil(sorted.length / PAGE_SIZE));
  const safePage = Math.min(page, totalPages);
  const paginated = sorted.slice((safePage - 1) * PAGE_SIZE, safePage * PAGE_SIZE);

  return (
    <div className="container" style={{ paddingTop: 'var(--space-10)', paddingBottom: 'var(--space-16)' }}>
      <h1 className={styles.pageHeading}>My Bookings</h1>

      {/* Filter tabs */}
      <div className={styles.tabsRow} role="tablist" aria-label="Filter bookings by status">
        {TABS.map((tab) => {
          const count = tab === 'Upcoming'
            ? bookings.filter((b) => isUpcoming(b)).length
            : bookings.filter((b) => !isUpcoming(b)).length;
          return (
            <button
              key={tab}
              role="tab"
              aria-selected={activeTab === tab}
              className={`${styles.tab} ${activeTab === tab ? styles.tabActive : ''}`}
              onClick={() => setActiveTab(tab)}
            >
              {tab}
              {count > 0 && <span className={styles.tabCount}>{count}</span>}
            </button>
          );
        })}
      </div>

      <hr className={`ornamental-divider ${styles.tabDivider}`} />

      {isLoading ? (
        <div className={styles.grid}>
          {[...Array(3)].map((_, i) => <BookingSkeleton key={i} />)}
        </div>
      ) : error ? (
        <div className={styles.errorRow}>
          <p>{error}</p>
          <button onClick={loadBookings}>Retry</button>
        </div>
      ) : sorted.length === 0 ? (
        <EmptyState
          title={activeTab === 'Upcoming' ? 'No upcoming bookings' : 'No past bookings'}
          description="Browse shows to get started with your cinema experience."
          icon="clapperboard"
          action={
            <Link
              href="/"
              style={{
                display: 'inline-flex',
                alignItems: 'center',
                padding: '10px 24px',
                background: 'var(--antique-gold)',
                color: 'var(--void-black)',
                fontFamily: 'var(--font-body)',
                fontSize: '0.8125rem',
                fontWeight: 700,
                letterSpacing: '0.12em',
                textTransform: 'uppercase',
                textDecoration: 'none',
                borderRadius: 'var(--radius-md)',
              }}
            >
              Browse Movies
            </Link>
          }
        />
      ) : (
        <>
          <div className={styles.grid}>
            {paginated.map((booking) => (
              <BookingCard key={booking.booking_id} booking={booking} />
            ))}
          </div>

          {totalPages > 1 && (
            <div className={styles.pagination} role="navigation" aria-label="Booking pages">
              <button
                className={styles.pageBtn}
                onClick={() => setPage((p) => Math.max(1, p - 1))}
                disabled={safePage === 1}
                aria-label="Previous page"
              >
                <ChevronLeft size={16} strokeWidth={2} />
              </button>

              {Array.from({ length: totalPages }, (_, i) => i + 1).map((p) => (
                <button
                  key={p}
                  className={`${styles.pageBtn} ${p === safePage ? styles.pageBtnActive : ''}`}
                  onClick={() => setPage(p)}
                  aria-label={`Page ${p}`}
                  aria-current={p === safePage ? 'page' : undefined}
                >
                  {p}
                </button>
              ))}

              <button
                className={styles.pageBtn}
                onClick={() => setPage((p) => Math.min(totalPages, p + 1))}
                disabled={safePage === totalPages}
                aria-label="Next page"
              >
                <ChevronRight size={16} strokeWidth={2} />
              </button>

              <span className={styles.pageInfo}>
                {safePage} of {totalPages}
              </span>
            </div>
          )}
        </>
      )}
    </div>
  );
}
