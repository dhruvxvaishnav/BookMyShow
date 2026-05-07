'use client';
import type { KeyboardEvent, Ref } from 'react';
import type { Seat as SeatType } from '@/types/api';
import type { ShowExperience } from '@/utils/showExperience';
import type { SeatDisplayState } from './Seat';
import Seat from './Seat';
import styles from './SeatRow.module.css';

type SeatMapExperience = Exclude<ShowExperience, 'all'>;

interface SeatRowProps {
  rowLabel: string;
  seats: SeatType[];
  displayStates: Record<string, SeatDisplayState>;
  conflictingSeats: string[];
  onSeatClick: (seat: SeatType) => void;
  focusedSeatId?: string;
  registerSeat?: (seatId: string) => Ref<HTMLButtonElement>;
  onSeatFocus?: (seatId: string) => void;
  onSeatKeyDown?: (event: KeyboardEvent<HTMLButtonElement>, seat: SeatType) => void;
  experience?: SeatMapExperience;
  tooltipPlacement?: 'above' | 'below';
}

export default function SeatRow({
  rowLabel,
  seats,
  displayStates,
  conflictingSeats,
  onSeatClick,
  focusedSeatId,
  registerSeat,
  onSeatFocus,
  onSeatKeyDown,
  experience = 'normal',
  tooltipPlacement = 'above',
}: SeatRowProps) {
  const seatSegments = splitSeatsForAisles(seats, experience);

  return (
    <div className={styles.row} data-experience={experience}>
      <div className={styles.rowLabel}>
        <span className={styles.label}>{rowLabel}</span>
      </div>

      <div className={styles.seats}>
        {seatSegments.map((segment, segmentIndex) => (
          <div key={`${rowLabel}-${segmentIndex}`} className={styles.seatSegment}>
            {segment.map((seat) => (
              <Seat
                key={seat.seat_id}
                seat={seat}
                displayState={displayStates[seat.seat_id] ?? 'available'}
                isConflicting={conflictingSeats.includes(seat.seat_id)}
                tooltipPlacement={tooltipPlacement}
                onClick={onSeatClick}
                buttonRef={registerSeat?.(seat.seat_id)}
                tabIndex={focusedSeatId === seat.seat_id ? 0 : -1}
                onFocus={() => onSeatFocus?.(seat.seat_id)}
                onKeyDown={onSeatKeyDown}
              />
            ))}
          </div>
        ))}
      </div>

      <div className={styles.rowLabel} aria-hidden="true">
        <span className={styles.label}>{rowLabel}</span>
      </div>
    </div>
  );
}

function splitSeatsForAisles(seats: SeatType[], experience: SeatMapExperience) {
  if (seats.length <= 6) return [seats];

  if (experience === 'luxe') {
    return chunkSeats(seats, 2);
  }

  if (experience === 'imax') {
    const sideCount = seats.length >= 18 ? 4 : 3;
    return splitWithSideBlocks(seats, sideCount);
  }

  const sideCount = seats.length >= 12 ? 4 : 3;
  return splitWithSideBlocks(seats, sideCount);
}

function splitWithSideBlocks(seats: SeatType[], sideCount: number) {
  if (seats.length <= sideCount * 2 + 2) return [seats];
  return [
    seats.slice(0, sideCount),
    seats.slice(sideCount, seats.length - sideCount),
    seats.slice(seats.length - sideCount),
  ].filter((segment) => segment.length > 0);
}

function chunkSeats(seats: SeatType[], chunkSize: number) {
  const chunks: SeatType[][] = [];
  for (let i = 0; i < seats.length; i += chunkSize) {
    chunks.push(seats.slice(i, i + chunkSize));
  }
  return chunks;
}
