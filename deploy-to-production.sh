#!/bin/bash

# =================================================================
# === SolanaSniperV3 - PRODUCTION DEPLOYMENT SCRIPT v1.0 ===
# === Zoptymalizowany pod Contabo VDS (6 vCPU / 24 GB RAM) ===
# =================================================================

set -euo pipefail  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging function
log() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')] $1${NC}"
}

warn() {
    echo -e "${YELLOW}[$(date +'%Y-%m-%d %H:%M:%S')] WARNING: $1${NC}"
}

error() {
    echo -e "${RED}[$(date +'%Y-%m-%d %H:%M:%S')] ERROR: $1${NC}"
    exit 1
}

# Check if running as root
if [[ $EUID -eq 0 ]]; then
   error "Ten skrypt nie powinien być uruchamiany jako root!"
fi

log "🚀 Starting SolanaSniperV3 Production Deployment"

# --- Pre-deployment Checks ---
log "🔍 Performing pre-deployment checks..."

# Check if Docker is installed and running
if ! command -v docker &> /dev/null; then
    error "Docker nie jest zainstalowany!"
fi

if ! docker info &> /dev/null; then
    error "Docker nie jest uruchomiony!"
fi

# Check if Docker Compose is available
if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
    error "Docker Compose nie jest zainstalowany!"
fi

# Check available system resources
TOTAL_RAM=$(free -g | awk '/^Mem:/{print $2}')
TOTAL_CPU=$(nproc)

log "📊 System Resources:"
log "   CPU Cores: $TOTAL_CPU"
log "   Total RAM: ${TOTAL_RAM}GB"

if [[ $TOTAL_RAM -lt 20 ]]; then
    warn "Mało RAM! Zalecane minimum: 24GB, dostępne: ${TOTAL_RAM}GB"
fi

if [[ $TOTAL_CPU -lt 4 ]]; then
    warn "Mało CPU! Zalecane minimum: 6 cores, dostępne: $TOTAL_CPU"
fi

# Check if .env.production exists
if [[ ! -f ".env.production" ]]; then
    error "Plik .env.production nie istnieje! Skopiuj .env.production.example i skonfiguruj."
fi

# Check if critical environment variables are set
source .env.production

if [[ -z "${SOLANA_PRIVATE_KEY:-}" ]] || [[ "${SOLANA_PRIVATE_KEY}" == "YOUR_SOLANA_PRIVATE_KEY_HERE" ]]; then
    error "SOLANA_PRIVATE_KEY nie jest skonfigurowany w .env.production!"
fi

if [[ "${POSTGRES_PASSWORD}" == "sniper_secure_pass_2024_CHANGE_ME" ]]; then
    warn "Używasz domyślnego hasła PostgreSQL! Zmień je w .env.production"
fi

log "✅ Pre-deployment checks passed"

# --- Backup existing deployment ---
if docker-compose -f docker-compose.production.yml ps | grep -q "Up"; then
    log "📦 Creating backup of current deployment..."
    
    # Create backup directory
    BACKUP_DIR="backups/$(date +%Y%m%d_%H%M%S)"
    mkdir -p "$BACKUP_DIR"
    
    # Backup database
    log "💾 Backing up PostgreSQL database..."
    docker-compose -f docker-compose.production.yml exec -T postgres pg_dump -U "$POSTGRES_USER" "$POSTGRES_DB" > "$BACKUP_DIR/postgres_backup.sql" || warn "Database backup failed"
    
    # Backup DragonflyDB
    log "💾 Backing up DragonflyDB..."
    docker-compose -f docker-compose.production.yml exec -T dragonfly redis-cli BGSAVE || warn "DragonflyDB backup failed"
    
    log "✅ Backup completed: $BACKUP_DIR"
fi

# --- Stop existing services ---
log "🛑 Stopping existing services..."
docker-compose -f docker-compose.production.yml down --remove-orphans || true

# --- Clean up old images (optional) ---
read -p "🧹 Czy chcesz usunąć stare obrazy Docker? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    log "🧹 Cleaning up old Docker images..."
    docker system prune -f
    docker image prune -f
