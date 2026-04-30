import styles from './ScreenIndicator.module.css';

export default function ScreenIndicator() {
  return (
    <div className={styles.wrapper}>
      <div className={styles.curve}>
        <span className={styles.label}>SCREEN</span>
      </div>
    </div>
  );
}
