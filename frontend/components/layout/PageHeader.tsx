'use client';
import Link from 'next/link';
import { ArrowLeft } from 'lucide-react';
import styles from './PageHeader.module.css';

interface PageHeaderProps {
  title: string;
  subtitle?: string;
  backHref?: string;
  actions?: React.ReactNode;
}

export default function PageHeader({ title, subtitle, backHref, actions }: PageHeaderProps) {
  return (
    <div className={styles.header}>
      <div className={`container ${styles.inner}`}>
        <div className={styles.left}>
          {backHref && (
            <Link href={backHref} className={styles.backBtn}>
              <ArrowLeft size={18} strokeWidth={1.5} />
            </Link>
          )}
          <div className={styles.titles}>
            <h1 className={styles.title}>{title}</h1>
            {subtitle && <p className={styles.subtitle}>{subtitle}</p>}
          </div>
        </div>
        {actions && <div className={styles.actions}>{actions}</div>}
      </div>
    </div>
  );
}
