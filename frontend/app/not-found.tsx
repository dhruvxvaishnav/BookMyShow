import Link from 'next/link';
import { Film } from 'lucide-react';
import styles from './not-found.module.css';

export default function NotFound() {
  return (
    <div className={styles.wrapper}>
      <div className={styles.icon}>
        <Film size={48} strokeWidth={1} />
      </div>
      <h1 className={styles.title}>404</h1>
      <h2 className={styles.subtitle}>Page Not Found</h2>
      <p className={styles.desc}>
        The page you&apos;re looking for doesn&apos;t exist or has been moved.
      </p>
      <Link href="/" className={styles.homeBtn}>
        Back to Home
      </Link>
    </div>
  );
}