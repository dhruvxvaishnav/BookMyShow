'use client';
import type { KeyboardEvent, Ref } from 'react';
import type { Seat as SeatType } from '@/types/api';
import { formatPrice } from '@/utils/format';
import styles from './Seat.module.css';

export type SeatDisplayState = 'available' | 'selected' | 'locked-you' | 'locked-other' | 'booked';

interface SeatProps {
  seat: SeatType;
  displayState: SeatDisplayState;
  isConflicting?: boolean;
  onClick: (seat: SeatType) => void;
  buttonRef?: Ref<HTMLButtonElement>;
  tabIndex?: number;
  onFocus?: () => void;
  onKeyDown?: (event: KeyboardEvent<HTMLButtonElement>, seat: SeatType) => void;
}

export default function Seat({
  seat,
  displayState,
  isConflicting,
  onClick,
  buttonRef,
  tabIndex,
  onFocus,
  onKeyDown,
}: SeatProps) {
  const canClick = displayState === 'available' || displayState === 'selected';

  const handleClick = () => {
    if (canClick) onClick(seat);
  };

  const stateLabel: Record<SeatDisplayState, string> = {
    available: 'available',
    selected: 'selected',
    'locked-you': 'locked by you',
    'locked-other': 'locked by another user',
    booked: 'booked',
  };

  // Type-specific modifier class
  const typeClass =
    seat.seat_type === 'Premium' ? styles.typePremium
    : seat.seat_type === 'Recliner' ? styles.typeRecliner
    : '';

  return (
    <div className={styles.wrapper}>
      <button
        ref={buttonRef}
        type="button"
        className={[
          styles.seat,
          styles[displayState],
          typeClass,
          isConflicting ? styles.conflicting : '',
        ].join(' ')}
        data-seat-id={seat.seat_id}
        onClick={handleClick}
        onFocus={onFocus}
        onKeyDown={(event) => onKeyDown?.(event, seat)}
        disabled={!canClick}
        tabIndex={canClick ? tabIndex : undefined}
        title={`${seat.seat_number} — ${seat.seat_type} — ${formatPrice(seat.price)}`}
        aria-label={`${seat.seat_number}, ${seat.seat_type} seat, ${stateLabel[displayState]}, ${formatPrice(seat.price)}`}
        aria-pressed={displayState === 'selected'}
      >
        <span className={styles.label}>{seat.seat_number}</span>
        <span className={styles.srOnly}>
          {seat.seat_number}, {seat.seat_type}, {stateLabel[displayState]}, {formatPrice(seat.price)}
        </span>
      </button>

      <div className={styles.tooltip}>
        <strong>{seat.seat_number}</strong>
        <span>{seat.seat_type}</span>
        <span>{formatPrice(seat.price)}</span>
      </div>
    </div>
  );
}
