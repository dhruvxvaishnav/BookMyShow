import Link from 'next/link';
import styles from './not-found.module.css';

export default function NotFound() {
  return (
    <div className={styles.wrapper}>
      {/* Film strip decoration — pure CSS */}
      <div className={styles.filmStrip} aria-hidden="true">
        {Array.from({ length: 9 }).map((_, i) => (
          <div key={i} className={styles.filmHole} />
        ))}
      </div>

      {/* Large 404 */}
      <h1 className={styles.code}>404</h1>

      {/* Ornamental divider */}
      <hr className={`ornamental-divider ${styles.divider}`} />

      <h2 className={styles.subtitle}>Scene Not Found</h2>

      <p className={styles.desc}>
        The page you&apos;re looking for doesn&apos;t exist or has been moved.
      </p>

      <Link href="/" className={styles.homeBtn}>
        Back to Home
      </Link>
    </div>
  );
}
