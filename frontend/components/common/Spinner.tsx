import styles from './Spinner.module.css';

interface SpinnerProps {
  size?: 'sm' | 'md' | 'lg';
  label?: string;
}

export default function Spinner({ size = 'md', label }: SpinnerProps) {
  return (
    <div className={`${styles.wrapper} ${styles[size]}`} role="status" aria-label={label ?? 'Loading'}>
      <svg className={styles.spinner} viewBox="0 0 24 24" fill="none">
        <circle className={styles.track} cx="12" cy="12" r="10" strokeWidth="3" />
        <path className={styles.arc} cx="12" cy="12" r="10" strokeWidth="3"
          strokeLinecap="round"
          strokeDasharray="40 20"
        />
      </svg>
      {label && <span className={styles.label}>{label}</span>}
    </div>
  );
}
