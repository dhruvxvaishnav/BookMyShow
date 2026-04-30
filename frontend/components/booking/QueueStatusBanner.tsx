'use client';
import { Clock } from 'lucide-react';
import Spinner from '@/components/common/Spinner';
import styles from './QueueStatusBanner.module.css';

interface QueueStatusBannerProps {
  position: number | null;
  status: string;
  isProcessing: boolean;
}

export default function QueueStatusBanner({ position, status, isProcessing }: QueueStatusBannerProps) {
  return (
    <div className={`${styles.banner} ${isProcessing ? styles.processing : ''}`}>
      <Spinner size="sm" />
      <div className={styles.text}>
        {isProcessing ? (
          <span>Acquiring your seats...</span>
        ) : (
          <>
            <span>You&apos;re in the queue</span>
            {position !== null && (
              <span className={styles.position}>#{position}</span>
            )}
          </>
        )}
      </div>
    </div>
  );
}
