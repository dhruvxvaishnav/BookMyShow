'use client';
import { useState } from 'react';
import { useRouter } from 'next/navigation';
import { Shield, Mail, Lock } from 'lucide-react';
import { useAuth } from '@/contexts/AuthContext';
import { getErrorMessage } from '@/utils/error';
import { isValidEmail } from '@/utils/validation';
import styles from './page.module.css';

export default function AdminLoginPage() {
  const router = useRouter();
  const { adminLogin } = useAuth();
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [error, setError] = useState('');
  const [isLoading, setIsLoading] = useState(false);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError('');
    if (!isValidEmail(email)) {
      setError('Enter a valid email address');
      return;
    }
    if (!password) {
      setError('Password is required');
      return;
    }
    setIsLoading(true);
    try {
      await adminLogin(email, password);
      router.push('/admin');
    } catch (err) {
      setError(getErrorMessage(err));
    } finally {
      setIsLoading(false);
    }
  }

  return (
    <div className={styles.page}>
      <div className={styles.card}>
        {/* Gold accent bar */}
        <div className="gold-accent-bar" />

        <div className={styles.cardInner}>
          {/* Logo / badge */}
          <div className={styles.logoWrap}>
            <div className={styles.shieldCircle}>
              <Shield size={28} strokeWidth={1.5} />
            </div>
            <span className={`marquee-label ${styles.portalLabel}`}>Admin Portal</span>
          </div>

          <div className="ornamental-divider" style={{ margin: '0 0 28px' }} />

          <h1 className={styles.title}>Administrator Sign In</h1>
          <p className={styles.subtitle}>Restricted access — authorised personnel only</p>

          <form onSubmit={handleSubmit} className={styles.form}>
            <div className={styles.field}>
              <label className={styles.label} htmlFor="admin-email">Email Address</label>
              <div className={styles.inputWrap}>
                <Mail size={15} className={styles.inputIcon} />
                <input
                  id="admin-email"
                  type="email"
                  className={styles.input}
                  placeholder="admin@cineplex.com"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  required
                  autoComplete="email"
                />
              </div>
            </div>

            <div className={styles.field}>
              <label className={styles.label} htmlFor="admin-password">Password</label>
              <div className={styles.inputWrap}>
                <Lock size={15} className={styles.inputIcon} />
                <input
                  id="admin-password"
                  type="password"
                  className={styles.input}
                  placeholder="••••••••"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  required
                  autoComplete="current-password"
                />
              </div>
            </div>

            {error && (
              <div className={styles.errorBox} role="alert">
                {error}
              </div>
            )}

            <button type="submit" className={styles.submitBtn} disabled={isLoading}>
              {isLoading ? (
                <span className={styles.loadingDots}>Authenticating</span>
              ) : (
                'Sign In to Dashboard'
              )}
            </button>
          </form>
        </div>
      </div>
    </div>
  );
}
