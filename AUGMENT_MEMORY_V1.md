# AUGMENT_MEMORY_V2.1_CONTABO_OPTIMIZED - Projekt SolanaSniperV3
# Wersja 2.1 - Architektura Hybrydowa z Optymalizacją pod Contabo VDS

## PRZEGLĄD PROJEKTU

**Nazwa:** SolanaSniperV3 Enhanced
**Cel:** Wydajny system automatycznego tradingu na sieci Solana
**Architektura:** Hybrydowa - 2 główne serwisy komunikujące się przez REST API

## ZASADY BEZPIECZEŃSTWA (KATEGORIA 1: KRYTYCZNE - NIGDY NIE ŁAM)

### 1. ZAKAZ HARDKODOWANIA SEKRETÓW
- Wszystkie klucze API, klucze prywatne, hasła MUSZĄ być pobierane z menedżera sekretów Kestry: `{{ secret('KEY') }}`
- Alternatywnie: zmienne środowiskowe z prefiksem `SNIPER_`
- NIGDY nie umieszczaj sekretów bezpośrednio w kodzie

### 2. WALIDACJA WEJŚCIA
- Każda dana z zewnętrznego źródła MUSI być zwalidowana
- Implementuj sanityzację danych przed użyciem
- Zapobiegaj atakom: SQL Injection, manipulacja zleceniem

### 3. TRYB "PAPER TRADING" DOMYŚLNIE
- Wszystkie funkcje transakcyjne MUSZĄ domyślnie działać w trybie symulacji
- Przełączenie na tryb "live" wymaga jawnej, manualnej zmiany konfiguracji
- Zmienna środowiskowa: `SNIPER_TRADING_MODE=paper|live`

## GŁÓWNA FILOZOFIA ARCHITEKTONICZNA

System jest zbudowany w oparciu o architekturę **dwóch głównych serwisów**:

1. **Rdzeń Wykonawczy (`sniper-core`) w Ruście:** Odpowiedzialny za wszystkie operacje o niskim opóźnieniu i bezpośrednią interakcję z blockchainem Solana.

2. **Host Strategii (`strategy-host`) w Pythonie:** Odpowiedzialny za logikę biznesową, analizę AI, scraping i podejmowanie decyzji.

**Komunikacja:** `strategy-host` (Python) komunikuje się z `sniper-core` (Rust) poprzez **proste, wydajne REST API**.

## ŚRODOWISKO PRODUKCYJNE - CONTABO VDS

### Specyfikacja Serwera
- **CPU:** 6 rdzeni (vCPU)
- **RAM:** 24 GB
- **Cel:** Maksymalne i inteligentne wykorzystanie dostępnych zasobów

### Strategia Alokacji Zasobów

#### 1. `sniper-core` (Rust): WYSOKI PRIORYTET CPU
- **Rdzenie:** 4 z 6 dostępnych rdzeni (tokio multi-thread)
- **RAM:** 4-6 GB (ograniczone, aby nie zdominować systemu)
- **Konfiguracja:**
```rust
#[tokio::main(worker_threads = 4)]
async fn main() {
    // Wykorzystanie 4 rdzeni dla maksymalnej wydajności
}
```

#### 2. DragonflyDB: WYSOKI PRIORYTET CPU & RAM
- **Rdzenie:** Współdzielenie wszystkich 6 rdzeni (multi-threaded)
- **RAM:** 8-10 GB (znaczna część dla cache)
- **Konfiguracja:**
```yaml
dragonfly:
  command: ["--maxmemory=10gb", "--threads=6"]
```

#### 3. Pozostałe Serwisy: NISKI PRIORYTET
- **Rdzenie:** 2 pozostałe rdzenie
- **RAM:** 8-12 GB (Python, Kestra, PostgreSQL)

## STOS TECHNOLOGICZNY (TECHNOLOGY STACK)

- **Rdzeń Wykonawczy:** Rust, Axum, Tokio, Solana-SDK, **Helius SDK** (smart transactions)
- **Host Strategii:** Python, FastAPI, Scrapy, biblioteki do AI, **PyO3** (Rust extensions)
- **Orkiestracja:** **Kestra** z `runner: DOCKER` (rust:latest image)
- **Cache:** **DragonflyDB** (25x szybszy niż Redis, pełna kompatybilność)
- **Baza Danych:** **PostgreSQL**
- **Deployment:** **Docker Compose** na serwerze VDS

### Kluczowe Technologie

#### Helius SDK (Rust)
- **Smart Transactions:** Automatyczna optymalizacja compute units i priority fees
- **Retry Logic:** 60s timeout z inteligentnym polling
- **Regional Optimization:** Frankfurt/Pittsburgh co-location dla HFT

