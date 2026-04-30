'use client';
import { SelectHTMLAttributes, forwardRef } from 'react';
import styles from './Select.module.css';

interface SelectProps extends SelectHTMLAttributes<HTMLSelectElement> {
  label?: string;
  error?: string;
  options: { value: string; label: string }[];
}

const Select = forwardRef<HTMLSelectElement, SelectProps>(
  ({ label, error, options, className, id, ...props }, ref) => {
    const selectId = id ?? label?.toLowerCase().replace(/\s+/g, '-');
    return (
      <div className={`${styles.field} ${className ?? ''}`}>
        {label && <label className={styles.label} htmlFor={selectId}>{label}</label>}
        <div className={styles.selectWrapper}>
          <select
            ref={ref}
            id={selectId}
            className={`${styles.select} ${error ? styles.selectError : ''}`}
            {...props}
          >
            {options.map((opt) => (
              <option key={opt.value} value={opt.value}>{opt.label}</option>
            ))}
          </select>
          <span className={styles.chevron}>
            <svg width="12" height="8" viewBox="0 0 12 8" fill="none">
              <path d="M1 1l5 5 5-5" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"/>
            </svg>
          </span>
        </div>
        {error && <span className={styles.error}>{error}</span>}
      </div>
    );
  }
);

Select.displayName = 'Select';
export default Select;
