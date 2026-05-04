import axios, { AxiosError, InternalAxiosRequestConfig } from 'axios';
import { ApiError, ApiResponse } from '@/types/api';

const BASE_URL = process.env.NEXT_PUBLIC_API_BASE_URL ?? 'http://localhost:8080';

export const apiClient = axios.create({
  baseURL: BASE_URL,
  timeout: 10000,
  headers: { 'Content-Type': 'application/json' },
});

// ── Request interceptor: inject auth headers ─────────────
apiClient.interceptors.request.use(
  (config: InternalAxiosRequestConfig) => {
    if (typeof window === 'undefined') return config;

    // JWT user auth (preferred)
    const accessToken = localStorage.getItem('bms_access_token');
    if (accessToken && config.headers) {
      config.headers['Authorization'] = `Bearer ${accessToken}`;
    }

    // Admin JWT token
    const adminToken = localStorage.getItem('bms_admin_token');
    if (adminToken && config.headers) {
      config.headers['X-Admin-Token'] = adminToken;
    }

    return config;
  },
  (error) => Promise.reject(error)
);

let isRefreshing = false;
let refreshQueue: Array<(token: string) => void> = [];

function drainQueue(token: string) {
  refreshQueue.forEach((cb) => cb(token));
  refreshQueue = [];
}

// ── Response interceptor: unwrap ApiResponse envelope + auto-refresh ─────
apiClient.interceptors.response.use(
  (response) => {
    const data = response.data as ApiResponse<unknown>;
    if (data && 'success' in data) {
      if (data.success && data.data !== undefined) {
        response.data = data.data;
        return response;
      } else if (!data.success && data.error) {
        return Promise.reject(
          new ApiError(data.error.code, data.error.message, data.error.details, response.status)
        );
      }
    }
    return response;
  },
  async (error: AxiosError<ApiResponse<unknown>>) => {
    const original = error.config as InternalAxiosRequestConfig & { _retry?: boolean };

    // Auto-refresh on 401 (skip for auth endpoints themselves)
    if (
      error.response?.status === 401 &&
      !original?._retry &&
      original?.url &&
      !original.url.includes('/auth/')
    ) {
      const refreshToken = typeof window !== 'undefined'
        ? localStorage.getItem('bms_refresh_token')
        : null;

      if (refreshToken) {
        if (isRefreshing) {
          return new Promise<string>((resolve) => {
            refreshQueue.push(resolve);
          }).then((token) => {
            if (original.headers) original.headers['Authorization'] = `Bearer ${token}`;
            return apiClient(original);
          });
        }

        isRefreshing = true;
        original._retry = true;

        try {
          const res = await axios.post<{ success: boolean; data: { access_token: string; refresh_token: string } }>(
            `${BASE_URL}/auth/refresh`,
            { refresh_token: refreshToken },
            { headers: { 'Content-Type': 'application/json' } }
          );
          const newAccess = res.data.data.access_token;
          const newRefresh = res.data.data.refresh_token;
          localStorage.setItem('bms_access_token', newAccess);
          localStorage.setItem('bms_refresh_token', newRefresh);
          drainQueue(newAccess);
          if (original.headers) original.headers['Authorization'] = `Bearer ${newAccess}`;
          return apiClient(original);
        } catch {
          // Refresh failed — clear tokens and let the app redirect to login
          localStorage.removeItem('bms_access_token');
          localStorage.removeItem('bms_refresh_token');
          drainQueue('');
          window.location.href = '/login';
        } finally {
          isRefreshing = false;
        }
      }
    }

    if (error.response) {
      const data = error.response.data;
      if (data && data.error) {
        return Promise.reject(
          new ApiError(data.error.code, data.error.message, data.error.details, error.response.status)
        );
      }
      return Promise.reject(
        new ApiError('HTTP_ERROR', `Server error: ${error.response.status}`, undefined, error.response.status)
      );
    }
    if (error.code === 'ECONNABORTED') {
      return Promise.reject(new ApiError('TIMEOUT', 'Request timed out. Please try again.'));
    }
    if (!error.response) {
      return Promise.reject(new ApiError('NETWORK_ERROR', 'Unable to connect. Check your internet connection.'));
    }
    return Promise.reject(error);
  }
);

export default apiClient;
