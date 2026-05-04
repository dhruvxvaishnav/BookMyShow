'use client';
import { useState, useEffect } from 'react';

function getUserIdFromJwt(): string | null {
  try {
    const token = localStorage.getItem('bms_access_token');
    if (!token) return null;
    const payload = JSON.parse(atob(token.split('.')[1]));
    if (payload.exp && payload.exp * 1000 < Date.now()) return null;
    return payload.sub ?? null;
  } catch {
    return null;
  }
}

export function useUserId(): string {
  const [userId, setUserId] = useState<string>('');

  useEffect(() => {
    const jwtId = getUserIdFromJwt();
    if (jwtId) {
      setUserId(jwtId);
      return;
    }
    let id = localStorage.getItem('bms_user_id');
    if (!id) {
      id = crypto.randomUUID();
      localStorage.setItem('bms_user_id', id);
    }
    setUserId(id);
  }, []);

  return userId;
}
