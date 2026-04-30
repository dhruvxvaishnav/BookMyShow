'use client';
import type { Seat as SeatType } from '@/types/api';
import type { SeatDisplayState } from './Seat';
import Seat from './Seat';
import styles from './SeatRow.module.css';
import Badge from '@/components/common/Badge';

interface SeatRowProps {
  rowLabel: string;
  seats: SeatType[];
  displayStates: Record<string, SeatDisplayState>;
  conflictingSeats: string[];
  onSeatClick: (seat: SeatType) => void;
}

export default function SeatRow({
  rowLabel,
  seats,
  displayStates,
  conflictingSeats,
  onSeatClick,
}: SeatRowProps) {
  // Detect row type from first seat
  const firstSeat = seats[0];
  const isPremium = firstSeat?.seat_type === 'Premium';
  const isRecliner = firstSeat?.seat_type === 'Recliner';

  return (
    <div className={styles.row}>
      {/* Row label + type indicator */}
      <div className={styles.rowLabel}>
        <span className={styles.label}>{rowLabel}</span>
        {isPremium && <Badge variant="purple">PR</Badge>}
        {isRecliner && <Badge variant="cyan">RC</Badge>}
      </div>

      {/* Seats */}
      <div className={styles.seats}>
        {seats.map((seat) => (
          <Seat
            key={seat.seat_id}
            seat={seat}
            displayState={displayStates[seat.seat_id] ?? 'available'}
            isConflicting={conflictingSeats.includes(seat.seat_id)}
            onClick={onSeatClick}
          />
        ))}
      </div>
    </div>
  );
}
