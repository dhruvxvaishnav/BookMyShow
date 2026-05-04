'use client';
import { useState, useEffect } from 'react';
import { useRouter } from 'next/navigation';
import Link from 'next/link';
import { Plus, BarChart2, TrendingUp } from 'lucide-react';
import PageHeader from '@/components/layout/PageHeader';
import Button from '@/components/forms/Button';
import EmptyState from '@/components/common/EmptyState';
import Badge from '@/components/common/Badge';
import { CardSkeleton } from '@/components/common/LoadingSkeleton';
import { getShows, getShowAvailability } from '@/api/shows';
import { getAdminBookings } from '@/api/admin';
import { useRequireAdmin } from '@/hooks/useRequireAuth';
import type { Show, ShowAvailability, Booking } from '@/types/api';
import { formatDateTime, formatPrice } from '@/utils/format';
import styles from './page.module.css';

export default function AdminDashboardPage() {
  const isAdmin = useRequireAdmin();
  const [shows, setShows] = useState<Show[]>([]);
  const [availabilities, setAvailabilities] = useState<Record<string, ShowAvailability>>({});
  const [bookings, setBookings] = useState<Booking[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    Promise.all([
      getShows(),
      getAdminBookings().catch(() => [] as Booking[]),
    ])
      .then(async ([showList, bookingList]) => {
        const availList = await Promise.all(
          showList.map((sh) => getShowAvailability(sh.show_id).catch(() => null))
        );
        const avail: Record<string, ShowAvailability> = {};
        showList.forEach((s, i) => { if (availList[i]) avail[s.show_id] = availList[i]!; });
        setShows(showList);
        setAvailabilities(avail);
        setBookings(bookingList);
      })
      .finally(() => setIsLoading(false));
  }, []);

  if (!isAdmin) return null;

  const todayBookings = bookings.filter((b) => b.status === 'Success');
  const totalRevenue = todayBookings.reduce((sum, b) => sum + b.total_amount, 0);
  const activeLocks = bookings.filter((b) => b.status === 'Pending' || b.status === 'PaymentPending').length;

  return (
    <>
      <PageHeader
        title="Admin Dashboard"
        subtitle="Manage shows, bookings, and analytics"
        actions={
          <Link href="/admin/shows/new">
            <Button variant="primary" leftIcon={<Plus size={16} strokeWidth={1.5} />}>
              Create Show
            </Button>
          </Link>
        }
      />

      <div className="container">
        {/* Stats cards */}
        <div className={styles.statsRow}>
          <StatCard label="Total Shows" value={shows.length.toString()} icon={<BarChart2 size={20} strokeWidth={1.5} />} />
          <StatCard label="Bookings Today" value={todayBookings.length.toString()} icon={<TrendingUp size={20} strokeWidth={1.5} />} />
          <StatCard label="Revenue" value={formatPrice(totalRevenue)} icon={<TrendingUp size={20} strokeWidth={1.5} />} />
          <StatCard label="Active Locks" value={activeLocks.toString()} icon={<BarChart2 size={20} strokeWidth={1.5} />} />
        </div>

        {/* Shows table */}
        <div className={styles.section}>
          <h2 className={styles.sectionTitle}>Shows</h2>
          {isLoading ? (
            <div className={styles.table}>
              {[...Array(3)].map((_, i) => <CardSkeleton key={i} />)}
            </div>
          ) : shows.length === 0 ? (
            <EmptyState
              title="No shows yet"
              description="Create your first show to get started."
              icon="clapperboard"
              action={
                <Link href="/admin/shows/new">
                  <Button variant="primary" leftIcon={<Plus size={16} strokeWidth={1.5} />}>
                    Create Show
                  </Button>
                </Link>
              }
            />
          ) : (
            <div className={styles.table}>
              <div className={styles.tableHead}>
                <span>Show</span>
                <span>Screen</span>
                <span>Time</span>
                <span>Occupancy</span>
                <span>Actions</span>
              </div>
              {shows.map((show) => {
                const avail = availabilities[show.show_id];
                const occupancy = avail?.occupancy_percent ?? 0;
                return (
                  <div key={show.show_id} className={styles.tableRow}>
                    <div className={styles.showInfo}>
                      <span className={styles.showName}>{show.name}</span>
                      <span className={styles.showTheatre}>{show.theatre_name}</span>
                    </div>
                    <span className={styles.mono}>{show.screen_number}</span>
                    <span className={styles.mono}>{formatDateTime(show.start_time)}</span>
                    <div className={styles.occupancy}>
                      <div className={styles.occupancyBar}>
                        <div className={styles.occupancyFill} style={{ width: `${occupancy}%` }} />
                      </div>
                      <span>{occupancy}%</span>
                    </div>
                    <Link href={`/admin/shows/${show.show_id}`}>
                      <Button variant="secondary" size="sm">
                        Analytics
                      </Button>
                    </Link>
                  </div>
                );
              })}
            </div>
          )}
        </div>

        {/* Recent bookings */}
        <div className={styles.section}>
          <h2 className={styles.sectionTitle}>Recent Bookings</h2>
          {bookings.length === 0 ? (
            <p className={styles.emptyText}>No bookings yet.</p>
          ) : (
            <div className={styles.table}>
              <div className={styles.tableHead}>
                <span>Booking ID</span>
                <span>Show</span>
                <span>Status</span>
                <span>Amount</span>
              </div>
              {bookings.slice(0, 10).map((b) => (
                <div key={b.booking_id} className={styles.tableRow}>
                  <Link href={`/admin/bookings/${b.booking_id}`} className={styles.mono}>
                    {b.booking_id.slice(0, 8)}...
                  </Link>
                  <span className={styles.mono}>{b.show_id.slice(0, 8)}...</span>
                  <Badge variant={statusVariant(b.status)}>{b.status}</Badge>
                  <span className={styles.mono}>{formatPrice(b.total_amount)}</span>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </>
  );
}

function StatCard({ label, value, icon }: { label: string; value: string; icon: React.ReactNode }) {
  return (
    <div className={styles.statCard}>
      <div className={styles.statIcon}>{icon}</div>
      <div>
        <div className={styles.statValue}>{value}</div>
        <div className={styles.statLabel}>{label}</div>
      </div>
    </div>
  );
}

const statusVariant = (status: string): 'success' | 'error' | 'warning' | 'muted' | 'gold' => {
  const map: Record<string, 'success' | 'error' | 'warning' | 'muted' | 'gold'> = {
    Success: 'success', Pending: 'warning', PaymentPending: 'gold',
    Expired: 'muted', Cancelled: 'error', PartialSuccess: 'warning',
  };
  return map[status] ?? 'muted';
};
