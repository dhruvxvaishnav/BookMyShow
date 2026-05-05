'use client';
import { useLockTimer } from '@/hooks/useLockTimer';
import { Clock } from 'lucide-react';
import styles from './LockTimer.module.css';

interface LockTimerProps {
  expiresAt: number | null;
  label?: string;
  sublabel?: string;
  onExpire?: () => void;
}

export default function LockTimer({ expiresAt, label, sublabel, onExpire }: LockTimerProps) {
  const { formatted, state, isExpired, secondsLeft } = useLockTimer(expiresAt);

  if (isExpired && onExpire) {
    onExpire();
  }

  const srMessage = state === 'critical'
    ? `Warning: seat lock expires in ${secondsLeft} seconds`
    : state === 'expired'
    ? 'Seat lock has expired'
    : undefined;

  return (
    <>
      {srMessage && (
        <div aria-live="assertive" aria-atomic="true" className={styles.srOnly}>
          {srMessage}
        </div>
      )}
      <div className={`${styles.wrapper} ${styles[state]}`}>
        <div className={styles.iconRow}>
          <Clock size={20} strokeWidth={1.5} />
          {label && <span className={styles.label}>{label}</span>}
        </div>
        <div className={styles.time} aria-label={`Time remaining: ${formatted}`}>{formatted}</div>
        {sublabel && <span className={styles.sub}>{sublabel}</span>}
      </div>
    </>
  );
}
