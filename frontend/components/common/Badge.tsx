import styles from './Badge.module.css';

type BadgeVariant = 'success' | 'error' | 'warning' | 'info' | 'muted' | 'gold' | 'purple' | 'cyan';

interface BadgeProps {
  children: React.ReactNode;
  variant?: BadgeVariant;
  className?: string;
}

export default function Badge({ children, variant = 'info', className }: BadgeProps) {
  return (
    <span className={`${styles.badge} ${styles[variant]} ${className ?? ''}`}>
      {children}
    </span>
  );
}
