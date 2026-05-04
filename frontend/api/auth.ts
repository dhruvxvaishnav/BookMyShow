import apiClient from './client';
import type { AuthResponse } from '@/types/api';

export async function register(
  email: string,
  password: string,
  user_name: string
): Promise<AuthResponse> {
  const res = await apiClient.post<AuthResponse>('/auth/register', {
    email,
    password,
    user_name,
  });
  return res.data;
}

export async function login(
  email: string,
  password: string
): Promise<AuthResponse> {
  const res = await apiClient.post<AuthResponse>('/auth/login', { email, password });
  return res.data;
}

export async function refreshAccessToken(refresh_token: string): Promise<AuthResponse> {
  const res = await apiClient.post<AuthResponse>('/auth/refresh', { refresh_token });
  return res.data;
}

export async function adminLogin(
  email: string,
  password: string
): Promise<AuthResponse> {
  const res = await apiClient.post<AuthResponse>('/admin/auth/login', { email, password });
  return res.data;
}
