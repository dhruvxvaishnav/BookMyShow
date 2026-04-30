'use client';
import { useState, useEffect, useCallback, useRef } from 'react';

export type TimerState = 'safe' | 'warning' | 'critical' | 'expired';

export interface LockTimerResult {
  secondsLeft: number;
  formatted: string;
  state: TimerState;
  isExpired: boolean;
}

/**
 * Countdown timer from a Unix expires_at timestamp.
 * Polls every second.
 */
export function useLockTimer(expiresAt: number | null): LockTimerResult {
  const [secondsLeft, setSecondsLeft] = useState<number>(0);
  const tickRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const compute = useCallback(() => {
    if (expiresAt === null) return 0;
    return Math.max(0, expiresAt - Math.floor(Date.now() / 1000));
  }, [expiresAt]);

  useEffect(() => {
    setSecondsLeft(compute());

    tickRef.current = setInterval(() => {
      const remaining = compute();
      setSecondsLeft(remaining);
      if (remaining === 0 && tickRef.current) {
        clearInterval(tickRef.current);
      }
    }, 1000);

    return () => {
      if (tickRef.current) clearInterval(tickRef.current);
    };
  }, [compute]);

  const m = Math.floor(secondsLeft / 60);
  const s = secondsLeft % 60;
  const formatted = `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;

  let state: TimerState = 'safe';
  if (secondsLeft === 0) state = 'expired';
  else if (secondsLeft <= 30) state = 'critical';
  else if (secondsLeft <= 120) state = 'warning';

  return { secondsLeft, formatted, state, isExpired: secondsLeft === 0 };
}
