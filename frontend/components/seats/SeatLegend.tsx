import styles from './SeatLegend.module.css';

export default function SeatLegend() {
  return (
    <div className={styles.legend}>
      <div className={styles.item}>
        <div className={`${styles.swatch} ${styles.available}`} />
        <span>Available</span>
      </div>
      <div className={styles.item}>
        <div className={`${styles.swatch} ${styles.selected}`} />
        <span>Your Selection</span>
      </div>
      <div className={styles.item}>
        <div className={`${styles.swatch} ${styles.locked}`} />
        <span>Unavailable</span>
      </div>
      <div className={styles.item}>
        <div className={`${styles.swatch} ${styles.premium}`} />
        <span>Comfort</span>
      </div>
      <div className={styles.item}>
        <div className={`${styles.swatch} ${styles.recliner}`} />
        <span>Recliner</span>
      </div>
    </div>
  );
}
