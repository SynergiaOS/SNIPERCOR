use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{info, warn, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod handlers;
mod solana;

use config::AppConfig;

// Konfiguracja Tokio dla Contabo VDS (4 z 6 rdzeni)
#[tokio::main(worker_threads = 4)]
async fn main() -> anyhow::Result<()> {
    // Inicjalizacja structured logging
    init_logging()?;
    
    info!("🚀 Starting SniperCore v0.1.0 on Contabo VDS");
    info!("📊 Tokio configured for 4 worker threads");
    
    // Ładowanie konfiguracji
    let config = AppConfig::load()?;
    info!("⚙️  Configuration loaded: mode={}", config.trading_mode);
    
    // Sprawdzenie trybu paper trading
    if config.trading_mode == "paper" {
        warn!("📝 Running in PAPER TRADING mode - no real transactions");
    } else {
        info!("💰 Running in LIVE TRADING mode");
    }
    
    // Inicjalizacja shared state
    let app_state = Arc::new(AppState::new(config).await?);
    
    // Konfiguracja routingu
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/transaction/execute", post(execute_transaction))
        .route("/api/v1/market-data/price/:token", get(get_token_price))
        .route("/metrics", get(metrics_handler))
        .with_state(app_state)
        .layer(
            tower_http::trace::TraceLayer::new_for_http()
                .make_span_with(tower_http::trace::DefaultMakeSpan::default())
                .on_response(tower_http::trace::DefaultOnResponse::default()),
        );
    
    // Start serwera na porcie 8003
    let listener = TcpListener::bind("0.0.0.0:8003").await?;
    info!("🌐 Server listening on http://0.0.0.0:8003");
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

// Shared application state
#[derive(Clone)]
pub struct AppState {
    config: AppConfig,
    solana_client: Arc<solana::SolanaClient>,
    redis_client: Arc<redis::Client>,
}

impl AppState {
    async fn new(config: AppConfig) -> anyhow::Result<Self> {
        // Inicjalizacja Solana client z Helius
        let solana_client = Arc::new(
            solana::SolanaClient::new(&config.solana_rpc_url, &config.helius_api_key).await?
        );
        
        // Inicjalizacja Redis/DragonflyDB client
        let redis_client = Arc::new(
            redis::Client::open(config.redis_url.as_str())?
        );
        
        info!("✅ Application state initialized");
        
        Ok(Self {
            config,
            solana_client,
            redis_client,
        })
    }
}

// Health check endpoint
async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "sniper-core",
        "version": "0.1.0",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "environment": "contabo-vds"
    }))
}

// Execute transaction endpoint
async fn execute_transaction(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ExecuteTransactionRequest>,
) -> Result<Json<ExecuteTransactionResponse>, StatusCode> {
    let transaction_id = uuid::Uuid::new_v4();
    
    info!(
        transaction_id = %transaction_id,
        token = %payload.token_address,
        amount = %payload.amount,
        "🔄 Processing transaction request"
    );
    
    // Paper trading check
    if state.config.trading_mode == "paper" {
        warn!(
            transaction_id = %transaction_id,
            "📝 Paper trading mode - simulating transaction"
        );
        
        return Ok(Json(ExecuteTransactionResponse {
            transaction_id: transaction_id.to_string(),
            signature: format!("PAPER_TRADE_{}", transaction_id),
            status: "simulated".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }));
    }
    
    // Real transaction execution
    match state.solana_client.execute_smart_transaction(&payload).await {
        Ok(signature) => {
            info!(
                transaction_id = %transaction_id,
                signature = %signature,
                "✅ Transaction executed successfully"
            );
            
            Ok(Json(ExecuteTransactionResponse {
                transaction_id: transaction_id.to_string(),
                signature,
                status: "confirmed".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            }))
        }
        Err(e) => {
            error!(
                transaction_id = %transaction_id,
                error = %e,
                "❌ Transaction execution failed"
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Get token price endpoint
async fn get_token_price(
    State(state): State<Arc<AppState>>,
    Path(token): Path<String>,
) -> Result<Json<TokenPriceResponse>, StatusCode> {
    match state.solana_client.get_token_price(&token).await {
        Ok(price) => Ok(Json(TokenPriceResponse {
            token_address: token,
            price_usd: price,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

// Metrics endpoint dla monitoringu
async fn metrics_handler() -> String {
    // TODO: Implement Prometheus metrics
    "# HELP sniper_core_info Information about sniper-core\n".to_string()
}

// Request/Response types
#[derive(Debug, Deserialize)]
struct ExecuteTransactionRequest {
    token_address: String,
    amount: f64,
    slippage: Option<f64>,
    priority_fee: Option<u64>,
}

#[derive(Debug, Serialize)]
struct ExecuteTransactionResponse {
    transaction_id: String,
    signature: String,
    status: String,
    timestamp: String,
}

#[derive(Debug, Serialize)]
struct TokenPriceResponse {
    token_address: String,
    price_usd: f64,
    timestamp: String,
}

// Logging initialization
fn init_logging() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "sniper_core=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();
    
    Ok(())
}