#### DragonflyDB
- **Wydajność:** 25x szybszy niż Redis
- **Kompatybilność:** Pełna zgodność z Redis API
- **Architektura:** Multi-threaded, shared-nothing

#### PyO3 Bridge
- **Rust Extensions:** Tworzenie Python modules w Rust
- **Maturin:** Narzędzie do budowania i publikacji
- **Zero-copy:** Wydajne przekazywanie danych między Python ↔ Rust

## SPECYFIKACJA KOMPONENTÓW

### 1. `sniper-core` (Serwis w Ruście)

- **Odpowiedzialność:** Wykonywanie poleceń z maksymalną prędkością
- **Struktura:** Jeden, samodzielny serwer HTTP
- **Port:** 8003
- **Kluczowe API (`/api/v1`):**
  - `POST /transaction/execute` - Przyjmuje dane zlecenia i wysyła transakcję na Solanie
  - `GET /market-data/price/{token}` - Pobiera aktualną cenę tokena przez Jupiter/Raydium API
  - `GET /health` - Zwraca status `{"status": "ok"}`

### 2. `strategy-host` (Aplikacja w Pythonie)

- **Odpowiedzialność:** Cała logika biznesowa
- **Struktura:** Jeden, główny proces Pythona, który zawiera logikę dla:
  - **Market Data Logic:** Używa WebSockets (Helius) do nasłuchiwania na nowe pule i publikuje zdarzenia do **DragonflyDB**
  - **Strategy Logic:** Nasłuchuje na zdarzenia z Dragonfly, wykonuje zapytania do `sniper-core` o cenę, pyta AI (DeepSeek) o analizę
  - **Risk Logic:** Przed wysłaniem polecenia wykonania, sprawdza limity ryzyka
  - **Execution Logic:** Wykonuje zapytanie `POST` do endpointu `/transaction/execute` w serwisie `sniper-core`

## IMPLEMENTACJE I BEST PRACTICES

### Helius Smart Transactions (Rust)

```rust
use helius::types::*;
use helius::Helius;

// Konfiguracja smart transaction
let config = SmartTransactionConfig {
    instructions,
    signers: vec![&keypair],
    send_options: RpcSendTransactionConfig {
        skip_preflight: true,  // Dla HFT
        max_retries: Some(0),  // Własna logika retry
        ..Default::default()
    },
    lookup_tables: None,
};

// Automatyczna optymalizacja i wysłanie
match helius.send_smart_transaction(config).await {
    Ok(signature) => println!("Transaction: {}", signature),
    Err(e) => eprintln!("Failed: {:?}", e),
}
```

### Kestra Workflow dla Rust

```yaml
id: rust_sniper_core
namespace: solana.sniper
tasks:
  - id: build_and_test
    type: io.kestra.plugin.scripts.shell.Commands
    taskRunner:
      type: io.kestra.plugin.scripts.runner.docker.Docker
      containerImage: rust:latest
    commands:
      - cargo build --release
      - cargo test
      - ./target/release/sniper-core
    outputFiles:
      - "target/release/sniper-core"
```

### DragonflyDB Configuration

```yaml
# docker-compose.yml
dragonfly:
  image: docker.dragonflydb.io/dragonflydb/dragonfly
  ports:
    - "6379:6379"
  command: ["--maxmemory=2gb", "--proactor_threads=4"]
  volumes:
    - dragonfly_data:/data
```

## KONFIGURACJA ŚRODOWISKA

### Zmienne Środowiskowe
```bash
# Tryb pracy
SNIPER_TRADING_MODE=paper  # paper|live

# Konfiguracja Solana
SNIPER_SOLANA_RPC_URL={{ secret('SOLANA_RPC_URL') }}
SNIPER_WALLET_PRIVATE_KEY={{ secret('WALLET_PRIVATE_KEY') }}

# Konfiguracja A2A
SNIPER_A2A_TOKEN={{ secret('A2A_TOKEN') }}
SNIPER_A2A_ENCRYPTION_KEY={{ secret('A2A_ENCRYPTION_KEY') }}

# Limity ryzyka
SNIPER_MAX_POSITION_SIZE=1000
SNIPER_MAX_DAILY_LOSS=500
```

## SPECYFIKACJA SNIPER-CORE (RUST)

### Wymagania Techniczne
- **Język:** Rust (wydajność i bezpieczeństwo)
- **Framework:** Axum (async HTTP server)
- **Solana SDK:** solana-sdk, solana-client
- **Konteneryzacja:** Docker

