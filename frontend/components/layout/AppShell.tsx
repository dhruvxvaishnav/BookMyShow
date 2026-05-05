'use client';
import { useState } from 'react';
import Link from 'next/link';
import { usePathname, useRouter } from 'next/navigation';
import {
  LayoutDashboard, Film, BookOpen, ClipboardList, LogOut, User, Shield,
  Check,
} from 'lucide-react';
import { useAuth } from '@/contexts/AuthContext';
import styles from './AppShell.module.css';

const ADMIN_TOKEN_KEY = 'bms_admin_token';
const DEFAULT_ADMIN_TOKEN = 'admin-secret';

export default function AppShell({ children }: { children: React.ReactNode }) {
  const pathname = usePathname();
  const router = useRouter();
  const { user, logout } = useAuth();
  const isAdmin = pathname?.startsWith('/admin');

  // Legacy admin token for the token-based admin access
  const [tokenInput, setTokenInput] = useState('');
  const [showTokenInput, setShowTokenInput] = useState(false);
  const [adminToken, setAdminToken] = useState(() => {
    if (typeof window !== 'undefined') {
      return localStorage.getItem(ADMIN_TOKEN_KEY) ?? DEFAULT_ADMIN_TOKEN;
    }
    return DEFAULT_ADMIN_TOKEN;
  });

  const saveToken = () => {
    const token = tokenInput.trim() || DEFAULT_ADMIN_TOKEN;
    localStorage.setItem(ADMIN_TOKEN_KEY, token);
    setAdminToken(token);
    setShowTokenInput(false);
    setTokenInput('');
  };

  const handleLogout = () => {
    logout();
    router.push('/');
  };

  if (isAdmin) {
    return (
      <div className={styles.adminShell}>
        {/* Admin Sidebar */}
        <aside className={styles.adminSidebar}>
          <div className={styles.sidebarLogo}>
            <Link href="/admin" className={styles.sidebarBrand}>CINEPLEX</Link>
            <div className={styles.sidebarLabel}>Admin Portal</div>
          </div>

          <nav className={styles.sidebarNav}>
            <Link
              href="/admin"
              className={pathname === '/admin' ? styles.sidebarActive : styles.sidebarLink}
            >
              <LayoutDashboard size={16} strokeWidth={1.5} />
              Dashboard
            </Link>
            <Link
              href="/admin/shows/new"
              className={pathname === '/admin/shows/new' ? styles.sidebarActive : styles.sidebarLink}
            >
              <Film size={16} strokeWidth={1.5} />
              Shows
            </Link>
            <Link
              href="/admin/bookings"
              className={pathname?.startsWith('/admin/bookings') ? styles.sidebarActive : styles.sidebarLink}
            >
              <BookOpen size={16} strokeWidth={1.5} />
              Bookings
            </Link>
            <Link
              href="/admin/audit-logs"
              className={pathname === '/admin/audit-logs' ? styles.sidebarActive : styles.sidebarLink}
            >
              <ClipboardList size={16} strokeWidth={1.5} />
              Audit Logs
            </Link>
          </nav>

          <div className={styles.sidebarBottom}>
            {user && (
              <div className={styles.sidebarUser}>{user.email}</div>
            )}
            <Link href="/" className={styles.sidebarLink} style={{ borderLeft: 'none', padding: 0 }}>
              User View
            </Link>
            {user ? (
              <button className={styles.sidebarLogout} onClick={handleLogout}>
                <LogOut size={14} strokeWidth={1.5} />
                Logout
              </button>
            ) : (
              <Link href="/admin/login" className={styles.sidebarLink} style={{ borderLeft: 'none', padding: 0, color: 'var(--antique-gold)' }}>
                <Shield size={14} strokeWidth={1.5} />
                Login
              </Link>
            )}
          </div>
        </aside>

        {/* Admin Content */}
        <main className={styles.adminMain}>
          {children}
        </main>
      </div>
    );
  }

  // ── User-facing layout ──
  return (
    <div className={styles.shell}>
      <header className={styles.header}>
        <div className={`container ${styles.headerInner}`}>
          {/* Brand */}
          <Link href="/" className={styles.brand}>
            CINEPLEX
          </Link>

          {/* Centre nav */}
          <nav className={styles.nav}>
            <Link
              href="/"
              className={pathname === '/' ? styles.navActive : styles.navLink}
            >
              Home
            </Link>
            <Link
              href="/movies"
              className={pathname?.startsWith('/movies') ? styles.navActive : styles.navLink}
            >
              Movies
            </Link>
            <Link
              href="/my-bookings"
              className={pathname === '/my-bookings' ? styles.navActive : styles.navLink}
            >
              My Bookings
            </Link>
          </nav>

          {/* Right: auth + admin */}
          <div className={styles.navRight}>
            {/* Legacy admin token toggle */}
            {showTokenInput ? (
              <div className={styles.tokenWrap}>
                <input
                  className={styles.tokenInput}
                  placeholder="admin token"
                  value={tokenInput}
                  onChange={(e) => setTokenInput(e.target.value)}
                  onKeyDown={(e) => e.key === 'Enter' && saveToken()}
                  autoFocus
                />
                <button className={styles.tokenSaveBtn} onClick={saveToken}>Set</button>
                <button
                  className={styles.tokenCancelBtn}
                  onClick={() => setShowTokenInput(false)}
                  aria-label="Cancel"
                >×</button>
              </div>
            ) : (
              <button
                className={`${styles.navLink} ${styles.adminBtn}`}
                onClick={() => setShowTokenInput(true)}
                title="Admin access"
                aria-label="Admin token"
              >
                {adminToken !== DEFAULT_ADMIN_TOKEN || adminToken ? (
                  <span className={styles.tokenSet}>
                    <Check size={13} strokeWidth={2} />
                    <Shield size={15} strokeWidth={1.5} />
                  </span>
                ) : (
                  <Shield size={15} strokeWidth={1.5} />
                )}
              </button>
            )}

            {user ? (
              <>
                <Link href="/profile" className={styles.userBtn}>
                  <User size={14} strokeWidth={1.5} />
                  {user.user_name}
                </Link>
                <button
                  className={styles.navLink}
                  onClick={handleLogout}
                  style={{ background: 'none', border: 'none', cursor: 'pointer', fontFamily: 'inherit' }}
                >
                  Logout
                </button>
              </>
            ) : (
              <Link href="/login" className={styles.loginBtn}>
                Login
              </Link>
            )}
          </div>
        </div>
      </header>

      <main className={`${styles.main} page-content`}>
        {children}
      </main>

      <footer className={styles.footer}>
        <div className="container">
          <div className={styles.footerInner}>
            <div className={styles.footerBrand}>CINEPLEX</div>
            <hr className="ornamental-divider" style={{ margin: '8px 0', width: '200px' }} />
            <p className={styles.footerText}>
              &copy; {new Date().getFullYear()} Cineplex — powered by Rust &amp; Next.js
            </p>
          </div>
        </div>
      </footer>
    </div>
  );
}
