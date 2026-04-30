import styles from './LoadingSkeleton.module.css';

interface LoadingSkeletonProps {
  rows?: number;
  height?: number;
  className?: string;
}

export function CardSkeleton() {
  return (
    <div className={styles.card}>
      <div className={`${styles.shimmerBlock} ${styles.cardImg}`} />
      <div className={styles.cardBody}>
        <div className={`${styles.shimmerLine} ${styles.title}`} />
        <div className={`${styles.shimmerLine} ${styles.subtitle}`} />
        <div className={`${styles.shimmerLine} ${styles.subtitle} ${styles.w75}`} />
        <div className={`${styles.shimmerLine} ${styles.btn}`} />
      </div>
    </div>
  );
}

export function SeatGridSkeleton() {
  return (
    <div className={styles.seatGrid}>
      <div className={`${styles.shimmerBlock} ${styles.screen}`} />
      {[...Array(6)].map((_, i) => (
        <div key={i} className={styles.seatRow}>
          <div className={styles.rowLabel} />
          {[...Array(10)].map((_, j) => (
            <div key={j} className={`${styles.shimmerBlock} ${styles.seat}`} />
          ))}
        </div>
      ))}
    </div>
  );
}

export function BookingSkeleton() {
  return (
    <div className={styles.booking}>
      <div className={`${styles.shimmerLine} ${styles.ticketTitle}`} />
      <div className={`${styles.shimmerLine} ${styles.ticketSub}`} />
      <div className={styles.timerRow}>
        <div className={`${styles.shimmerLine} ${styles.timer}`} />
      </div>
    </div>
  );
}

export default function LoadingSkeleton({ rows = 3, height = 20 }: LoadingSkeletonProps) {
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
      {[...Array(rows)].map((_, i) => (
        <div
          key={i}
          className="shimmer"
          style={{ height: `${height}px`, borderRadius: '6px' }}
        />
      ))}
    </div>
  );
}
