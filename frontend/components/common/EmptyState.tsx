import { Clapperboard, Film } from 'lucide-react';
import styles from './EmptyState.module.css';

interface EmptyStateProps {
  title: string;
  description?: string;
  icon?: 'film' | 'clapperboard';
  action?: React.ReactNode;
}

export default function EmptyState({ title, description, icon = 'film', action }: EmptyStateProps) {
  const Icon = icon === 'clapperboard' ? Clapperboard : Film;
  return (
    <div className={styles.wrapper}>
      <div className={styles.icon}>
        <Icon size={40} strokeWidth={1} />
      </div>
      <h3 className={styles.title}>{title}</h3>
      {description && <p className={styles.desc}>{description}</p>}
      {action && <div className={styles.action}>{action}</div>}
    </div>
  );
}