fi

# --- Build and deploy ---
log "🔨 Building and deploying services..."

# Build sniper-core with optimizations
log "🦀 Building sniper-core (Rust)..."
cd sniper-core
docker build -t sniper-core:production --target production .
cd ..

# Start services with proper order
log "🚀 Starting production services..."

# Start infrastructure first
docker-compose -f docker-compose.production.yml up -d postgres dragonfly

# Wait for infrastructure to be ready
log "⏳ Waiting for infrastructure services..."
sleep 30

# Check if infrastructure is healthy
if ! docker-compose -f docker-compose.production.yml exec postgres pg_isready -U "$POSTGRES_USER" -d "$POSTGRES_DB"; then
    error "PostgreSQL nie jest gotowy!"
fi

if ! docker-compose -f docker-compose.production.yml exec dragonfly redis-cli ping; then
    error "DragonflyDB nie jest gotowy!"
fi

# Start application services
log "🎯 Starting application services..."
docker-compose -f docker-compose.production.yml up -d sniper-core kestra

# Start monitoring (optional)
if [[ "${PROMETHEUS_ENABLED:-false}" == "true" ]]; then
    log "📊 Starting monitoring services..."
    docker-compose -f docker-compose.production.yml up -d prometheus
fi

# --- Health checks ---
log "🏥 Performing health checks..."

# Wait for services to start
sleep 60

# Check sniper-core health
if curl -f http://localhost:8003/health &> /dev/null; then
    log "✅ sniper-core is healthy"
else
    error "❌ sniper-core health check failed!"
fi

# Check Kestra health
if curl -f http://localhost:8080/health &> /dev/null; then
    log "✅ Kestra is healthy"
else
    warn "⚠️ Kestra health check failed - może potrzebować więcej czasu"
fi

# --- Display status ---
log "📊 Deployment Status:"
docker-compose -f docker-compose.production.yml ps

# --- Display resource usage ---
log "📈 Resource Usage:"
docker stats --no-stream --format "table {{.Container}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.MemPerc}}"

# --- Final instructions ---
log "🎉 Deployment completed successfully!"
log ""
log "📋 Available endpoints:"
log "   🌐 SniperCore API: http://localhost:8003"
log "   📊 Health Check:   http://localhost:8003/health"
log "   📈 Metrics:        http://localhost:8003/metrics"
log "   🔧 Kestra UI:      http://localhost:8080"
if [[ "${PROMETHEUS_ENABLED:-false}" == "true" ]]; then
    log "   📊 Prometheus:     http://localhost:9090"
fi
log ""
log "📝 Next steps:"
log "   1. Sprawdź logi: docker-compose -f docker-compose.production.yml logs -f"
log "   2. Monitoruj zasoby: docker stats"
log "   3. Skonfiguruj monitoring i alerty"
log "   4. Przetestuj wszystkie endpointy"
log ""
log "⚠️  WAŻNE: System działa w trybie PAPER TRADING"
log "   Aby przełączyć na LIVE trading, zmień TRADING_MODE w .env.production"

# --- Create monitoring script ---
cat > monitor-system.sh << 'EOF'
#!/bin/bash
echo "=== SolanaSniperV3 System Monitor ==="
echo "Timestamp: $(date)"
echo ""
echo "=== Container Status ==="
docker-compose -f docker-compose.production.yml ps
echo ""
echo "=== Resource Usage ==="
docker stats --no-stream --format "table {{.Container}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.MemPerc}}"
echo ""
echo "=== Health Checks ==="
echo -n "SniperCore: "
curl -s http://localhost:8003/health | jq -r '.status' 2>/dev/null || echo "FAILED"
echo -n "Kestra: "
curl -s http://localhost:8080/health | jq -r '.status' 2>/dev/null || echo "FAILED"
echo ""
echo "=== System Resources ==="
free -h
echo ""
df -h
EOF

chmod +x monitor-system.sh

log "📊 Created monitoring script: ./monitor-system.sh"
log "🎯 Deployment complete! System is ready for production use."
