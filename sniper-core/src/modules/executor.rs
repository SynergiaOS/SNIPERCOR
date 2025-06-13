// ⚡ Executor Module
// Ultra-fast Solana transaction execution

use tokio::sync::mpsc;
use tracing::{info, error, debug};
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
use std::time::Instant;

pub struct Executor {
    // TODO: Add channels when ready
    // execution_rx: mpsc::Receiver<ExecutionMessage>,
    solana_client: RpcClient,
    keypair: Keypair,
    execution_stats: ExecutionStats,
}

#[derive(Debug, Clone)]
pub enum OrderType {
    Market,
    Limit,
    StopMarket,
    StopLimit,
}

#[derive(Debug, Clone)]
pub enum ExecutionPriority {
    Critical,
    High,
    Normal,
    Low,
}

impl Executor {
    pub async fn new() -> anyhow::Result<Self> {
        info!("⚡ Executor initializing...");

        // Initialize Solana client with optimized settings
        let rpc_url = std::env::var("QUICKNODE_RPC_URL")
            .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());
        
        let client = RpcClient::new_with_commitment(
            rpc_url.clone(),
            CommitmentConfig::confirmed(),
        );

        // Load private key
        let private_key = std::env::var("SOLANA_PRIVATE_KEY")
            .map_err(|_| anyhow::anyhow!("SOLANA_PRIVATE_KEY environment variable not set"))?;
        
        let private_key_bytes = bs58::decode(&private_key)
            .into_vec()
            .map_err(|e| anyhow::anyhow!("Invalid private key format: {}", e))?;
        
        let keypair = Keypair::from_bytes(&private_key_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to create keypair: {}", e))?;

        info!("✅ Executor initialized");
        info!("🔗 RPC URL: {}", rpc_url);
        info!("🔑 Wallet: {}", keypair.pubkey());

        Ok(Self {
            solana_client: client,
            keypair,
            execution_stats: ExecutionStats::new(),
        })
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        info!("🚀 Starting Executor...");

        // TODO: Listen for execution messages
        // TODO: Process orders with ultra-low latency
        // TODO: Handle different order types

        // Placeholder implementation
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;
            debug!("⚡ Executor heartbeat - ready for execution");
        }
    }

    /// Execute market order with ultra-low latency
    async fn execute_market_order(&mut self, symbol: &str, quantity: f64) -> anyhow::Result<String> {
        let start_time = Instant::now();
        
        info!("⚡ Executing market order: {} {}", quantity, symbol);

        // TODO: Implement DEX integration
        // - Jupiter aggregator for best price
        // - Direct DEX interaction for speed
        // - Slippage protection

        let signature = self.execute_spl_transfer(symbol, quantity as u64, &self.keypair.pubkey()).await?;
        
        let execution_time = start_time.elapsed();
        self.execution_stats.record_success(execution_time);
        
        info!("✅ Market order executed: {} ({}ms)", signature, execution_time.as_millis());
        
        Ok(signature)
    }

    /// Execute limit order
    async fn execute_limit_order(&mut self, symbol: &str, quantity: f64, price: f64) -> anyhow::Result<String> {
        info!("📊 Executing limit order: {} {} @ {}", quantity, symbol, price);

        // TODO: Implement limit order logic
        // - Place order on DEX
        // - Monitor for fill
        // - Handle partial fills
        
        // For now, simulate with market order
        self.execute_market_order(symbol, quantity).await
    }

    /// Execute SPL token transfer (for demo/testing)
    async fn execute_spl_transfer(
        &self,
        token_symbol: &str,
        amount: u64,
        recipient: &Pubkey,
    ) -> anyhow::Result<String> {
        // For demo, use a known SPL token mint (USDC on devnet)
        let token_mint = match token_symbol {
            "USDC" => "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU", // USDC devnet
            _ => "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU", // Default to USDC
        };

        let token_mint_pubkey = Pubkey::from_str(token_mint)?;

        // Get associated token accounts
        let sender_ata = spl_associated_token_account::get_associated_token_address(
            &self.keypair.pubkey(),
            &token_mint_pubkey,
        );
        
        let recipient_ata = spl_associated_token_account::get_associated_token_address(
            recipient,
            &token_mint_pubkey,
        );

        // Build transfer instruction
        let transfer_instruction = spl_instruction::transfer(
            &spl_token::id(),
            &sender_ata,
            &recipient_ata,
            &self.keypair.pubkey(),
            &[&self.keypair.pubkey()],
            amount,
        )?;

        // Get recent blockhash for ultra-low latency
        let recent_blockhash = self.solana_client.get_latest_blockhash()?;

        // Create and sign transaction
        let transaction = Transaction::new_signed_with_payer(
            &[transfer_instruction],
            Some(&self.keypair.pubkey()),
            &[&self.keypair],
            recent_blockhash,
        );

        // Send transaction with high priority
        let signature = self.solana_client.send_and_confirm_transaction(&transaction)?;

        Ok(signature.to_string())
    }

    /// Get execution statistics
    pub fn get_stats(&self) -> &ExecutionStats {
        &self.execution_stats
    }

    /// Health check for executor
    pub async fn health_check(&self) -> anyhow::Result<bool> {
        // TODO: Check Solana connection
        // TODO: Verify wallet balance
        // TODO: Test transaction capability

        Ok(true)
    }
}

#[derive(Debug, Default)]
pub struct ExecutionStats {
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub average_latency_ms: f64,
    pub min_latency_ms: u64,
    pub max_latency_ms: u64,
}

impl ExecutionStats {
    pub fn new() -> Self {
        Self {
            min_latency_ms: u64::MAX,
            ..Default::default()
        }
    }

    pub fn record_success(&mut self, latency: std::time::Duration) {
        self.total_executions += 1;
        self.successful_executions += 1;
        self.update_latency_stats(latency);
    }

    pub fn record_failure(&mut self, latency: std::time::Duration) {
        self.total_executions += 1;
        self.failed_executions += 1;
        self.update_latency_stats(latency);
    }

    fn update_latency_stats(&mut self, latency: std::time::Duration) {
        let latency_ms = latency.as_millis() as u64;
        
        self.min_latency_ms = self.min_latency_ms.min(latency_ms);
        self.max_latency_ms = self.max_latency_ms.max(latency_ms);
        
        // Update rolling average
        let total_latency = self.average_latency_ms * (self.total_executions - 1) as f64;
        self.average_latency_ms = (total_latency + latency_ms as f64) / self.total_executions as f64;
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_executions == 0 {
            0.0
        } else {
            self.successful_executions as f64 / self.total_executions as f64 * 100.0
        }
    }
}
