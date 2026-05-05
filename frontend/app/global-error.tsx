'use client';

import * as Sentry from '@sentry/nextjs';
import { useEffect } from 'react';
import styles from './global-error.module.css';

export default function GlobalError({
  error,
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  useEffect(() => {
    // Log the error to Sentry
    Sentry.captureException(error);
  }, [error]);

  return (
    <html>
      <body>
        <div className={styles.wrapper}>
          <div className={styles.filmStrip} aria-hidden="true">
            {Array.from({ length: 9 }).map((_, i) => (
              <div key={i} className={styles.filmHole} />
            ))}
          </div>
          <h1 className={styles.code}>500</h1>
          <hr className={styles.divider} />
          <h2 className={styles.subtitle}>Projection Interrupted</h2>
          <p className={styles.desc}>Something went wrong while loading this scene.</p>
          <button onClick={() => reset()} className={styles.actionBtn}>
            Try Again
          </button>
        </div>
      </body>
    </html>
  );
}
