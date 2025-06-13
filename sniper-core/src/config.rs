use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    // Trading configuration
    pub trading_mode: String,  // "paper" or "live"
    
    // Solana configuration
    pub solana_rpc_url: String,
    pub helius_api_key: String,
    pub wallet_private_key: String,
    
    // Cache configuration (DragonflyDB)
    pub redis_url: String,
    
    // Database configuration
    pub database_url: String,
    
    // Server configuration
    pub server_host: String,
    pub server_port: u16,
    
    // Performance tuning dla Contabo VDS
    pub max_connections: u32,
    pub request_timeout_ms: u64,
    pub retry_attempts: u32,
    
    // Risk management
    pub max_position_size: f64,
    pub max_daily_loss: f64,
    pub max_slippage: f64,
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        // Load .env file if exists
        dotenvy::dotenv().ok();
        
        let config = Self {
            // Trading mode - MUST default to paper for safety
            trading_mode: env::var("SNIPER_TRADING_MODE")
                .unwrap_or_else(|_| "paper".to_string()),
            
            // Solana configuration
            solana_rpc_url: env::var("SNIPER_SOLANA_RPC_URL")
                .map_err(|_| anyhow::anyhow!("SNIPER_SOLANA_RPC_URL must be set"))?,
            
            helius_api_key: env::var("SNIPER_HELIUS_API_KEY")
                .map_err(|_| anyhow::anyhow!("SNIPER_HELIUS_API_KEY must be set"))?,
            
            wallet_private_key: env::var("SNIPER_WALLET_PRIVATE_KEY")
                .map_err(|_| anyhow::anyhow!("SNIPER_WALLET_PRIVATE_KEY must be set"))?,
            
            // Cache configuration
            redis_url: env::var("SNIPER_REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            
            // Database configuration
            database_url: env::var("SNIPER_DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://sniper:password@localhost/sniper".to_string()),
            
            // Server configuration
            server_host: env::var("SNIPER_SERVER_HOST")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            
            server_port: env::var("SNIPER_SERVER_PORT")
                .unwrap_or_else(|_| "8003".to_string())
                .parse()
                .unwrap_or(8003),
            
            // Performance tuning dla Contabo VDS (6 CPU / 24GB RAM)
            max_connections: env::var("SNIPER_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .unwrap_or(1000),
            
            request_timeout_ms: env::var("SNIPER_REQUEST_TIMEOUT_MS")
                .unwrap_or_else(|_| "5000".to_string())
                .parse()
                .unwrap_or(5000),
            
            retry_attempts: env::var("SNIPER_RETRY_ATTEMPTS")
                .unwrap_or_else(|_| "3".to_string())
                .parse()
                .unwrap_or(3),
            
            // Risk management defaults
            max_position_size: env::var("SNIPER_MAX_POSITION_SIZE")
                .unwrap_or_else(|_| "1000.0".to_string())
                .parse()
                .unwrap_or(1000.0),
            
            max_daily_loss: env::var("SNIPER_MAX_DAILY_LOSS")
                .unwrap_or_else(|_| "500.0".to_string())
                .parse()
                .unwrap_or(500.0),
            
            max_slippage: env::var("SNIPER_MAX_SLIPPAGE")
                .unwrap_or_else(|_| "0.05".to_string())
                .parse()
                .unwrap_or(0.05),
        };
        
        // Validation
        config.validate()?;
        
        Ok(config)
    }
    
    fn validate(&self) -> anyhow::Result<()> {
        // Validate trading mode
        if !["paper", "live"].contains(&self.trading_mode.as_str()) {
            return Err(anyhow::anyhow!(
                "Invalid trading mode: {}. Must be 'paper' or 'live'", 
                self.trading_mode
            ));
        }
        
        // Validate URLs
        if !self.solana_rpc_url.starts_with("http") {
            return Err(anyhow::anyhow!("Invalid Solana RPC URL"));
        }
        
        if !self.redis_url.starts_with("redis://") {
            return Err(anyhow::anyhow!("Invalid Redis URL"));
        }
        
        // Validate risk parameters
        if self.max_slippage < 0.0 || self.max_slippage > 1.0 {
            return Err(anyhow::anyhow!("Max slippage must be between 0.0 and 1.0"));
        }
        
        if self.max_position_size <= 0.0 {
            return Err(anyhow::anyhow!("Max position size must be positive"));
        }
        
        if self.max_daily_loss <= 0.0 {
            return Err(anyhow::anyhow!("Max daily loss must be positive"));
        }
        
        Ok(())
    }
    
    /// Returns true if running in paper trading mode
    pub fn is_paper_trading(&self) -> bool {
        self.trading_mode == "paper"
    }
    
    /// Returns true if running in live trading mode
    pub fn is_live_trading(&self) -> bool {
        self.trading_mode == "live"
    }
    
    /// Get Contabo-optimized connection pool size
    pub fn get_connection_pool_size(&self) -> u32 {
        // Dla Contabo VDS z 6 CPU, ograniczamy pool connections
        std::cmp::min(self.max_connections, 500)
    }
    
    /// Get timeout for HTTP requests
    pub fn get_request_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.request_timeout_ms)
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            trading_mode: "paper".to_string(),
            solana_rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
            helius_api_key: "".to_string(),
            wallet_private_key: "".to_string(),
            redis_url: "redis://localhost:6379".to_string(),
            database_url: "postgresql://sniper:password@localhost/sniper".to_string(),
            server_host: "0.0.0.0".to_string(),
            server_port: 8003,
            max_connections: 1000,
            request_timeout_ms: 5000,
            retry_attempts: 3,
            max_position_size: 1000.0,
            max_daily_loss: 500.0,
            max_slippage: 0.05,
        }
    }
}
