'use client';
import { useRouter } from 'next/navigation';
import { User, Mail, Film, LogOut } from 'lucide-react';
import Link from 'next/link';
import { useAuth } from '@/contexts/AuthContext';
import styles from './page.module.css';

export default function ProfilePage() {
  const { user, logout } = useAuth();
  const router = useRouter();

  if (!user) {
    return (
      <div className={styles.page}>
        <div className="container">
          <div className={styles.emptyState}>
            <User size={48} strokeWidth={1} className={styles.emptyIcon} />
            <h2 className={styles.emptyTitle}>Not Signed In</h2>
            <p className={styles.emptyDesc}>Sign in to view your profile and booking history.</p>
            <Link href="/login" className={styles.loginBtn}>Sign In</Link>
          </div>
        </div>
      </div>
    );
  }

  const handleLogout = () => {
    logout();
    router.push('/');
  };

  const initials = user.user_name
    .split(' ')
    .map((n) => n[0])
    .join('')
    .toUpperCase()
    .slice(0, 2);

  return (
    <div className={styles.page}>
      <div className="container">
        <div className={styles.card}>
          <div className={styles.goldBar} />

          <div className={styles.avatarSection}>
            <div className={styles.avatar}>{initials}</div>
            <div>
              <h1 className={styles.name}>{user.user_name}</h1>
              <span className={styles.roleBadge}>{user.role === 'admin' ? 'Administrator' : 'Member'}</span>
            </div>
          </div>

          <hr className="ornamental-divider" />

          <div className={styles.infoGrid}>
            <div className={styles.infoItem}>
              <div className={styles.infoLabel}>
                <Mail size={14} strokeWidth={1.5} />
                Email Address
              </div>
              <div className={styles.infoValue}>{user.email}</div>
            </div>
            <div className={styles.infoItem}>
              <div className={styles.infoLabel}>
                <User size={14} strokeWidth={1.5} />
                User ID
              </div>
              <div className={styles.infoValueMono}>{user.user_id.slice(0, 20)}&hellip;</div>
            </div>
          </div>

          <hr className="ornamental-divider" />

          <div className={styles.actions}>
            <Link href="/my-bookings" className={styles.actionBtn}>
              <Film size={16} strokeWidth={1.5} />
              My Bookings
            </Link>
            {user.role === 'admin' && (
              <Link href="/admin" className={styles.adminBtn}>
                Admin Portal
              </Link>
            )}
            <button className={styles.logoutBtn} onClick={handleLogout}>
              <LogOut size={16} strokeWidth={1.5} />
              Sign Out
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
