'use client';
import { useCallback, useEffect, useMemo, useState } from 'react';
import Link from 'next/link';
import { AlertTriangle, CheckCircle2, Search } from 'lucide-react';
import Badge from '@/components/common/Badge';
import { getAuditLogs } from '@/api/admin';
import { useRequireAdmin } from '@/hooks/useRequireAuth';
import { getErrorMessage } from '@/utils/error';
import { formatDateTime, formatPrice, shortId } from '@/utils/format';
import type { AuditLog } from '@/types/api';
import styles from './page.module.css';

export default function AuditLogsPage() {
  const isAdmin = useRequireAdmin();
  const [logs, setLogs] = useState<AuditLog[]>([]);
  const [query, setQuery] = useState('');
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState('');

  const load = useCallback(async () => {
    setIsLoading(true);
    setError('');
    try {
      setLogs(await getAuditLogs());
    } catch (err) {
      setError(getErrorMessage(err));
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => { load(); }, [load]);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return logs;
    return logs.filter((log) => [
      log.audit_id,
      log.booking_id,
      log.user_id,
      log.show_id,
      log.event_type,
      log.status_from ?? '',
      log.status_to ?? '',
      log.message ?? '',
    ].some((value) => value.toLowerCase().includes(q)));
  }, [logs, query]);

  const failureCount = filtered.filter((log) => log.failed_seats.length > 0 || log.failed_amount > 0).length;

  if (!isAdmin) return null;

  return (
    <div className={styles.page}>
      <header className={styles.header}>
        <div>
          <p className="marquee-label">Support Ledger</p>
          <h1 className={styles.title}>Audit Logs</h1>
          <p className={styles.subtitle}>
            {filtered.length} events · {failureCount} requiring attention
          </p>
        </div>
        <button className={styles.refreshBtn} onClick={load}>Refresh</button>
      </header>

      <section className={styles.toolbar}>
        <div className={styles.searchWrap}>
          <Search size={15} strokeWidth={1.5} className={styles.searchIcon} />
          <input
            className={styles.searchInput}
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder="Search audit, booking, user, event..."
          />
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
          <span>Audit ID</span>
          <span>Booking</span>
          <span>Event</span>
          <span>Status Change</span>
          <span>Confirmed</span>
          <span>Failed</span>
          <span>Failed Amount</span>
          <span>Timestamp</span>
        </div>

        {isLoading ? (
          Array.from({ length: 6 }, (_, index) => (
            <div key={index} className={`${styles.tableRow} ${styles.loadingRow}`}>
              {Array.from({ length: 8 }, (_, cell) => <span key={cell} className={styles.shimmer} />)}
            </div>
          ))
        ) : filtered.length === 0 ? (
          <div className={styles.empty}>No audit events match the current search.</div>
        ) : (
          filtered.map((log) => {
            const hasFailure = log.failed_seats.length > 0 || log.failed_amount > 0;
            return (
              <article
                key={log.audit_id}
                className={`${styles.tableRow} ${hasFailure ? styles.failureRow : ''}`}
              >
                <span className={styles.mono}>{shortId(log.audit_id)}</span>
                <Link href={`/admin/bookings/${log.booking_id}`} className={styles.bookingLink}>
                  {shortId(log.booking_id)}
                </Link>
                <Badge variant={eventVariant(log.event_type)}>{formatLabel(log.event_type)}</Badge>
                <span className={styles.transition}>
                  {log.status_from ?? 'Start'} <span>-&gt;</span> {log.status_to ?? 'Recorded'}
                </span>
                <span className={styles.pillGroup}>
                  {log.confirmed_seats.length > 0 ? (
                    log.confirmed_seats.slice(0, 3).map((seat) => (
                      <span key={seat} className={styles.confirmedPill}>{seat}</span>
                    ))
                  ) : (
                    <span className={styles.muted}>None</span>
                  )}
                </span>
                <span className={styles.pillGroup}>
                  {log.failed_seats.length > 0 ? (
                    log.failed_seats.slice(0, 3).map((seat) => (
                      <span key={seat} className={styles.failedPill}>{seat}</span>
                    ))
                  ) : (
                    <span className={styles.muted}>None</span>
                  )}
                </span>
                <span className={hasFailure ? styles.failedAmount : styles.mutedAmount}>
                  {formatPrice(log.failed_amount)}
                </span>
                <time className={styles.date}>{formatDateTime(log.created_at)}</time>
                {log.message && (
                  <p className={styles.message}>
                    {hasFailure ? <AlertTriangle size={13} strokeWidth={1.5} /> : <CheckCircle2 size={13} strokeWidth={1.5} />}
                    {log.message}
                  </p>
                )}
              </article>
            );
          })
        )}
      </section>
    </div>
  );
}

function formatLabel(value: string) {
  return value.replace(/_/g, ' ').replace(/\b\w/g, (letter) => letter.toUpperCase());
}

function eventVariant(value: string): 'success' | 'error' | 'warning' | 'info' | 'gold' | 'muted' {
  const lower = value.toLowerCase();
  if (lower.includes('fail') || lower.includes('refund')) return 'error';
  if (lower.includes('partial') || lower.includes('compensation')) return 'warning';
  if (lower.includes('success') || lower.includes('confirm')) return 'success';
  if (lower.includes('payment')) return 'gold';
  return 'info';
}
