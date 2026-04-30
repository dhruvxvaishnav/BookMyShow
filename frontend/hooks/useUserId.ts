'use client';
import { useState, useEffect } from 'react';

/**
 * Get or generate a persistent user UUID from localStorage.
 */
export function useUserId(): string {
  const [userId, setUserId] = useState<string>('');

  useEffect(() => {
    let id = localStorage.getItem('bms_user_id');
    if (!id) {
      id = crypto.randomUUID();
      localStorage.setItem('bms_user_id', id);
    }
    setUserId(id);
  }, []);

  return userId;
}
