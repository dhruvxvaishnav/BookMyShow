'use client';
import { useState, useEffect, useCallback } from 'react';
import PageHeader from '@/components/layout/PageHeader';
import BookingCard from '@/components/booking/BookingCard';
import EmptyState from '@/components/common/EmptyState';
import { BookingSkeleton } from '@/components/common/LoadingSkeleton';
import { useToast } from '@/components/layout/Toast';
import { useUserId } from '@/hooks/useUserId';
import { getUserBookings } from '@/api/bookings';
import { getErrorMessage } from '@/utils/error';
import type { Booking, BookingStatus } from '@/types/api';
import styles from './page.module.css';

type FilterTab = 'All' | BookingStatus;

const TABS: FilterTab[] = ['All', 'Pending', 'PaymentPending', 'Success', 'Cancelled', 'Expired'];

export default function MyBookingsPage() {
  const userId = useUserId();
  const toast = useToast();
  const [bookings, setBookings] = useState<Booking[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<FilterTab>('All');

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

  const filtered = bookings.filter((b) => activeTab === 'All' || b.status === activeTab);

  // Sort by created_at descending
  const sorted = [...filtered].sort(
    (a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
  );

  return (
    <>
      <PageHeader
        title="My Bookings"
        subtitle={userId ? `User ID: ${userId.slice(0, 8)}...` : undefined}
      />

      <div className="container">
        {/* Filter tabs */}
        <div className={styles.tabs}>
          {TABS.map((tab) => {
            const count = tab === 'All' ? bookings.length : bookings.filter((b) => b.status === tab).length;
            return (
              <button
                key={tab}
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
          <div className={styles.grid}>
            {sorted.map((booking) => (
              <BookingCard key={booking.booking_id} booking={booking} />
            ))}
          </div>
        )}
      </div>
    </>
  );
}
