'use client';
import { useState, useEffect } from 'react';
import { useRouter } from 'next/navigation';
import Link from 'next/link';
import {
  Plus, DollarSign, CheckCircle, Lock, CalendarDays,
  ChevronLeft, ChevronRight, Filter, Film,
} from 'lucide-react';
import { getShows, getShowAvailability } from '@/api/shows';
import { getAdminBookings } from '@/api/admin';
import { useRequireAdmin } from '@/hooks/useRequireAuth';
import type { Show, ShowAvailability, Booking } from '@/types/api';
import { formatDateTime, formatPrice } from '@/utils/format';
import styles from './page.module.css';

const PAGE_SIZE = 8;

export default function AdminDashboardPage() {
  const isAdmin = useRequireAdmin();
  const router = useRouter();
  const [shows, setShows] = useState<Show[]>([]);
  const [availabilities, setAvailabilities] = useState<Record<string, ShowAvailability>>({});
  const [bookings, setBookings] = useState<Booking[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [page, setPage] = useState(0);

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

  const successBookings = bookings.filter((b) => b.status === 'success');
  const totalRevenue = successBookings.reduce((sum, b) => sum + b.total_amount, 0);
  const activeLocks = bookings.filter((b) => b.status === 'pending' || b.status === 'payment_pending').length;

  // Pagination for shows table
  const totalPages = Math.ceil(shows.length / PAGE_SIZE);
  const pagedShows = shows.slice(page * PAGE_SIZE, (page + 1) * PAGE_SIZE);

  return (
    <div className={styles.page}>
      {/* Header */}
      <div className={styles.header}>
        <div>
          <h1 className={styles.pageTitle}>Performance Overview</h1>
          <p className={styles.pageSubtitle}>Last updated: Just now</p>
        </div>
        <Link href="/admin/shows/new" className={styles.createBtn}>
          <Plus size={16} strokeWidth={2} />
          Create Show
        </Link>
      </div>

      {/* KPI Cards */}
      <div className={styles.kpiRow}>
        <KpiCard
          label="Total Revenue"
          value={formatPrice(totalRevenue)}
          note="+12.5% vs last week"
          noteType="success"
          icon={<DollarSign size={22} strokeWidth={1.5} />}
          iconColor="#F5A623"
          iconBg="rgba(245,166,35,0.12)"
        />
        <KpiCard
          label="Active Bookings"
          value={successBookings.length.toString()}
          note="Confirmed bookings"
          noteType="success"
          icon={<CheckCircle size={22} strokeWidth={1.5} />}
          iconColor="#22C55E"
          iconBg="rgba(34,197,94,0.12)"
        />
        <KpiCard
          label="Currently Locked"
          value={activeLocks.toString()}
          note="Awaiting payment"
          noteType="warning"
          icon={<Lock size={22} strokeWidth={1.5} />}
          iconColor="#F59E0B"
          iconBg="rgba(245,158,11,0.12)"
        />
        <KpiCard
          label="Total Shows"
          value={shows.length.toString()}
          note="Scheduled today"
          noteType="info"
          icon={<CalendarDays size={22} strokeWidth={1.5} />}
          iconColor="#3B82F6"
          iconBg="rgba(59,130,246,0.12)"
        />
      </div>

      {/* Show Performance Table */}
      <div className={styles.tableSection}>
        <div className={styles.tableSectionHeader}>
          <span className="marquee-label">Show Performance</span>
          <button className={styles.filterBtn}>
            <Filter size={14} strokeWidth={1.5} />
            Filter
          </button>
        </div>

        <div className={styles.table}>
          <div className={styles.tableHead}>
            <span>Show Name</span>
            <span>Occupancy</span>
            <span>Revenue</span>
            <span>Status</span>
          </div>

          {isLoading ? (
            <>
              {[...Array(4)].map((_, i) => (
                <div key={i} className={`${styles.tableRow} ${styles.shimmerRow}`}>
                  <div className={styles.shimmerCell} />
                  <div className={styles.shimmerCell} style={{ width: '60%' }} />
                  <div className={styles.shimmerCell} style={{ width: '40%' }} />
                  <div className={styles.shimmerCell} style={{ width: '30%' }} />
                </div>
              ))}
            </>
          ) : pagedShows.length === 0 ? (
            <div className={styles.emptyRow}>No shows found. Create one to get started.</div>
          ) : (
            pagedShows.map((show) => {
              const avail = availabilities[show.show_id];
              const occupancy = avail?.occupancy_percent ?? 0;
              const revenue = successBookings
                .filter((b) => b.show_id === show.show_id)
                .reduce((sum, b) => sum + b.total_amount, 0);
              const isActive = show.end_time > Date.now() / 1000;
              return (
                <div
                  key={show.show_id}
                  className={styles.tableRow}
                  onClick={() => router.push(`/admin/shows/${show.show_id}`)}
                  role="button"
                  tabIndex={0}
                  onKeyDown={(e) => e.key === 'Enter' && router.push(`/admin/shows/${show.show_id}`)}
                >
                  <div className={styles.showCell}>
                    <div className={styles.showIcon}>
                      <Film size={14} strokeWidth={1.5} />
                    </div>
                    <div className={styles.showInfo}>
                      <span className={styles.showName}>{show.name}</span>
                      <span className={styles.showMeta}>
                        {show.theatre_name} · Screen {show.screen_number}
                      </span>
                    </div>
                  </div>

                  <div className={styles.occupancyCell}>
                    <div className={styles.occupancyBar}>
                      <div
                        className={styles.occupancyFill}
                        style={{
                          width: `${occupancy}%`,
                          background: occupancy > 70 ? '#22C55E' : occupancy > 40 ? '#F59E0B' : '#EF4444',
                        }}
                      />
                    </div>
                    <span className={styles.occupancyPct}>{occupancy}%</span>
                  </div>

                  <span className={styles.revenueMono}>{formatPrice(revenue)}</span>

                  <span className={`${styles.statusBadge} ${isActive ? styles.statusActive : styles.statusCancelled}`}>
                    {isActive ? 'Active' : 'Ended'}
                  </span>
                </div>
              );
            })
          )}
        </div>

        {/* Pagination */}
        {shows.length > 0 && (
          <div className={styles.pagination}>
            <span className={styles.paginationInfo}>
              Showing {page * PAGE_SIZE + 1}–{Math.min((page + 1) * PAGE_SIZE, shows.length)} of {shows.length} shows
            </span>
            <div className={styles.paginationBtns}>
              <button
                className={styles.pageBtn}
                disabled={page === 0}
                onClick={() => setPage((p) => p - 1)}
                aria-label="Previous page"
              >
                <ChevronLeft size={16} strokeWidth={1.5} />
              </button>
              <button
                className={styles.pageBtn}
                disabled={page >= totalPages - 1}
                onClick={() => setPage((p) => p + 1)}
                aria-label="Next page"
              >
                <ChevronRight size={16} strokeWidth={1.5} />
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

interface KpiCardProps {
  label: string;
  value: string;
  note: string;
  noteType: 'success' | 'warning' | 'info' | 'muted';
  icon: React.ReactNode;
  iconColor: string;
  iconBg: string;
}

function KpiCard({ label, value, note, noteType, icon, iconColor, iconBg }: KpiCardProps) {
  return (
    <div className={styles.kpiCard}>
      <div className={styles.kpiContent}>
        <p className={styles.kpiLabel}>{label}</p>
        <p className={styles.kpiValue}>{value}</p>
        <p className={`${styles.kpiNote} ${styles[`note_${noteType}`]}`}>{note}</p>
      </div>
      <div className={styles.kpiIcon} style={{ color: iconColor, background: iconBg }}>
        {icon}
      </div>
    </div>
  );
}
