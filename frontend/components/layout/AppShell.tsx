'use client';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { Clapperboard, User, Shield, LayoutGrid } from 'lucide-react';
import styles from './AppShell.module.css';

export default function AppShell({ children }: { children: React.ReactNode }) {
  const pathname = usePathname();
  const isAdmin = pathname?.startsWith('/admin');

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
                <Link href="/admin" className={styles.navLink}>
                  <Shield size={16} strokeWidth={1.5} />
                  Admin
                </Link>
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
