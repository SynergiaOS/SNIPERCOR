use axum::{
    extract::Json,
    http::StatusCode,
    response::Json as ResponseJson,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    transaction::Transaction,
};
use spl_token::instruction as spl_instruction;
use spl_associated_token_account;
use std::str::FromStr;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Modules for future microservices integration
mod modules;
mod config;

// Request/Response structures
#[derive(Debug, Deserialize)]
struct TransactionRequest {
    private_key: String,
    recipient_address: String,
    token_mint: String,
    amount: u64,
}

#[derive(Debug, Serialize)]
struct TransactionResponse {
    status: String,
    signature: Option<String>,
    message: Option<String>,
}

// Konfiguracja Tokio dla Contabo VDS (5 z 6 rdzeni - Pure Rust Architecture)
#[tokio::main(worker_threads = 5)]
async fn main() -> anyhow::Result<()> {
    // Load configuration
    let config = config::Config::from_env()?;

    // Inicjalizacja structured logging w formacie JSON
    init_logging()?;

    info!("🚀 Starting SniperCore v0.1.0 - Ultra-HFT Pure Rust System");
    info!("📊 Tokio configured for {} worker threads", config.server.worker_threads);
    info!("🌐 Trading mode: {}", config.trading.mode);
    info!("🔗 Solana RPC: {}", config.solana.rpc_url);

    // A2A protocol ready for future microservices
    info!("🔧 A2A protocol endpoints ready");

    // Konfiguracja serwera HTTP (Axum) z A2A endpoints
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/transaction/execute", post(execute_transaction))
        .route("/metrics", get(get_metrics))
        .route("/status", get(get_system_status))
        .nest("/api/v1/a2a", modules::a2a_server::A2AServer::get_routes())
        .layer(TraceLayer::new_for_http());

    let bind_addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = TcpListener::bind(&bind_addr).await?;

    info!("🌐 Server listening on http://{}", bind_addr);
    info!("📋 Available endpoints:");
    info!("   GET  /health - Health check");
    info!("   POST /api/v1/transaction/execute - Execute SPL token transfer");
    info!("   GET  /metrics - System metrics");
    info!("   GET  /status - System status");
    info!("   *    /api/v1/a2a/* - A2A protocol endpoints");

    // Graceful shutdown handling
    let server_handle = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Wait for server or shutdown signal
    tokio::select! {
        _ = server_handle => info!("🌐 HTTP server stopped"),
        _ = tokio::signal::ctrl_c() => {
            info!("🛑 Received shutdown signal");
        }
    }

    info!("✅ SniperCore shutdown complete");
    Ok(())
}

// Health check endpoint
async fn health_check() -> ResponseJson<serde_json::Value> {
    ResponseJson(serde_json::json!({
        "status": "ok",
        "service": "sniper-core",
        "version": "0.1.0",
        "network": "devnet",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

// Execute transaction endpoint - Real Solana SPL Token Transfer
async fn execute_transaction(
    Json(request): Json<TransactionRequest>,
) -> Result<ResponseJson<TransactionResponse>, StatusCode> {
    info!(
        "🔄 Received SPL token transfer request: {} tokens to {}",
        request.amount, request.recipient_address
    );

    // Execute the transaction
    match execute_spl_transfer(request).await {
        Ok(signature) => {
            info!("✅ Transaction successful: {}", signature);
            Ok(ResponseJson(TransactionResponse {
                status: "success".to_string(),
                signature: Some(signature),
                message: None,
            }))
        }
        Err(e) => {
            error!("❌ Transaction failed: {}", e);
            Ok(ResponseJson(TransactionResponse {
                status: "error".to_string(),
                signature: None,
                message: Some(e.to_string()),
            }))
        }
    }
}

// Main SPL Token Transfer Logic
async fn execute_spl_transfer(request: TransactionRequest) -> anyhow::Result<String> {
    // 1. Parse private key from base58
    let private_key_bytes = bs58::decode(&request.private_key)
        .into_vec()
        .map_err(|e| anyhow::anyhow!("Invalid private key format: {}", e))?;

    let keypair = Keypair::from_bytes(&private_key_bytes)
        .map_err(|e| anyhow::anyhow!("Failed to create keypair: {}", e))?;

    info!("🔑 Keypair created from private key");

    // 2. Create RPC client connected to Solana Devnet
    let rpc_url = "https://api.devnet.solana.com";
    let client = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());

    info!("🌐 Connected to Solana Devnet: {}", rpc_url);

    // 3. Parse addresses
    let recipient_pubkey = Pubkey::from_str(&request.recipient_address)
        .map_err(|e| anyhow::anyhow!("Invalid recipient address: {}", e))?;

    let token_mint_pubkey = Pubkey::from_str(&request.token_mint)
        .map_err(|e| anyhow::anyhow!("Invalid token mint address: {}", e))?;

    // 4. Get associated token accounts
    let sender_ata = spl_associated_token_account::get_associated_token_address(
        &keypair.pubkey(),
        &token_mint_pubkey,
    );

    let recipient_ata = spl_associated_token_account::get_associated_token_address(
        &recipient_pubkey,
        &token_mint_pubkey,
    );

    info!("📍 Sender ATA: {}", sender_ata);
    info!("📍 Recipient ATA: {}", recipient_ata);

    // 5. Build SPL token transfer instruction
    let transfer_instruction = spl_instruction::transfer(
        &spl_token::id(),
        &sender_ata,
        &recipient_ata,
        &keypair.pubkey(),
        &[&keypair.pubkey()],
        request.amount,
    )?;

    info!("📝 Transfer instruction created for {} tokens", request.amount);

    // 6. Get recent blockhash
    let recent_blockhash = client
        .get_latest_blockhash()
        .map_err(|e| anyhow::anyhow!("Failed to get recent blockhash: {}", e))?;

    // 7. Create and sign transaction
    let transaction = Transaction::new_signed_with_payer(
        &[transfer_instruction],
        Some(&keypair.pubkey()),
        &[&keypair],
        recent_blockhash,
    );

    info!("✍️ Transaction signed");

    // 8. Send transaction and wait for confirmation
    let signature = client
        .send_and_confirm_transaction(&transaction)
        .map_err(|e| anyhow::anyhow!("Transaction failed: {}", e))?;

    info!("🚀 Transaction sent and confirmed: {}", signature);

    Ok(signature.to_string())
}

// System metrics endpoint
async fn get_metrics() -> ResponseJson<serde_json::Value> {
    // TODO: Get real metrics from HFT system
    ResponseJson(serde_json::json!({
        "total_signals": 0,
        "total_executions": 0,
        "success_rate": 0.0,
        "average_latency_ms": 0.0,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

// System status endpoint
async fn get_system_status() -> ResponseJson<serde_json::Value> {
    ResponseJson(serde_json::json!({
        "status": "running",
        "modules": {
            "data_ingestor": "online",
            "strategy_engine": "online",
            "risk_manager": "online",
            "executor": "online",
            "cache_manager": "online",
            "a2a_server": "online"
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

// Logging initialization - ustrukturyzowane logowanie w formacie JSON
fn init_logging() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "sniper_core=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    Ok(())
}
