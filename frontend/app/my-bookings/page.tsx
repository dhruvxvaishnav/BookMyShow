'use client';
import { useState, useEffect, useCallback } from 'react';
import { ChevronLeft, ChevronRight } from 'lucide-react';
import PageHeader from '@/components/layout/PageHeader';
import BookingCard from '@/components/booking/BookingCard';
import EmptyState from '@/components/common/EmptyState';
import { BookingSkeleton } from '@/components/common/LoadingSkeleton';
import { useUserId } from '@/hooks/useUserId';
import { useRequireAuth } from '@/hooks/useRequireAuth';
import { getUserBookings } from '@/api/bookings';
import { getErrorMessage } from '@/utils/error';
import type { Booking, BookingStatus } from '@/types/api';
import styles from './page.module.css';

type FilterTab = 'All' | BookingStatus;

const TABS: FilterTab[] = ['All', 'Pending', 'PaymentPending', 'Success', 'Cancelled', 'Expired'];
const PAGE_SIZE = 9;

export default function MyBookingsPage() {
  const isAuthed = useRequireAuth();
  const userId = useUserId();
  const [bookings, setBookings] = useState<Booking[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<FilterTab>('All');
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

  const filtered = bookings.filter((b) => activeTab === 'All' || b.status === activeTab);
  const sorted = [...filtered].sort(
    (a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
  );

  const totalPages = Math.max(1, Math.ceil(sorted.length / PAGE_SIZE));
  const safePage = Math.min(page, totalPages);
  const paginated = sorted.slice((safePage - 1) * PAGE_SIZE, safePage * PAGE_SIZE);

  return (
    <>
      <PageHeader
        title="My Bookings"
        subtitle={userId ? `User ID: ${userId.slice(0, 8)}...` : undefined}
      />

      <div className="container">
        {/* Filter tabs */}
        <div className={styles.tabs} role="tablist" aria-label="Filter bookings by status">
          {TABS.map((tab) => {
            const count = tab === 'All' ? bookings.length : bookings.filter((b) => b.status === tab).length;
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
            title={activeTab === 'All' ? 'No bookings yet' : `No ${activeTab.toLowerCase()} bookings`}
            description="Browse shows to get started with your cinema experience."
            icon="clapperboard"
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
                  {(safePage - 1) * PAGE_SIZE + 1}–{Math.min(safePage * PAGE_SIZE, sorted.length)} of {sorted.length}
                </span>
              </div>
            )}
          </>
        )}
      </div>
    </>
  );
}
