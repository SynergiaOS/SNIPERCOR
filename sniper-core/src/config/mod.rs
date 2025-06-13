// ⚙️ Configuration Management
// Centralized configuration for the HFT system

use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub solana: SolanaConfig,
    pub trading: TradingConfig,
    pub cache: CacheConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub worker_threads: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaConfig {
    pub rpc_url: String,
    pub ws_url: String,
    pub private_key: String,
    pub commitment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingConfig {
    pub mode: String, // "paper" or "live"
    pub max_position_size: f64,
    pub max_daily_loss: f64,
    pub min_confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub redis_url: String,
    pub default_ttl: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String, // "json" or "text"
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            server: ServerConfig {
                host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("SERVER_PORT")
                    .unwrap_or_else(|_| "8003".to_string())
                    .parse()
                    .unwrap_or(8003),
                worker_threads: env::var("TOKIO_WORKER_THREADS")
                    .unwrap_or_else(|_| "5".to_string())
                    .parse()
                    .unwrap_or(5),
            },
            solana: SolanaConfig {
                rpc_url: env::var("QUICKNODE_RPC_URL")
                    .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string()),
                ws_url: env::var("HELIUS_WS_URL")
                    .unwrap_or_else(|_| "wss://atlas-mainnet.helius-rpc.com".to_string()),
                private_key: env::var("SOLANA_PRIVATE_KEY")
                    .map_err(|_| anyhow::anyhow!("SOLANA_PRIVATE_KEY must be set"))?,
                commitment: env::var("SOLANA_COMMITMENT")
                    .unwrap_or_else(|_| "confirmed".to_string()),
            },
            trading: TradingConfig {
                mode: env::var("TRADING_MODE").unwrap_or_else(|_| "paper".to_string()),
                max_position_size: env::var("MAX_POSITION_SIZE")
                    .unwrap_or_else(|_| "1000.0".to_string())
                    .parse()
                    .unwrap_or(1000.0),
                max_daily_loss: env::var("MAX_DAILY_LOSS")
                    .unwrap_or_else(|_| "500.0".to_string())
                    .parse()
                    .unwrap_or(500.0),
                min_confidence: env::var("MIN_CONFIDENCE")
                    .unwrap_or_else(|_| "0.6".to_string())
                    .parse()
                    .unwrap_or(0.6),
            },
            cache: CacheConfig {
                redis_url: env::var("DRAGONFLY_URL")
                    .unwrap_or_else(|_| "redis://dragonfly:6379".to_string()),
                default_ttl: env::var("CACHE_DEFAULT_TTL")
                    .unwrap_or_else(|_| "3600".to_string())
                    .parse()
                    .unwrap_or(3600),
            },
            logging: LoggingConfig {
                level: env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
                format: env::var("LOG_FORMAT").unwrap_or_else(|_| "json".to_string()),
            },
        })
    }
}
