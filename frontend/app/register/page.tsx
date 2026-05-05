'use client';
import { useState } from 'react';
import Link from 'next/link';
import { useRouter } from 'next/navigation';
import { Clapperboard, Mail, Lock, User, Eye, EyeOff } from 'lucide-react';
import { useAuth } from '@/contexts/AuthContext';
import { getErrorMessage } from '@/utils/error';
import { getPasswordStrengthError, isValidEmail } from '@/utils/validation';
import styles from './page.module.css';

export default function RegisterPage() {
  const router = useRouter();
  const { register } = useAuth();
  const [userName, setUserName] = useState('');
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [confirm, setConfirm] = useState('');
  const [showPassword, setShowPassword] = useState(false);
  const [showConfirm, setShowConfirm] = useState(false);
  const [error, setError] = useState('');
  const [isLoading, setIsLoading] = useState(false);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError('');
    if (!isValidEmail(email)) {
      setError('Enter a valid email address');
      return;
    }
    if (!userName.trim()) {
      setError('Full name is required');
      return;
    }
    if (password !== confirm) {
      setError('Passwords do not match');
      return;
    }
    const passwordError = getPasswordStrengthError(password);
    if (passwordError) {
      setError(passwordError);
      return;
    }
    setIsLoading(true);
    try {
      await register(email, password, userName);
      router.push('/');
    } catch (err) {
      setError(getErrorMessage(err));
    } finally {
      setIsLoading(false);
    }
  }

  return (
    <div className={styles.page}>
      <div className={styles.card}>
        {/* Crimson-to-gold top accent bar */}
        <div className="gold-accent-bar" />

        <div className={styles.cardInner}>
          {/* Logo area */}
          <div className={styles.logoArea}>
            <Clapperboard size={32} className={styles.logoIcon} strokeWidth={1.5} />
            <span className="marquee-label">Cineplex</span>
          </div>

          {/* Ornamental divider */}
          <hr className={`ornamental-divider ${styles.divider}`} />

          <h1 className={styles.title}>Create your account</h1>

          <form onSubmit={handleSubmit} className={styles.form}>
            {/* Full Name field */}
            <div className={styles.field}>
              <label className={styles.label}>Full Name</label>
              <div className={styles.inputWrap}>
                <User size={16} className={styles.inputIcon} />
                <input
                  type="text"
                  className={styles.input}
                  placeholder="Jane Doe"
                  value={userName}
                  onChange={(e) => setUserName(e.target.value)}
                  required
                  autoComplete="name"
                />
              </div>
            </div>

            {/* Email field */}
            <div className={styles.field}>
              <label className={styles.label}>Email Address</label>
              <div className={styles.inputWrap}>
                <Mail size={16} className={styles.inputIcon} />
                <input
                  type="email"
                  className={styles.input}
                  placeholder="you@example.com"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  required
                  autoComplete="email"
                />
              </div>
            </div>

            {/* Password field */}
            <div className={styles.field}>
              <label className={styles.label}>Password</label>
              <div className={styles.inputWrap}>
                <Lock size={16} className={styles.inputIcon} />
                <input
                  type={showPassword ? 'text' : 'password'}
                  className={`${styles.input} ${styles.inputPassword}`}
                  placeholder="Min 8 characters"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  required
                  autoComplete="new-password"
                />
                <button
                  type="button"
                  className={styles.inputIconRight}
                  onClick={() => setShowPassword((v) => !v)}
                  aria-label={showPassword ? 'Hide password' : 'Show password'}
                >
                  {showPassword ? <EyeOff size={16} /> : <Eye size={16} />}
                </button>
              </div>
            </div>

            {/* Confirm Password field */}
            <div className={styles.field}>
              <label className={styles.label}>Confirm Password</label>
              <div className={styles.inputWrap}>
                <Lock size={16} className={styles.inputIcon} />
                <input
                  type={showConfirm ? 'text' : 'password'}
                  className={`${styles.input} ${styles.inputPassword}`}
                  placeholder="Repeat password"
                  value={confirm}
                  onChange={(e) => setConfirm(e.target.value)}
                  required
                  autoComplete="new-password"
                />
                <button
                  type="button"
                  className={styles.inputIconRight}
                  onClick={() => setShowConfirm((v) => !v)}
                  aria-label={showConfirm ? 'Hide confirm password' : 'Show confirm password'}
                >
                  {showConfirm ? <EyeOff size={16} /> : <Eye size={16} />}
                </button>
              </div>
            </div>

            {error && <p className={styles.error}>{error}</p>}

            <button type="submit" className={styles.button} disabled={isLoading}>
              {isLoading ? 'Creating account…' : 'Create Account →'}
            </button>
          </form>

          <p className={styles.footer}>
            Already have an account?{' '}
            <Link href="/login" className={styles.link}>
              Sign in
            </Link>
          </p>
        </div>
      </div>
    </div>
  );
}
