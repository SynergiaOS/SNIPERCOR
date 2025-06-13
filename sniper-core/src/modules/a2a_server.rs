// 🔗 A2A Server Module
// Agent-to-Agent communication protocol for future microservices

use axum::{
    extract::{Json, Path, Query},
    http::StatusCode,
    response::Json as ResponseJson,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use tracing::{info, error, debug};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

pub struct A2AServer {
    agent_registry: AgentRegistry,
    message_queue: MessageQueue,
}

impl A2AServer {
    pub async fn new() -> anyhow::Result<Self> {
        info!("🔗 A2AServer initializing...");

        Ok(Self {
            agent_registry: AgentRegistry::new(),
            message_queue: MessageQueue::new(),
        })
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        info!("🚀 A2AServer running (embedded in main server)");
        
        // A2A server runs as part of the main Axum server
        // Routes are registered in main.rs
        
        Ok(())
    }

    /// Get A2A routes for the main Axum server
    pub fn get_routes() -> Router {
        Router::new()
            .route("/a2a/agents", get(list_agents).post(register_agent))
            .route("/a2a/agents/:agent_id", get(get_agent).delete(unregister_agent))
            .route("/a2a/messages", post(send_message))
            .route("/a2a/messages/:agent_id", get(get_messages))
            .route("/a2a/health", get(a2a_health_check))
    }
}

/// Agent registration and discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: Uuid,
    pub name: String,
    pub agent_type: AgentType,
    pub capabilities: Vec<String>,
    pub endpoint: String,
    pub status: AgentStatus,
    pub registered_at: DateTime<Utc>,
    pub last_heartbeat: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentType {
    DataProvider,
    StrategyEngine,
    RiskManager,
    Executor,
    Monitor,
    External,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentStatus {
    Online,
    Offline,
    Busy,
    Error,
}

/// A2A Message protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2AMessage {
    pub id: Uuid,
    pub from_agent: Uuid,
    pub to_agent: Uuid,
    pub message_type: MessageType,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub priority: MessagePriority,
    pub requires_response: bool,
    pub correlation_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    MarketData,
    TradingSignal,
    ExecutionOrder,
    RiskAlert,
    SystemStatus,
    Heartbeat,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagePriority {
    Critical,
    High,
    Normal,
    Low,
}

/// Agent registry for service discovery
pub struct AgentRegistry {
    agents: HashMap<Uuid, AgentInfo>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
        }
    }

    pub fn register(&mut self, agent: AgentInfo) -> Uuid {
        let id = agent.id;
        self.agents.insert(id, agent);
        info!("🔗 Agent registered: {}", id);
        id
    }

    pub fn unregister(&mut self, agent_id: &Uuid) -> bool {
        if self.agents.remove(agent_id).is_some() {
            info!("🔗 Agent unregistered: {}", agent_id);
            true
        } else {
            false
        }
    }

    pub fn get(&self, agent_id: &Uuid) -> Option<&AgentInfo> {
        self.agents.get(agent_id)
    }

    pub fn list(&self) -> Vec<&AgentInfo> {
        self.agents.values().collect()
    }

    pub fn find_by_type(&self, agent_type: &AgentType) -> Vec<&AgentInfo> {
        self.agents
            .values()
            .filter(|agent| std::mem::discriminant(&agent.agent_type) == std::mem::discriminant(agent_type))
            .collect()
    }
}

/// Message queue for async communication
pub struct MessageQueue {
    messages: HashMap<Uuid, Vec<A2AMessage>>, // agent_id -> messages
}

impl MessageQueue {
    pub fn new() -> Self {
        Self {
            messages: HashMap::new(),
        }
    }

    pub fn send_message(&mut self, message: A2AMessage) {
        let to_agent = message.to_agent;
        self.messages.entry(to_agent).or_insert_with(Vec::new).push(message);
        debug!("📨 Message queued for agent: {}", to_agent);
    }

    pub fn get_messages(&mut self, agent_id: &Uuid) -> Vec<A2AMessage> {
        self.messages.remove(agent_id).unwrap_or_default()
    }
}

// ===== HTTP HANDLERS =====

/// List all registered agents
async fn list_agents() -> ResponseJson<Vec<AgentInfo>> {
    // TODO: Access global agent registry
    ResponseJson(vec![])
}

/// Register a new agent
#[derive(Deserialize)]
struct RegisterAgentRequest {
    name: String,
    agent_type: AgentType,
    capabilities: Vec<String>,
    endpoint: String,
}

async fn register_agent(
    Json(request): Json<RegisterAgentRequest>,
) -> Result<ResponseJson<AgentInfo>, StatusCode> {
    let agent = AgentInfo {
        id: Uuid::new_v4(),
        name: request.name,
        agent_type: request.agent_type,
        capabilities: request.capabilities,
        endpoint: request.endpoint,
        status: AgentStatus::Online,
        registered_at: Utc::now(),
        last_heartbeat: Utc::now(),
    };

    info!("🔗 Registering agent: {} ({})", agent.name, agent.id);

    // TODO: Add to global registry
    Ok(ResponseJson(agent))
}

/// Get agent information
async fn get_agent(Path(agent_id): Path<Uuid>) -> Result<ResponseJson<AgentInfo>, StatusCode> {
    // TODO: Get from global registry
    Err(StatusCode::NOT_FOUND)
}

/// Unregister an agent
async fn unregister_agent(Path(agent_id): Path<Uuid>) -> Result<StatusCode, StatusCode> {
    // TODO: Remove from global registry
    info!("🔗 Unregistering agent: {}", agent_id);
    Ok(StatusCode::OK)
}

/// Send message to another agent
async fn send_message(
    Json(message): Json<A2AMessage>,
) -> Result<ResponseJson<serde_json::Value>, StatusCode> {
    info!("📨 Sending A2A message: {} -> {}", message.from_agent, message.to_agent);

    // TODO: Add to global message queue
    
    Ok(ResponseJson(serde_json::json!({
        "status": "queued",
        "message_id": message.id
    })))
}

/// Get messages for an agent
#[derive(Deserialize)]
struct GetMessagesQuery {
    limit: Option<usize>,
    message_type: Option<String>,
}

async fn get_messages(
    Path(agent_id): Path<Uuid>,
    Query(query): Query<GetMessagesQuery>,
) -> ResponseJson<Vec<A2AMessage>> {
    debug!("📬 Getting messages for agent: {}", agent_id);
    
    // TODO: Get from global message queue
    ResponseJson(vec![])
}

/// A2A health check
async fn a2a_health_check() -> ResponseJson<serde_json::Value> {
    ResponseJson(serde_json::json!({
        "status": "healthy",
        "protocol_version": "1.0",
        "capabilities": [
            "agent_registration",
            "message_routing",
            "service_discovery"
        ],
        "timestamp": Utc::now().to_rfc3339()
    }))
}
