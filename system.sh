#!/bin/bash
set -e # Zatrzymuje skrypt w przypadku błędu

echo "🚀 --- Konfiguracja Środowiska dla SNIPERCOR --- 🚀"
echo "Sprawdzanie i instalowanie wymaganego oprogramowania..."

# Kolory dla output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Funkcja logowania
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

# Funkcja do sprawdzania i instalowania poleceń
install_if_missing() {
    local cmd=$1
    local install_cmd=$2
    local description=$3
    
    if ! command -v $cmd &> /dev/null; then
        log "📦 Instalowanie $description..."
        eval $install_cmd
        log "✅ $description zainstalowany."
    else
        log "✅ $description jest już zainstalowany."
    fi
}

# Sprawdzenie systemu operacyjnego
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    OS="linux"
    DISTRO=$(lsb_release -si 2>/dev/null || echo "Unknown")
elif [[ "$OSTYPE" == "darwin"* ]]; then
    OS="macos"
else
    error "Nieobsługiwany system operacyjny: $OSTYPE"
fi

log "🖥️ Wykryto system: $OS"

# --- Krok 1: Aktualizacja systemu ---
log "🔄 Aktualizacja systemu..."
if [[ "$OS" == "linux" ]]; then
    sudo apt-get update -qq
elif [[ "$OS" == "macos" ]]; then
    # macOS - sprawdź czy Homebrew jest zainstalowany
    if ! command -v brew &> /dev/null; then
        log "📦 Instalowanie Homebrew..."
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    fi
fi

# --- Krok 2: Instalacja Rusta ---
install_if_missing "rustc" \
    "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && source \$HOME/.cargo/env" \
    "Rust toolchain"

# Upewnij się, że cargo jest w PATH dla reszty skryptu
if ! command -v cargo &> /dev/null; then
    source "$HOME/.cargo/env"
fi

# Sprawdź wersję Rust
RUST_VERSION=$(rustc --version | cut -d' ' -f2)
log "🦀 Rust version: $RUST_VERSION"

# --- Krok 3: Instalacja Dockera i Docker Compose ---
if [[ "$OS" == "linux" ]]; then
    install_if_missing "docker" \
        "sudo apt-get install -y docker.io docker-compose-v2 && sudo systemctl start docker && sudo systemctl enable docker && sudo usermod -aG docker \$USER" \
        "Docker i Docker Compose"
elif [[ "$OS" == "macos" ]]; then
    install_if_missing "docker" \
        "brew install --cask docker" \
        "Docker Desktop"
fi

# Sprawdź wersję Docker
if command -v docker &> /dev/null; then
    DOCKER_VERSION=$(docker --version | cut -d' ' -f3 | tr -d ',')
    log "🐳 Docker version: $DOCKER_VERSION"
fi

# --- Krok 4: Instalacja dodatkowych narzędzi ---
log "🛠️ Instalowanie dodatkowych narzędzi..."

if [[ "$OS" == "linux" ]]; then
    # Linux tools
    sudo apt-get install -y curl jq htop git build-essential pkg-config libssl-dev
elif [[ "$OS" == "macos" ]]; then
    # macOS tools
    brew install curl jq htop git
fi

# --- Krok 5: Konfiguracja Firewalla (tylko Linux) ---
if [[ "$OS" == "linux" ]] && command -v ufw &> /dev/null; then
    log "🛡️ Konfigurowanie firewalla UFW..."
    if sudo ufw status | grep -q "inactive"; then
        sudo ufw default deny incoming
        sudo ufw default allow outgoing
        sudo ufw allow ssh
        sudo ufw allow 8003/tcp  # SniperCore API
        sudo ufw allow 8080/tcp  # Kestra UI
        sudo ufw allow 9090/tcp  # Prometheus (opcjonalnie)
        yes | sudo ufw enable
        log "✅ Firewall skonfigurowany i aktywny."
    else
        log "✅ Firewall jest już aktywny."
    fi
else
    warn "UFW nie znaleziony lub system nie-Linux. Pomięto konfigurację firewalla."
fi

# --- Krok 6: Konfiguracja Git (jeśli nie skonfigurowany) ---
if ! git config --global user.name &> /dev/null; then
    warn "Git nie jest skonfigurowany. Skonfiguruj go ręcznie:"
    echo "  git config --global user.name 'Your Name'"
    echo "  git config --global user.email 'your.email@example.com'"
