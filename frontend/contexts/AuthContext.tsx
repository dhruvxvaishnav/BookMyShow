'use client';
import React, { createContext, useCallback, useContext, useEffect, useState } from 'react';
import { login as apiLogin, register as apiRegister, adminLogin as apiAdminLogin } from '@/api/auth';
import type { AuthResponse } from '@/types/api';

interface AuthUser {
  user_id: string;
  email: string;
  user_name: string;
  role: 'user' | 'admin';
}

interface AuthContextValue {
  user: AuthUser | null;
  isLoading: boolean;
  isAdmin: boolean;
  login: (email: string, password: string) => Promise<void>;
  register: (email: string, password: string, user_name: string) => Promise<void>;
  adminLogin: (email: string, password: string) => Promise<void>;
  logout: () => void;
}

const AuthContext = createContext<AuthContextValue | null>(null);

function parseJwt(token: string): AuthUser | null {
  try {
    const payload = JSON.parse(atob(token.split('.')[1]));
    if (payload.exp && payload.exp * 1000 < Date.now()) return null;
    return {
      user_id: payload.sub,
      email: payload.email,
      user_name: payload.user_name,
      role: payload.role,
    };
  } catch {
    return null;
  }
}

function storeTokens(auth: AuthResponse) {
  localStorage.setItem('bms_access_token', auth.access_token);
  localStorage.setItem('bms_refresh_token', auth.refresh_token);
  if (auth.role === 'admin') {
    localStorage.setItem('bms_admin_token', auth.access_token);
  }
}

function clearTokens() {
  localStorage.removeItem('bms_access_token');
  localStorage.removeItem('bms_refresh_token');
  localStorage.removeItem('bms_admin_token');
}

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [user, setUser] = useState<AuthUser | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  // Hydrate from stored token on mount
  useEffect(() => {
    const token = localStorage.getItem('bms_access_token');
    if (token) {
      const parsed = parseJwt(token);
      setUser(parsed);
    }
    setIsLoading(false);
  }, []);

  const login = useCallback(async (email: string, password: string) => {
    const auth = await apiLogin(email, password);
    storeTokens(auth);
    setUser({
      user_id: auth.user_id,
      email: auth.email,
      user_name: auth.user_name,
      role: auth.role,
    });
  }, []);

  const register = useCallback(async (email: string, password: string, user_name: string) => {
    const auth = await apiRegister(email, password, user_name);
    storeTokens(auth);
    setUser({
      user_id: auth.user_id,
      email: auth.email,
      user_name: auth.user_name,
      role: auth.role,
    });
  }, []);

  const adminLoginFn = useCallback(async (email: string, password: string) => {
    const auth = await apiAdminLogin(email, password);
    storeTokens(auth);
    setUser({
      user_id: auth.user_id,
      email: auth.email,
      user_name: auth.user_name,
      role: auth.role,
    });
  }, []);

  const logout = useCallback(() => {
    clearTokens();
    setUser(null);
  }, []);

  return (
    <AuthContext.Provider
      value={{
        user,
        isLoading,
        isAdmin: user?.role === 'admin',
        login,
        register,
        adminLogin: adminLoginFn,
        logout,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth(): AuthContextValue {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error('useAuth must be used inside AuthProvider');
  return ctx;
}
