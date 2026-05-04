'use client';
import { useCallback, useEffect, useState, use } from 'react';
import PageHeader from '@/components/layout/PageHeader';
import Badge from '@/components/common/Badge';
import { getAdminBooking, getAuditLogs } from '@/api/admin';
import { useRequireAdmin } from '@/hooks/useRequireAuth';
import { getErrorMessage } from '@/utils/error';
import { formatDateTime, formatPrice } from '@/utils/format';
import type { AuditLog, Booking } from '@/types/api';
import styles from './page.module.css';

interface PageProps { params: Promise<{ bookingId: string }> }

export default function AdminBookingDetailPage({ params }: PageProps) {
  const isAdmin = useRequireAdmin();
  const { bookingId } = use(params);
  const [booking, setBooking] = useState<Booking | null>(null);
  const [auditLogs, setAuditLogs] = useState<AuditLog[]>([]);
  const [error, setError] = useState('');
  const [isLoading, setIsLoading] = useState(true);

  const load = useCallback(async () => {
    setIsLoading(true);
    setError('');
    try {
      const [bookingData, auditData] = await Promise.all([
        getAdminBooking(bookingId),
        getAuditLogs({ bookingId }),
      ]);
      setBooking(bookingData);
      setAuditLogs(auditData);
    } catch (err) {
      setError(getErrorMessage(err));
    } finally {
      setIsLoading(false);
    }
  }, [bookingId]);

  useEffect(() => { load(); }, [load]);

  if (!isAdmin) return null;

  return (
    <>
      <PageHeader
        title="Booking Detail"
        subtitle={booking ? booking.booking_id : bookingId}
        backHref="/admin"
      />

      <div className="container">
        {error && <div className={styles.error}>{error}</div>}

        {isLoading ? (
          <div className={styles.loading} />
        ) : booking && (
          <div className={styles.layout}>
            <section className={styles.panel}>
              <h2 className={styles.title}>Booking</h2>
              <div className={styles.grid}>
                <Detail label="Status" value={<Badge variant={statusVariant(booking.status)}>{booking.status}</Badge>} />
                <Detail label="Amount" value={formatPrice(booking.total_amount)} />
                <Detail label="User" value={booking.user_id} mono />
                <Detail label="Show" value={booking.show_id} mono />
                <Detail label="Seats" value={booking.seat_ids.join(', ')} />
                <Detail label="Payment" value={booking.payment_id ?? 'None'} mono />
                <Detail label="Created" value={formatDateTime(Number(booking.created_at))} />
                <Detail label="Expires" value={formatDateTime(booking.expires_at)} />
              </div>
            </section>

            <section className={styles.panel}>
              <h2 className={styles.title}>Audit Trail</h2>
              {auditLogs.length === 0 ? (
                <p className={styles.empty}>No audit events recorded for this booking.</p>
              ) : (
                <div className={styles.timeline}>
                  {auditLogs.map((log) => (
                    <article key={log.audit_id} className={styles.event}>
                      <div className={styles.eventHead}>
                        <span className={styles.eventType}>{formatEvent(log.event_type)}</span>
                        <time>{formatDateTime(log.created_at)}</time>
                      </div>
                      <div className={styles.transition}>
                        <span>{log.status_from ?? 'start'}</span>
                        <span>-&gt;</span>
                        <span>{log.status_to ?? 'recorded'}</span>
                      </div>
                      {log.message && <p className={styles.message}>{log.message}</p>}
                      {(log.failed_seats.length > 0 || log.failed_amount > 0) && (
                        <p className={styles.warning}>
                          Failed seats: {log.failed_seats.join(', ') || 'none'} - Refund candidate {formatPrice(log.failed_amount)}
                        </p>
                      )}
                    </article>
                  ))}
                </div>
              )}
            </section>
          </div>
        )}
      </div>
    </>
  );
}

function Detail({
  label,
  value,
  mono,
}: {
  label: string;
  value: React.ReactNode;
  mono?: boolean;
}) {
  return (
    <div className={styles.detail}>
      <span>{label}</span>
      <strong className={mono ? styles.mono : undefined}>{value}</strong>
    </div>
  );
}

function formatEvent(eventType: string) {
  return eventType.replace(/_/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase());
}

const statusVariant = (status: string): 'success' | 'error' | 'warning' | 'muted' | 'gold' => {
  const map: Record<string, 'success' | 'error' | 'warning' | 'muted' | 'gold'> = {
    success: 'success',
    pending: 'warning',
    payment_pending: 'gold',
    expired: 'muted',
    cancelled: 'error',
    success_partial: 'warning',
  };
  return map[status] ?? 'muted';
};
