'use client';
import { useState, useEffect, useCallback, use, useMemo } from 'react';
import { useRouter } from 'next/navigation';
import { RefreshCw } from 'lucide-react';
import PageHeader from '@/components/layout/PageHeader';
import Button from '@/components/forms/Button';
import Badge from '@/components/common/Badge';
import Modal from '@/components/layout/Modal';
import { useToast } from '@/components/layout/Toast';
import { getShow, getSeatLayout } from '@/api/shows';
import { getShowAnalytics, forceReleaseSeat, cancelShow } from '@/api/admin';
import { useRequireAdmin } from '@/hooks/useRequireAuth';
import { formatPrice } from '@/utils/format';
import type { Show, AdminShowAnalytics, Seat } from '@/types/api';
import styles from './page.module.css';

interface PageProps { params: Promise<{ showId: string }> }

export default function ShowAnalyticsPage({ params }: PageProps) {
  const isAdmin = useRequireAdmin();
  const { showId } = use(params);
  const router = useRouter();
  const toast = useToast();

  const [show, setShow] = useState<Show | null>(null);
  const [analytics, setAnalytics] = useState<AdminShowAnalytics | null>(null);
  const [seats, setSeats] = useState<Seat[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [releasingSeatId, setReleasingSeatId] = useState<string | null>(null);
  const [confirmReleaseSeat, setConfirmReleaseSeat] = useState<Seat | null>(null);
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

  if (!isAdmin) return null;

  const handleForceRelease = async (seatId: string) => {
    setConfirmReleaseSeat(null);
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

        {/* Visual Seat Map */}
        <div className={styles.section}>
          <div className={styles.sectionTitleRow}>
            <h2 className={styles.sectionTitle}>Live Seat Map</h2>
            <button className={styles.refreshBtn} onClick={load} aria-label="Refresh seat data">
              <RefreshCw size={14} strokeWidth={1.5} />
              Refresh
            </button>
          </div>
          <AdminSeatMap
            seats={seats}
            releasingSeatId={releasingSeatId}
            onRelease={(seat) => setConfirmReleaseSeat(seat)}
          />
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
                      onClick={() => setConfirmReleaseSeat(seat)}
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

      {/* Per-seat release confirmation */}
      <Modal
        isOpen={!!confirmReleaseSeat}
        onClose={() => setConfirmReleaseSeat(null)}
        title="Release Seat Lock?"
      >
        <div className={styles.modalBody}>
          <p>
            Release the lock on seat <strong style={{ color: 'var(--antique-gold)', fontFamily: 'var(--font-mono)' }}>{confirmReleaseSeat?.seat_number}</strong>?
            This will make the seat available for other users immediately.
          </p>
          <div className={styles.modalActions}>
            <Button variant="secondary" onClick={() => setConfirmReleaseSeat(null)}>Cancel</Button>
            <Button
              variant="danger"
              isLoading={releasingSeatId === confirmReleaseSeat?.seat_id}
              onClick={() => confirmReleaseSeat && handleForceRelease(confirmReleaseSeat.seat_id)}
            >
              Release Seat
            </Button>
          </div>
        </div>
      </Modal>

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

function AdminSeatMap({
  seats,
  releasingSeatId,
  onRelease,
}: {
  seats: Seat[];
  releasingSeatId: string | null;
  onRelease: (seat: Seat) => void;
}) {
  const rows = useMemo(() => {
    const grouped: Record<string, Seat[]> = {};
    for (const seat of seats) {
      if (!grouped[seat.row_label]) grouped[seat.row_label] = [];
      grouped[seat.row_label].push(seat);
    }
    for (const key of Object.keys(grouped)) {
      grouped[key].sort((a, b) => a.seat_number.localeCompare(b.seat_number, undefined, { numeric: true }));
    }
    return Object.keys(grouped).sort().map((label) => ({ label, seats: grouped[label] }));
  }, [seats]);

  if (rows.length === 0) return null;

  return (
    <div className={styles.seatMap}>
      {/* Screen indicator */}
      <div className={styles.screenIndicator}>
        <div className={styles.screenCurve} aria-hidden="true" />
        <span className={styles.screenLabel}>SCREEN</span>
      </div>

      {/* Seat rows */}
      <div className={styles.seatRows}>
        {rows.map(({ label, seats: rowSeats }) => (
          <div key={label} className={styles.seatRow}>
            <span className={styles.rowLabel}>{label}</span>
            {rowSeats.map((seat) => {
              const isLocked = seat.status === 'Locked';
              const isBooked = seat.status === 'Booked';
              const isReleasing = releasingSeatId === seat.seat_id;
              const seatClass = [
                styles.adminSeat,
                isLocked ? styles.seatLocked : '',
                isBooked ? styles.seatBooked : '',
                !isLocked && !isBooked ? styles.seatAvailable : '',
                seat.seat_type === 'Premium' ? styles.seatTypePremium : '',
                seat.seat_type === 'Recliner' ? styles.seatTypeRecliner : '',
              ].filter(Boolean).join(' ');

              return (
                <div key={seat.seat_id} className={styles.seatCell}>
                  <div
                    className={seatClass}
                    title={`${seat.seat_number} — ${seat.seat_type} — ${seat.status}`}
                    aria-label={`Seat ${seat.seat_number}, ${seat.seat_type}, ${seat.status}`}
                  >
                    <span className={styles.seatNum}>{seat.seat_number.replace(/^[A-Z]+/, '')}</span>
                  </div>
                  {isLocked && (
                    <button
                      className={`${styles.releaseBtn} ${isReleasing ? styles.releaseBtnLoading : ''}`}
                      onClick={() => onRelease(seat)}
                      disabled={isReleasing}
                      aria-label={`Release seat ${seat.seat_number}`}
                    >
                      {isReleasing ? '…' : 'Release'}
                    </button>
                  )}
                </div>
              );
            })}
            <span className={styles.rowLabel}>{label}</span>
          </div>
        ))}
      </div>

      {/* Legend */}
      <div className={styles.seatMapLegend}>
        <span className={styles.legendItem}><span className={`${styles.legendDot} ${styles.seatAvailable}`} /> Available</span>
        <span className={styles.legendItem}><span className={`${styles.legendDot} ${styles.seatLocked}`} /> Locked</span>
        <span className={styles.legendItem}><span className={`${styles.legendDot} ${styles.seatBooked}`} /> Booked</span>
        <span className={styles.legendItem}><span className={`${styles.legendDot} ${styles.seatTypePremium}`} style={{ border: '2px solid #A855F7' }} /> Premium</span>
        <span className={styles.legendItem}><span className={`${styles.legendDot} ${styles.seatTypeRecliner}`} style={{ border: '2px solid #06B6D4' }} /> Recliner</span>
      </div>
    </div>
  );
}