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

    // User auth
    const userId = localStorage.getItem('bms_user_id');
    if (userId && config.headers) {
      config.headers['X-User-Id'] = userId;
    }

    // Admin auth
    const adminToken = localStorage.getItem('bms_admin_token');
    if (adminToken && config.headers) {
      config.headers['X-Admin-Token'] = adminToken;
    }

    return config;
  },
  (error) => Promise.reject(error)
);

// ── Response interceptor: unwrap ApiResponse envelope ─────
apiClient.interceptors.response.use(
  (response) => {
    const data = response.data as ApiResponse<unknown>;
    if (data && 'success' in data) {
      if (data.success && data.data !== undefined) {
        response.data = data.data;
        return response;
      } else if (!data.success && data.error) {
        const err = new ApiError(
          data.error.code,
          data.error.message,
          data.error.details,
          response.status
        );
        return Promise.reject(err);
      }
    }
    return response;
  },
  (error: AxiosError<ApiResponse<unknown>>) => {
    if (error.response) {
      const data = error.response.data;
      if (data && data.error) {
        return Promise.reject(
          new ApiError(
            data.error.code,
            data.error.message,
            data.error.details,
            error.response.status
          )
        );
      }
      return Promise.reject(
        new ApiError(
          'HTTP_ERROR',
          `Server error: ${error.response.status}`,
          undefined,
          error.response.status
        )
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
