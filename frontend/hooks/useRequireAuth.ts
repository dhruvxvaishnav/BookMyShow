'use client';
import { useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { useAuth } from '@/contexts/AuthContext';

export function useRequireAuth(): boolean {
  const { user, isLoading } = useAuth();
  const router = useRouter();

  useEffect(() => {
    if (!isLoading && !user) {
      router.replace('/login');
    }
  }, [isLoading, user, router]);

  return !isLoading && !!user;
}

export function useRequireAdmin(): boolean {
  const { user, isLoading, isAdmin } = useAuth();
  const router = useRouter();

  useEffect(() => {
    if (!isLoading) {
      if (!user) router.replace('/admin/login');
      else if (!isAdmin) router.replace('/');
    }
  }, [isLoading, user, isAdmin, router]);

  return !isLoading && isAdmin;
}
