'use client';
import type { Seat, Show } from '@/types/api';
import { formatPrice, formatSeatList } from '@/utils/format';
import Button from '@/components/forms/Button';
import { Lock, Trash2 } from 'lucide-react';
import styles from './SeatSelectionPanel.module.css';

interface SeatSelectionPanelProps {
  selectedSeats: Seat[];
  show: Show;
  onLock: () => void;
  isLocking: boolean;
}

export default function SeatSelectionPanel({
  selectedSeats,
  show,
  onLock,
  isLocking,
}: SeatSelectionPanelProps) {
  const total = selectedSeats.reduce((sum, s) => sum + s.price, 0);
  const count = selectedSeats.length;

  const breakdown = selectedSeats.reduce((acc, seat) => {
    if (!acc[seat.seat_type]) {
      acc[seat.seat_type] = { count: 0, price: seat.price };
    }
    acc[seat.seat_type].count += 1;
    return acc;
  }, {} as Record<string, { count: number; price: number }>);

  const breakdownText = Object.entries(breakdown)
    .map(([type, data]) => `${data.count}× ${formatSeatType(type)} ${formatPrice(data.price * data.count)}`)
    .join(' + ');

  return (
    <div className={styles.panel}>
      <div className={styles.header}>
        <h3>Your Selection</h3>
        {count > 0 && (
          <button
            className={styles.clearBtn}
            onClick={() => {}}
            title="Clear all"
            aria-label="Clear selected seats"
          >
            <Trash2 size={14} strokeWidth={1.5} />
          </button>
        )}
      </div>

      <div className={styles.body}>
        {count === 0 ? (
          <p className={styles.empty}>Click seats to select them.</p>
        ) : (
          <>
            <div className={styles.seats}>
              {selectedSeats.map((seat) => (
                <div key={seat.seat_id} className={styles.seatRow}>
                  <span className={styles.seatNum}>
                    {seat.seat_number}
                    <BadgeInline type={seat.seat_type} />
                  </span>
                  <span className={styles.seatPrice}>{formatPrice(seat.price)}</span>
                </div>
              ))}
            </div>

            <div className={styles.divider} />

            <div className={styles.totalRow}>
              <div style={{ display: 'flex', flexDirection: 'column' }}>
                <span>Total ({count} seat{count !== 1 ? 's' : ''})</span>
                {count > 0 && <span style={{ fontSize: '0.8rem', color: 'var(--text-muted)' }}>{breakdownText}</span>}
              </div>
              <span className={styles.totalAmt}>{formatPrice(total)}</span>
            </div>
          </>
        )}
      </div>

      <div className={styles.footer}>
        <Button
          variant="primary"
          size="lg"
          isLoading={isLocking}
          disabled={count === 0}
          onClick={onLock}
          leftIcon={<Lock size={16} strokeWidth={1.5} />}
          style={{ width: '100%' }}
        >
          Lock Seats {count > 0 ? `(${count})` : ''}
        </Button>
        <p className={styles.note}>5 min lock · extend up to 2 times</p>
      </div>
    </div>
  );
}

function BadgeInline({ type }: { type: string }) {
  const cls = type === 'comfort' || type === 'premium' ? styles.premiumBadge : type === 'recliner' ? styles.reclinerBadge : '';
  return <span className={`${styles.typeBadge} ${cls}`}>{formatSeatType(type)}</span>;
}

function formatSeatType(type: string) {
  return type === 'premium' ? 'comfort' : type;
}
