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
    pub distributed_lock: DistributedLockConfig,
    pub jwt: JwtConfig,
    pub email: EmailConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmailConfig {
    pub api_key: Option<String>,
    pub from_address: String,
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
    pub stripe_secret_key: Option<String>,
    pub stripe_webhook_secret: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitConfig {
    pub lock_requests_per_min: usize,
    pub payment_requests_per_min: usize,
    pub default_requests_per_min: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DistributedLockConfig {
    pub redis_url: Option<String>,
    pub lock_ttl_ms: u64,
    pub renewal_interval_ms: u64,
    pub acquire_timeout_ms: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    pub secret: String,
    pub access_token_expiry_secs: u64,
    pub refresh_token_expiry_secs: u64,
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
                stripe_secret_key: None,
                stripe_webhook_secret: None,
            },
            rate_limit: RateLimitConfig {
                lock_requests_per_min: 5,
                payment_requests_per_min: 3,
                default_requests_per_min: 60,
            },
            distributed_lock: DistributedLockConfig {
                redis_url: None,
                lock_ttl_ms: 10_000,
                renewal_interval_ms: 3_000,
                acquire_timeout_ms: 2_000,
            },
            jwt: JwtConfig {
                secret: "dev-jwt-secret-change-in-production-must-be-at-least-32-chars".to_string(),
                access_token_expiry_secs: 900,      // 15 min
                refresh_token_expiry_secs: 604_800, // 7 days
            },
            email: EmailConfig {
                api_key: None,
                from_address: "tickets@bookmyshow.local".to_string(),
            },
        }
    }
}

impl AppConfig {
    /// Load configuration from config.toml, with optional env-var overrides.
    /// Env vars take precedence: e.g. `APP_PORT=9000 cargo run`
    /// Falls back to compiled-in defaults if config.toml is absent or incomplete.
    pub fn load() -> Result<Self, ConfigError> {
        // config.toml is optional — missing file is not an error
        let mut builder = Config::builder().add_source(File::with_name("config").required(false));

        // Env-var overrides use set_override (builder API, not deprecated)
        macro_rules! env_override {
            ($key:literal, $var:literal) => {
                if let Ok(v) = env::var($var) {
                    builder = builder.set_override($key, v)?;
                }
            };
        }

        env_override!("app.port", "APP_PORT");
        env_override!("app.host", "APP_HOST");
        env_override!("app.log_level", "LOG_LEVEL");
        env_override!("seat_lock.ttl_seconds", "SEAT_LOCK_TTL_SECS");
        env_override!("seat_lock.max_extensions", "SEAT_LOCK_MAX_EXTENSIONS");
        env_override!("seat_lock.extension_seconds", "SEAT_LOCK_EXTENSION_SECS");
        env_override!(
            "seat_lock.grace_period_seconds",
            "SEAT_LOCK_GRACE_PERIOD_SECS"
        );
        env_override!("jwt.secret", "JWT_SECRET");
        env_override!("jwt.access_token_expiry_secs", "JWT_ACCESS_EXPIRY_SECS");
        env_override!("jwt.refresh_token_expiry_secs", "JWT_REFRESH_EXPIRY_SECS");
        env_override!("payment.stripe_secret_key", "STRIPE_SECRET_KEY");
        env_override!("payment.stripe_webhook_secret", "STRIPE_WEBHOOK_SECRET");
        env_override!("email.api_key", "EMAIL_API_KEY");
        env_override!("email.from_address", "EMAIL_FROM_ADDRESS");
        env_override!("distributed_lock.redis_url", "REDIS_URL");
        env_override!("distributed_lock.lock_ttl_ms", "DISTRIBUTED_LOCK_TTL_MS");
        env_override!(
            "distributed_lock.renewal_interval_ms",
            "DISTRIBUTED_LOCK_RENEWAL_INTERVAL_MS"
        );
        env_override!(
            "distributed_lock.acquire_timeout_ms",
            "DISTRIBUTED_LOCK_ACQUIRE_TIMEOUT_MS"
        );

        // If deserialization fails (e.g. no config.toml and no env vars), use defaults
        match builder.build()?.try_deserialize::<Self>() {
            Ok(cfg) => Ok(cfg),
            Err(_) => Ok(Self::default()),
        }
    }

    /// Returns a globally initialised config instance.
    pub fn init() -> Result<&'static Self, ConfigError> {
        let cfg = Self::load()?;
        CONFIG.set(cfg).expect("config already initialised");
        Ok(CONFIG.get().unwrap())
    }

    /// Returns the global config. Panics if not initialised via `init()`.
    pub fn get() -> &'static Self {
        CONFIG
            .get()
            .expect("config not initialised — call AppConfig::init() first")
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
        assert_eq!(cfg.distributed_lock.lock_ttl_ms, 10_000);
    }
}
