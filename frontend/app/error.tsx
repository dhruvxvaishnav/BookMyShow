'use client';
import { useEffect } from 'react';
import * as Sentry from '@sentry/nextjs';
import styles from './not-found.module.css';

export default function ErrorPage({
  error,
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  useEffect(() => {
    Sentry.captureException(error);
  }, [error]);

  return (
    <div className={styles.wrapper}>
      <div className={styles.filmStrip} aria-hidden="true">
        {Array.from({ length: 9 }).map((_, i) => (
          <div key={i} className={styles.filmHole} />
        ))}
      </div>
      <h1 className={styles.code}>500</h1>
      <hr className={`ornamental-divider ${styles.divider}`} />
      <h2 className={styles.subtitle}>Projection Interrupted</h2>
      <p className={styles.desc}>Something went wrong while loading this scene.</p>
      <button onClick={() => reset()} className={styles.homeBtn}>
        Try Again
      </button>
    </div>
  );
}
