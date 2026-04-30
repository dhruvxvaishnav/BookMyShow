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

  return (
    <div className={styles.panel}>
      <div className={styles.header}>
        <h3>Your Selection</h3>
        {count > 0 && (
          <button className={styles.clearBtn} onClick={() => {}} title="Clear all">
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
              <span>Total ({count} seat{count !== 1 ? 's' : ''})</span>
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
  const cls = type === 'Premium' ? styles.premiumBadge : type === 'Recliner' ? styles.reclinerBadge : '';
  return <span className={`${styles.typeBadge} ${cls}`}>{type}</span>;
}
