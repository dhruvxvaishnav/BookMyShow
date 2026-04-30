'use client';
import { useState } from 'react';
import type { Seat as SeatType } from '@/types/api';
import { formatPrice } from '@/utils/format';
import styles from './Seat.module.css';

export type SeatDisplayState = 'available' | 'selected' | 'locked-you' | 'locked-other' | 'booked';

interface SeatProps {
  seat: SeatType;
  displayState: SeatDisplayState;
  isConflicting?: boolean;
  onClick: (seat: SeatType) => void;
}

export default function Seat({ seat, displayState, isConflicting, onClick }: SeatProps) {
  const [isHovered, setIsHovered] = useState(false);

  const canClick = displayState === 'available' || displayState === 'selected';

  const handleClick = () => {
    if (canClick) onClick(seat);
  };

  return (
    <div className={styles.wrapper}>
      <button
        className={`
          ${styles.seat}
          ${styles[displayState]}
          ${isConflicting ? styles.conflicting : ''}
        `}
        onClick={handleClick}
        onMouseEnter={() => setIsHovered(true)}
        onMouseLeave={() => setIsHovered(false)}
        disabled={!canClick}
        title={`${seat.seat_number} — ${seat.seat_type} — ${formatPrice(seat.price)}`}
        aria-label={`${seat.seat_number}, ${seat.seat_type}, ${displayState}`}
      >
        <span className={styles.label}>{seat.seat_number}</span>
      </button>

      {isHovered && (
        <div className={styles.tooltip}>
          <strong>{seat.seat_number}</strong>
          <span>{seat.seat_type}</span>
          <span>{formatPrice(seat.price)}</span>
        </div>
      )}
    </div>
  );
}
