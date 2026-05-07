'use client';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import type { KeyboardEvent } from 'react';
import type { Seat as SeatType } from '@/types/api';
import type { SeatDisplayState } from './Seat';
import SeatRow from './SeatRow';
import ScreenIndicator from './ScreenIndicator';
import SeatLegend from './SeatLegend';
import type { ShowExperience } from '@/utils/showExperience';
import styles from './SeatGrid.module.css';

type SeatMapExperience = Exclude<ShowExperience, 'all'>;
type SeatZone = 'standard' | 'comfort' | 'recliner';

interface SeatGridProps {
  seats: SeatType[];
  selectedSeatIds: string[];
  lockedByYouSeatIds: string[];
  conflictingSeats: string[];
  userId: string;
  onSeatClick: (seat: SeatType) => void;
  experience?: SeatMapExperience;
}

const ZONE_LABELS: Record<SeatZone, string> = {
  recliner: 'Recliner',
  comfort: 'Comfort',
  standard: 'Standard',
};

export default function SeatGrid({
  seats,
  selectedSeatIds,
  lockedByYouSeatIds,
  conflictingSeats,
  userId,
  onSeatClick,
  experience = 'normal',
}: SeatGridProps) {
  const [focusedSeatId, setFocusedSeatId] = useState<string | undefined>();
  const seatRefs = useRef<Record<string, HTMLButtonElement | null>>({});

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
    // The screen sits at the bottom, so show back rows first and front rows last.
    return Object.keys(grouped)
      .sort((a, b) => b.localeCompare(a, undefined, { numeric: true }))
      .map((label) => ({ label, seats: grouped[label] }));
  }, [seats]);

  const sections = useMemo(() => {
    const groupedSections: Array<{ zone: SeatZone; rows: typeof rows }> = [];

    for (const row of rows) {
      const zone = getSeatZone(row.seats[0]?.seat_type);
      const current = groupedSections.at(-1);

      if (current?.zone === zone) {
        current.rows.push(row);
      } else {
        groupedSections.push({ zone, rows: [row] });
      }
    }

    return groupedSections;
  }, [rows]);

  // Compute display state per seat
  const displayStates = useMemo(() => {
    const states: Record<string, SeatDisplayState> = {};
    for (const seat of seats) {
      if (seat.status === 'booked') {
        states[seat.seat_id] = 'booked';
      } else if (seat.status === 'locked') {
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

  const focusableRows = useMemo(() => rows.map((row) => ({
    ...row,
    seats: row.seats.filter((seat) => {
      const state = displayStates[seat.seat_id];
      return state === 'available' || state === 'selected';
    }),
  })).filter((row) => row.seats.length > 0), [rows, displayStates]);

  useEffect(() => {
    const stillFocusable = focusableRows.some((row) =>
      row.seats.some((seat) => seat.seat_id === focusedSeatId)
    );
    if (!stillFocusable) {
      setFocusedSeatId(focusableRows[0]?.seats[0]?.seat_id);
    }
  }, [focusableRows, focusedSeatId]);

  const registerSeat = useCallback((seatId: string) => (node: HTMLButtonElement | null) => {
    seatRefs.current[seatId] = node;
  }, []);

  const focusSeat = useCallback((seatId?: string) => {
    if (!seatId) return;
    setFocusedSeatId(seatId);
    seatRefs.current[seatId]?.focus();
  }, []);

  const handleSeatKeyDown = useCallback((event: KeyboardEvent<HTMLButtonElement>, seat: SeatType) => {
    const rowIndex = focusableRows.findIndex((row) =>
      row.seats.some((rowSeat) => rowSeat.seat_id === seat.seat_id)
    );
    if (rowIndex < 0) return;

    const seatIndex = focusableRows[rowIndex].seats.findIndex((rowSeat) => rowSeat.seat_id === seat.seat_id);
    let targetSeatId: string | undefined;

    if (event.key === 'ArrowRight') {
      targetSeatId = focusableRows[rowIndex].seats[seatIndex + 1]?.seat_id;
    } else if (event.key === 'ArrowLeft') {
      targetSeatId = focusableRows[rowIndex].seats[seatIndex - 1]?.seat_id;
    } else if (event.key === 'ArrowDown') {
      const nextRow = focusableRows[rowIndex + 1];
      targetSeatId = nextRow?.seats[Math.min(seatIndex, nextRow.seats.length - 1)]?.seat_id;
    } else if (event.key === 'ArrowUp') {
      const previousRow = focusableRows[rowIndex - 1];
      targetSeatId = previousRow?.seats[Math.min(seatIndex, previousRow.seats.length - 1)]?.seat_id;
    } else if (event.key === 'Home') {
      targetSeatId = focusableRows[rowIndex].seats[0]?.seat_id;
    } else if (event.key === 'End') {
      targetSeatId = focusableRows[rowIndex].seats.at(-1)?.seat_id;
    }

    if (targetSeatId) {
      event.preventDefault();
      focusSeat(targetSeatId);
    }
  }, [focusSeat, focusableRows]);

  return (
    <div className={`${styles.grid} ${styles[`grid_${experience}`]}`} role="grid" aria-label="Seat map">
      <div className={styles.rows} aria-label="Seat rows">
        {sections.map((section, sectionIndex) => (
          <section
            key={`${section.zone}-${section.rows.map((row) => row.label).join('-')}`}
            className={`${styles.typeSection} ${styles[`typeSection_${section.zone}`]}`}
            aria-label={`${ZONE_LABELS[section.zone]} seats`}
          >
            <div className={styles.typeSectionLabel} aria-hidden="true">
              <span className={styles.typeSectionLabelText}>{ZONE_LABELS[section.zone]}</span>
              <span className={styles.typeSectionLine} />
            </div>

            <div className={styles.typeSectionRows}>
              {section.rows.map(({ label, seats: rowSeats }) => (
                <SeatRow
                  key={label}
                  rowLabel={label}
                  seats={rowSeats}
                  displayStates={displayStates}
                  conflictingSeats={conflictingSeats}
                  onSeatClick={onSeatClick}
                  focusedSeatId={focusedSeatId}
                  registerSeat={registerSeat}
                  onSeatFocus={setFocusedSeatId}
                  onSeatKeyDown={handleSeatKeyDown}
                  experience={experience}
                  tooltipPlacement={sectionIndex === 0 ? 'below' : 'above'}
                />
              ))}
            </div>
          </section>
        ))}
      </div>

      <ScreenIndicator />

      <SeatLegend />
    </div>
  );
}

function getSeatZone(seatType?: string): SeatZone {
  if (seatType === 'recliner') return 'recliner';
  if (seatType === 'comfort' || seatType === 'premium') return 'comfort';
  return 'standard';
}
