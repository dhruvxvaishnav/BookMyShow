'use client';
import { useState, useEffect, useCallback, use } from 'react';
import { useRouter } from 'next/navigation';
import PageHeader from '@/components/layout/PageHeader';
import Button from '@/components/forms/Button';
import Badge from '@/components/common/Badge';
import Modal from '@/components/layout/Modal';
import { useToast } from '@/components/layout/Toast';
import { getShow, getSeatLayout } from '@/api/shows';
import { getShowAnalytics, forceReleaseSeat, cancelShow } from '@/api/admin';
import { formatPrice } from '@/utils/format';
import type { Show, AdminShowAnalytics, Seat } from '@/types/api';
import styles from './page.module.css';

interface PageProps { params: Promise<{ showId: string }> }

export default function ShowAnalyticsPage({ params }: PageProps) {
  const { showId } = use(params);
  const router = useRouter();
  const toast = useToast();

  const [show, setShow] = useState<Show | null>(null);
  const [analytics, setAnalytics] = useState<AdminShowAnalytics | null>(null);
  const [seats, setSeats] = useState<Seat[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [releasingSeatId, setReleasingSeatId] = useState<string | null>(null);
  const [showCancelModal, setShowCancelModal] = useState(false);
  const [isCancelling, setIsCancelling] = useState(false);

  const load = useCallback(async () => {
    try {
      const [showData, analyticsData, seatData] = await Promise.all([
        getShow(showId),
        getShowAnalytics(showId).catch(() => null),
        getSeatLayout(showId),
      ]);
      setShow(showData);
      setAnalytics(analyticsData);
      setSeats(seatData.seats);
    } catch {
      toast.showToast('Failed to load show analytics.', 'error');
    } finally {
      setIsLoading(false);
    }
  }, [showId, toast]);

  useEffect(() => { load(); }, [load]);

  const handleForceRelease = async (seatId: string) => {
    setReleasingSeatId(seatId);
    try {
      await forceReleaseSeat(showId, seatId);
      toast.showToast('Seat released successfully.', 'success');
      load();
    } catch (err) {
      toast.showToast(err instanceof Error ? err.message : 'Failed to release seat.', 'error');
    } finally {
      setReleasingSeatId(null);
    }
  };

  const handleCancelShow = async () => {
    setIsCancelling(true);
    try {
      await cancelShow(showId);
      toast.showToast('Show cancelled. All bookings will be refunded.', 'success');
      router.push('/admin');
    } catch (err) {
      toast.showToast(err instanceof Error ? err.message : 'Failed to cancel show.', 'error');
    } finally {
      setIsCancelling(false);
      setShowCancelModal(false);
    }
  };

  // Group seats by status
  const availableCount = seats.filter((s) => s.status === 'Available').length;
  const lockedCount = seats.filter((s) => s.status === 'Locked').length;
  const bookedCount = seats.filter((s) => s.status === 'Booked').length;
  const total = seats.length;

  return (
    <>
      <PageHeader
        title={show?.name ?? 'Show Analytics'}
        subtitle={show ? `${show.theatre_name} · Screen ${show.screen_number}` : undefined}
        backHref="/admin"
      />

      <div className="container">
        {/* Stats */}
        <div className={styles.statsRow}>
          <StatCard label="Available" value={availableCount.toString()} total={total} color="#22C55E" />
          <StatCard label="Locked" value={lockedCount.toString()} total={total} color="#F5A623" />
          <StatCard label="Booked" value={bookedCount.toString()} total={total} color="#3B82F6" />
          <StatCard label="Total Seats" value={total.toString()} total={total} color="#9CA3AF" />
          {analytics && (
            <StatCard label="Revenue" value={formatPrice(analytics.revenue)} total={total} color="#F5A623" />
          )}
        </div>

        {/* Seat override table */}
        <div className={styles.section}>
          <h2 className={styles.sectionTitle}>Seat Management</h2>
          <div className={styles.table}>
            <div className={styles.tableHead}>
              <span>Seat</span>
              <span>Type</span>
              <span>Status</span>
              <span>Expires</span>
              <span>Action</span>
            </div>
            {seats.map((seat) => (
              <div key={seat.seat_id} className={styles.tableRow}>
                <span className={styles.mono}>{seat.seat_number}</span>
                <Badge variant={seat.seat_type === 'Premium' ? 'purple' : seat.seat_type === 'Recliner' ? 'cyan' : 'muted'}>
                  {seat.seat_type}
                </Badge>
                <Badge variant={seat.status === 'Available' ? 'success' : seat.status === 'Locked' ? 'warning' : 'info'}>
                  {seat.status}
                </Badge>
                <span className={styles.mono}>
                  {seat.lock_expires_at
                    ? new Date(seat.lock_expires_at * 1000).toLocaleTimeString()
                    : '—'}
                </span>
                <div>
                  {seat.status === 'Locked' && (
                    <Button
                      variant="danger"
                      size="sm"
                      isLoading={releasingSeatId === seat.seat_id}
                      onClick={() => handleForceRelease(seat.seat_id)}
                    >
                      Release
                    </Button>
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Danger zone */}
        <div className={styles.dangerZone}>
          <h3>Danger Zone</h3>
          <p>Cancelling this show will cancel all active bookings and issue refunds.</p>
          <Button variant="danger" onClick={() => setShowCancelModal(true)}>
            Cancel Show
          </Button>
        </div>
      </div>

      <Modal
        isOpen={showCancelModal}
        onClose={() => setShowCancelModal(false)}
        title="Cancel Show?"
      >
        <div className={styles.modalBody}>
          <p>This will cancel all active bookings and issue refunds for this show. This action cannot be undone.</p>
          <div className={styles.modalActions}>
            <Button variant="secondary" onClick={() => setShowCancelModal(false)}>Keep Show</Button>
            <Button variant="danger" isLoading={isCancelling} onClick={handleCancelShow}>Cancel Show</Button>
          </div>
        </div>
      </Modal>
    </>
  );
}

function StatCard({ label, value, total, color }: { label: string; value: string; total: number; color: string }) {
  const pct = total > 0 ? Math.round((parseFloat(value) / total) * 100) : 0;
  return (
    <div className={styles.statCard}>
      <div className={styles.statValue} style={{ color }}>{value}</div>
      <div className={styles.statLabel}>{label}</div>
      {total > 0 && (
        <div className={styles.statBar}>
          <div className={styles.statFill} style={{ backgroundColor: color, width: `${pct}%` }} />
        </div>
      )}
    </div>
  );
}