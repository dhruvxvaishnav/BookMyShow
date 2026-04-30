'use client';
import type { Seat as SeatType } from '@/types/api';
import type { SeatDisplayState } from './Seat';
import SeatRow from './SeatRow';
import ScreenIndicator from './ScreenIndicator';
import SeatLegend from './SeatLegend';
import { useMemo } from 'react';
import styles from './SeatGrid.module.css';

interface SeatGridProps {
  seats: SeatType[];
  selectedSeatIds: string[];
  lockedByYouSeatIds: string[];
  conflictingSeats: string[];
  userId: string;
  onSeatClick: (seat: SeatType) => void;
}

export default function SeatGrid({
  seats,
  selectedSeatIds,
  lockedByYouSeatIds,
  conflictingSeats,
  userId,
  onSeatClick,
}: SeatGridProps) {
  // Group seats by row
  const rows = useMemo(() => {
    const grouped: Record<string, SeatType[]> = {};
    for (const seat of seats) {
      if (!grouped[seat.row_label]) grouped[seat.row_label] = [];
      grouped[seat.row_label].push(seat);
    }
    // Sort each row by seat number
    for (const key of Object.keys(grouped)) {
      grouped[key].sort((a, b) => a.seat_number.localeCompare(b.seat_number, undefined, { numeric: true }));
    }
    // Return sorted by row label
    return Object.keys(grouped).sort().map((label) => ({ label, seats: grouped[label] }));
  }, [seats]);

  // Compute display state per seat
  const displayStates = useMemo(() => {
    const states: Record<string, SeatDisplayState> = {};
    for (const seat of seats) {
      if (seat.status === 'Booked') {
        states[seat.seat_id] = 'booked';
      } else if (seat.status === 'Locked') {
        if (selectedSeatIds.includes(seat.seat_id) || lockedByYouSeatIds.includes(seat.seat_id)) {
          states[seat.seat_id] = 'locked-you';
        } else {
          states[seat.seat_id] = 'locked-other';
        }
      } else if (selectedSeatIds.includes(seat.seat_id)) {
        states[seat.seat_id] = 'selected';
      } else {
        states[seat.seat_id] = 'available';
      }
    }
    return states;
  }, [seats, selectedSeatIds, lockedByYouSeatIds]);

  return (
    <div className={styles.grid}>
      <ScreenIndicator />

      <div className={styles.rows}>
        {rows.map(({ label, seats: rowSeats }) => (
          <SeatRow
            key={label}
            rowLabel={label}
            seats={rowSeats}
            displayStates={displayStates}
            conflictingSeats={conflictingSeats}
            onSeatClick={onSeatClick}
          />
        ))}
      </div>

      <SeatLegend />
    </div>
  );
}