### Struktura Projektu
```
sniper-core/
├── Cargo.toml
├── Dockerfile
├── src/
│   ├── main.rs
│   ├── handlers/
│   │   ├── mod.rs
│   │   ├── health.rs
│   │   └── execution.rs
│   ├── solana/
│   │   ├── mod.rs
│   │   ├── client.rs
│   │   └── transaction.rs
│   └── config/
│       ├── mod.rs
│       └── settings.rs
```

### Kluczowe Zależności (Cargo.toml)
```toml
[dependencies]
# HTTP Server (Axum)
axum = "0.7"
tokio = { version = "1.0", features = ["full"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Solana Integration
solana-sdk = "2.0"
solana-client = "2.0"
helius = "0.1.5"  # Helius SDK dla smart transactions

# Error Handling & Utilities
anyhow = "1.0"
thiserror = "1.0"

# Logging (structured JSON)
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json"] }

# Configuration
config = "0.14"

# PyO3 dla Python extensions (opcjonalne)
pyo3 = { version = "0.25.1", features = ["extension-module"], optional = true }

[features]
python-extension = ["pyo3"]
```

## ZASADY ROZWOJU I JAKOŚCI KODU

### 1. Testowanie
- Wszystkie nowe funkcje MUSZĄ mieć testy jednostkowe
- Testy integracyjne dla komunikacji Python ↔ Rust
- Cały przepływ musi być testowany E2E w środowisku lokalnym przed wdrożeniem

### 2. Logowanie
- Używaj ustrukturyzowanego logowania z ID transakcji/zadania
- Poziomy: ERROR, WARN, INFO, DEBUG
- Format: JSON z timestamp, service_name, transaction_id

### 3. Obsługa Błędów
- Każda operacja I/O (sieć, baza danych) MUSI być opakowana w bloki try/except
- Odpowiednia logika obsługi błędów i logowanie
- Graceful degradation - system nie może się całkowicie zatrzymać z powodu jednego błędu

## HIGH-FREQUENCY TRADING OPTIMIZATIONS

### Lokalizacja Serwera
- **Preferowane regiony:** Frankfurt lub Pittsburgh (co-location z Helius)
- **Unikaj:** LATAM, South Africa (wysokie opóźnienia)
- **Eastern US / Western Europe** - optymalne dla sieci walidatorów Solana

### Optymalizacje Sieciowe
- **Warming caches:** Wysyłaj `getHealth` co sekundę do tego samego endpointu
- **Commitment level:** Używaj "processed" lub "confirmed" dla `getLatestBlockhash`
- **Skip preflight:** `skip_preflight: true` dla transakcji HFT
- **Max retries:** Ustaw na 0, implementuj własną logikę retry

### Optymalizacje Transakcji
```rust
// Helius best practices dla HFT
let send_options = RpcSendTransactionConfig {
    skip_preflight: true,
    max_retries: Some(0),
    preflight_commitment: Some(CommitmentLevel::Processed),
    ..Default::default()
};
```

### Monitoring i Metryki
- **Latencja:** Monitoruj czas od sygnału do potwierdzenia transakcji
- **Success rate:** Śledź % udanych transakcji
- **Compute units:** Optymalizuj zużycie CU dla niższych kosztów

## DEPLOYMENT

### Docker Compose
Wszystkie serwisy uruchamiane przez docker-compose.yml z:
- Siecią izolowaną między serwisami
- Persistent volumes dla PostgreSQL i DragonflyDB
- Environment variables z secrets
- Health checks dla wszystkich kontenerów
- Restart policies dla niezawodności

### Struktura Deployment dla Contabo VDS
```yaml
# docker-compose.yml - Zoptymalizowane pod 6 CPU / 24GB RAM
version: '3.8'
services:
  sniper-core:
    build: ./sniper-core
    ports:
      - "8003:8003"
    environment:
      - TOKIO_WORKER_THREADS=4
    deploy:
      resources:
        limits:
          cpus: '4.0'
          memory: 6G
        reservations:
          cpus: '3.0'
          memory: 4G

  dragonfly:
    image: docker.dragonflydb.io/dragonflydb/dragonfly
    ports:
      - "6379:6379"
    command: ["--maxmemory=10gb", "--threads=6"]
    deploy:
      resources:
        limits:
          memory: 12G
        reservations:
          memory: 8G

  strategy-host:
    build: ./strategy-host
    ports:
      - "8001:8001"
    deploy:
      resources:
        limits:
          cpus: '2.0'
          memory: 4G

  postgresql:
    image: postgres:15
    environment:
      POSTGRES_DB: sniper
      POSTGRES_USER: sniper
      POSTGRES_PASSWORD: ${DB_PASSWORD}
    deploy:
      resources:
        limits:
          memory: 2G

  kestra:
    image: kestra/kestra:latest
    deploy:
      resources:
        limits:
          memory: 2G
```
