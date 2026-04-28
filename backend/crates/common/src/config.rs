use config::{Config, ConfigError, File};
use serde::Deserialize;
use std::env;
use std::sync::OnceLock;

static CONFIG: OnceLock<AppConfig> = OnceLock::new();

/// Root application configuration, loaded from config.toml with environment-variable overrides.
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub app: AppSettings,
    pub seat_lock: SeatLockConfig,
    pub queue: QueueConfig,
    pub payment: PaymentConfig,
    pub rate_limit: RateLimitConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppSettings {
    pub host: String,
    pub port: u16,
    pub log_level: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SeatLockConfig {
    pub ttl_seconds: u64,
    pub max_extensions: u32,
    pub extension_seconds: u64,
    pub grace_period_seconds: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QueueConfig {
    pub processing_timeout_seconds: u64,
    pub max_concurrent_per_show: usize,
    pub poll_interval_ms: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaymentConfig {
    pub timeout_seconds: u64,
    pub mock_gateway_delay_ms: u64,
    pub mock_gateway_failure_rate: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitConfig {
    pub lock_requests_per_min: usize,
    pub payment_requests_per_min: usize,
    pub default_requests_per_min: usize,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            app: AppSettings {
                host: "0.0.0.0".to_string(),
                port: 8080,
                log_level: "info".to_string(),
            },
            seat_lock: SeatLockConfig {
                ttl_seconds: 300,
                max_extensions: 2,
                extension_seconds: 120,
                grace_period_seconds: 30,
            },
            queue: QueueConfig {
                processing_timeout_seconds: 10,
                max_concurrent_per_show: 3,
                poll_interval_ms: 500,
            },
            payment: PaymentConfig {
                timeout_seconds: 600,
                mock_gateway_delay_ms: 2000,
                mock_gateway_failure_rate: 0.2,
            },
            rate_limit: RateLimitConfig {
                lock_requests_per_min: 5,
                payment_requests_per_min: 3,
                default_requests_per_min: 60,
            },
        }
    }
}

impl AppConfig {
    /// Load configuration from config.toml, with optional env-var overrides.
    /// Env vars take precedence: e.g. `APP_PORT=9000 cargo run`
    pub fn load() -> Result<Self, ConfigError> {
        let mut cfg = Config::builder()
            .add_source(File::with_name("config"))
            .build()
            .unwrap_or_else(|_| {
                // Fallback to default if config.toml is missing (useful during dev)
                tracing::warn!("config.toml not found; using default configuration");
                Config::builder().build().unwrap()
            });

        // Environment variable overrides
        if let Ok(port) = env::var("APP_PORT") {
            let _ = cfg.set("app.port", port);
        }
        if let Ok(host) = env::var("APP_HOST") {
            let _ = cfg.set("app.host", host);
        }
        if let Ok(level) = env::var("LOG_LEVEL") {
            let _ = cfg.set("app.log_level", level);
        }
        if let Ok(ttl) = env::var("SEAT_LOCK_TTL_SECS") {
            let _ = cfg.set("seat_lock.ttl_seconds", ttl);
        }
        if let Ok(max_ext) = env::var("SEAT_LOCK_MAX_EXTENSIONS") {
            let _ = cfg.set("seat_lock.max_extensions", max_ext);
        }
        if let Ok(ext_secs) = env::var("SEAT_LOCK_EXTENSION_SECS") {
            let _ = cfg.set("seat_lock.extension_seconds", ext_secs);
        }
        if let Ok(grace) = env::var("SEAT_LOCK_GRACE_PERIOD_SECS") {
            let _ = cfg.set("seat_lock.grace_period_seconds", grace);
        }

        cfg.try_deserialize()
    }

    /// Returns a globally initialised config instance.
    pub fn init() -> Result<&'static Self, ConfigError> {
        let cfg = Self::load()?;
        CONFIG.set(cfg).expect("config already initialised");
        Ok(CONFIG.get().unwrap())
    }

    /// Returns the global config. Panics if not initialised via `init()`.
    pub fn get() -> &'static Self {
        CONFIG.get().expect("config not initialised — call AppConfig::init() first")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = AppConfig::default();
        assert_eq!(cfg.app.port, 8080);
        assert_eq!(cfg.seat_lock.ttl_seconds, 300);
        assert_eq!(cfg.seat_lock.max_extensions, 2);
        assert_eq!(cfg.queue.max_concurrent_per_show, 3);
        assert_eq!(cfg.payment.timeout_seconds, 600);
    }
}
