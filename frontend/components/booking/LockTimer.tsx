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
  const { formatted, state, isExpired } = useLockTimer(expiresAt);

  if (isExpired && onExpire) {
    onExpire();
  }

  return (
    <div className={`${styles.wrapper} ${styles[state]}`}>
      <div className={styles.iconRow}>
        <Clock size={20} strokeWidth={1.5} />
        {label && <span className={styles.label}>{label}</span>}
      </div>
      <div className={styles.time}>{formatted}</div>
      {sublabel && <span className={styles.sub}>{sublabel}</span>}
    </div>
  );
}