fi

# --- Krok 7: Weryfikacja projektu (opcjonalna) ---
if [ -f "Cargo.toml" ]; then
    log "🦀 Weryfikacja projektu Rust..."
    if cargo check --quiet; then
        log "✅ Projekt Rust kompiluje się poprawnie."
    else
        warn "⚠️ Wystąpiły błędy kompilacji w projekcie Rust."
    fi
elif [ -f "sniper-core/Cargo.toml" ]; then
    log "🦀 Weryfikacja projektu Rust w sniper-core/..."
    cd sniper-core
    if cargo check --quiet; then
        log "✅ Projekt Rust kompiluje się poprawnie."
    else
        warn "⚠️ Wystąpiły błędy kompilacji w projekcie Rust."
    fi
    cd ..
fi

# --- Krok 8: Sprawdzenie zasobów systemowych ---
log "📊 Sprawdzanie zasobów systemowych..."

if [[ "$OS" == "linux" ]]; then
    TOTAL_RAM=$(free -g | awk '/^Mem:/{print $2}')
    TOTAL_CPU=$(nproc)
elif [[ "$OS" == "macos" ]]; then
    TOTAL_RAM=$(( $(sysctl -n hw.memsize) / 1024 / 1024 / 1024 ))
    TOTAL_CPU=$(sysctl -n hw.ncpu)
fi

log "💾 RAM: ${TOTAL_RAM}GB"
log "🖥️ CPU cores: $TOTAL_CPU"

if [[ $TOTAL_RAM -lt 8 ]]; then
    warn "Mało RAM! Zalecane minimum: 8GB dla development, 24GB dla produkcji"
fi

if [[ $TOTAL_CPU -lt 4 ]]; then
    warn "Mało CPU! Zalecane minimum: 4 cores dla development, 6 cores dla produkcji"
fi

# --- Krok 9: Tworzenie przykładowego .env (jeśli nie istnieje) ---
if [ ! -f ".env.production" ] && [ ! -f "sniper-core/.env" ]; then
    log "📝 Tworzenie przykładowego pliku .env..."
    cat > .env.example << 'EOF'
# === SNIPERCOR ENVIRONMENT CONFIGURATION ===

# Solana Configuration
QUICKNODE_RPC_URL=https://api.devnet.solana.com
HELIUS_WS_URL=wss://atlas-mainnet.helius-rpc.com
SOLANA_PRIVATE_KEY=YOUR_SOLANA_PRIVATE_KEY_HERE
SOLANA_COMMITMENT=confirmed

# Server Configuration
SERVER_HOST=0.0.0.0
SERVER_PORT=8003
TOKIO_WORKER_THREADS=5

# Trading Configuration
TRADING_MODE=paper
MAX_POSITION_SIZE=1000.0
MAX_DAILY_LOSS=500.0
MIN_CONFIDENCE=0.6

# Database Configuration
POSTGRES_USER=sniper_user
POSTGRES_PASSWORD=CHANGE_ME_IN_PRODUCTION
POSTGRES_DB=sniper_db
DATABASE_URL=postgresql://sniper_user:CHANGE_ME_IN_PRODUCTION@postgres:5432/sniper_db

# Cache Configuration
DRAGONFLY_URL=redis://dragonfly:6379
CACHE_DEFAULT_TTL=3600

# Logging
RUST_LOG=info
LOG_FORMAT=json
EOF
    log "✅ Utworzono .env.example - skopiuj i skonfiguruj jako .env.production"
fi

echo ""
log "🎉 === ŚRODOWISKO JEST GOTOWE! === 🎉"
log "Masz wszystko, co potrzebne do budowy i uruchomienia SNIPERCOR."
echo ""
log "📋 Następne kroki:"
log "1. Skopiuj .env.example do .env.production i skonfiguruj"
log "2. Uruchom: ./deploy-to-production.sh"
log "3. Monitoruj: ./monitor-system.sh"
echo ""

if [[ "$OS" == "linux" ]] && groups $USER | grep -q docker; then
    warn "⚠️ Dodano Cię do grupy docker. Może być wymagane przelogowanie."
fi
