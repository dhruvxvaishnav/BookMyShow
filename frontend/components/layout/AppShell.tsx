'use client';
import { useState, useEffect } from 'react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { Clapperboard, User, Shield, LayoutGrid, Check } from 'lucide-react';
import styles from './AppShell.module.css';

const ADMIN_TOKEN_KEY = 'bms_admin_token';
const DEFAULT_ADMIN_TOKEN = 'admin-secret';

export default function AppShell({ children }: { children: React.ReactNode }) {
  const pathname = usePathname();
  const isAdmin = pathname?.startsWith('/admin');
  const [adminToken, setAdminToken] = useState('');
  const [tokenInput, setTokenInput] = useState('');
  const [showTokenInput, setShowTokenInput] = useState(false);

  useEffect(() => {
    setAdminToken(localStorage.getItem(ADMIN_TOKEN_KEY) ?? DEFAULT_ADMIN_TOKEN);
  }, []);

  const saveToken = () => {
    const token = tokenInput.trim() || DEFAULT_ADMIN_TOKEN;
    localStorage.setItem(ADMIN_TOKEN_KEY, token);
    setAdminToken(token);
    setShowTokenInput(false);
    setTokenInput('');
  };

  const isTokenSet = adminToken === DEFAULT_ADMIN_TOKEN || adminToken.length > 0;

  return (
    <div className={styles.shell}>
      <header className={styles.header}>
        <div className={`container ${styles.headerInner}`}>
          <Link href="/" className={styles.brand}>
            <Clapperboard size={22} strokeWidth={1.5} />
            <span>BookMyShow</span>
          </Link>

          <nav className={styles.nav}>
            {!isAdmin && (
              <>
                <Link href="/" className={pathname === '/' ? styles.navActive : styles.navLink}>
                  <LayoutGrid size={16} strokeWidth={1.5} />
                  Browse Shows
                </Link>
                <Link href="/my-bookings" className={pathname === '/my-bookings' ? styles.navActive : styles.navLink}>
                  <User size={16} strokeWidth={1.5} />
                  My Bookings
                </Link>
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
                    <button className={styles.tokenCancelBtn} onClick={() => setShowTokenInput(false)}>×</button>
                  </div>
                ) : (
                  <button
                    className={`${styles.navLink} ${styles.adminBtn}`}
                    onClick={() => setShowTokenInput(true)}
                    title={`Admin token: ${adminToken ? 'set' : 'not set'}`}
                  >
                    {isTokenSet ? (
                      <span className={styles.tokenSet}>
                        <Check size={14} strokeWidth={2} />
                        <Shield size={16} strokeWidth={1.5} />
                      </span>
                    ) : (
                      <Shield size={16} strokeWidth={1.5} />
                    )}
                    Admin
                  </button>
                )}
              </>
            )}
            {isAdmin && (
              <>
                <Link href="/admin" className={pathname === '/admin' ? styles.navActive : styles.navLink}>Dashboard</Link>
                <Link href="/admin/shows/new" className={pathname === '/admin/shows/new' ? styles.navActive : styles.navLink}>Create Show</Link>
                <Link href="/" className={styles.navLink}>User View</Link>
              </>
            )}
          </nav>
        </div>
      </header>

      <main className={`${styles.main} page-content`}>
        {children}
      </main>

      <footer className={styles.footer}>
        <div className="container">
          <p className={styles.footerText}>
            &copy; {new Date().getFullYear()} BookMyShow &mdash; powered by Rust &amp; Next.js
          </p>
        </div>
      </footer>
    </div>
  );
}
