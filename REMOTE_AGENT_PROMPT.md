# 🚀 FINALNY PROMPT DLA REMOTE AGENT AUGMENT
# Architektura Hybrydowa Rust + Python - SolanaSniperV3 dla Contabo VDS

**Bazując na wiedzy z dokumentu AUGMENT_MEMORY_V2.1_CONTABO_OPTIMIZED.md, wykonaj następujące zadanie:**

## 🖥️ **ŚRODOWISKO PRODUKCYJNE: Contabo VDS**
- **CPU:** 6 rdzeni (vCPU)
- **RAM:** 24 GB
- **Strategia:** Maksymalne wykorzystanie zasobów z inteligentną alokacją

## Cel: Stworzenie mikroserwisu `sniper-core` w Ruście

Zgodnie z filozofią architektoniczną z dokumentacji, tworzymy **Rdzeń Wykonawczy** - jeden z dwóch głównych serwisów systemu.

### KROK 1: Stwórz Strukturę Projektu
```bash
cargo new --bin sniper-core
cd sniper-core
```

### KROK 2: Skonfiguruj Zależności w Cargo.toml
Zmodyfikuj `sniper-core/Cargo.toml` zgodnie ze stosem technologicznym:

```toml
[package]
name = "sniper-core"
version = "0.1.0"
edition = "2021"

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

### KROK 3: Implementuj Serwer HTTP w src/main.rs
Stwórz serwer zgodny ze specyfikacją API z dokumentacji:
- **Port 8003** (zgodnie z AUGMENT_MEMORY_V1.md)
- **Endpointy:**
  - `GET /health` → `{"status": "ok"}`
  - `POST /api/v1/transaction/execute` → wykonanie transakcji
  - `GET /api/v1/market-data/price/{token}` → cena tokena

**Wymagania implementacji:**
- **Tokio multi-thread:** `#[tokio::main(worker_threads = 4)]` dla wykorzystania 4 z 6 rdzeni
- Strukturalne logowanie (JSON format)
- Obsługa zmiennych środowiskowych
- **Tryb "paper trading" domyślnie** (SNIPER_TRADING_MODE=paper)
- **Helius Smart Transactions** - użyj Helius SDK dla optymalizacji
- **HFT optimizations** - skip_preflight: true, max_retries: 0
- **Memory optimization** - jemalloc dla lepszego zarządzania pamięcią na Contabo
- Graceful error handling z proper HTTP status codes

**Przykład implementacji z Helius SDK:**
```rust
use helius::types::*;
use helius::Helius;

// Smart transaction config dla HFT
let config = SmartTransactionConfig {
    instructions,
    signers: vec![&keypair],
    send_options: RpcSendTransactionConfig {
        skip_preflight: true,
        max_retries: Some(0),
        preflight_commitment: Some(CommitmentLevel::Processed),
        ..Default::default()
    },
    lookup_tables: None,
};
```

### KROK 4: Stwórz Modularną Strukturę
Zorganizuj kod zgodnie ze specyfikacją:
```
sniper-core/
├── src/
│   ├── main.rs           # Entry point + server setup
│   ├── handlers/         # HTTP request handlers
│   │   ├── mod.rs
│   │   ├── health.rs     # GET /health
│   │   └── execution.rs  # POST /api/v1/transaction/execute
│   ├── solana/          # Solana blockchain integration
│   │   ├── mod.rs
│   │   └── client.rs    # Solana client wrapper
│   └── config/          # Configuration management
│       ├── mod.rs
│       └── settings.rs  # Environment variables handling
```

### KROK 5: Implementuj Zasady Bezpieczeństwa
**KRYTYCZNE - zgodnie z AUGMENT_MEMORY_V1.md:**
1. **ZAKAZ HARDKODOWANIA SEKRETÓW** - używaj zmiennych środowiskowych
2. **TRYB PAPER TRADING DOMYŚLNIE** - `SNIPER_TRADING_MODE=paper`
3. **WALIDACJA WEJŚCIA** - wszystkie dane z HTTP requests

### KROK 6: Stwórz Dockerfile
Multi-stage build dla produkcyjnego obrazu:
- Stage 1: Build (Rust compilation)
- Stage 2: Runtime (minimal image z binary)

### KROK 7: Dodaj Podstawowe Testy
- Unit testy dla handlers
- Integration test dla HTTP endpoints
- Test dla Solana client (mock mode)

### KROK 8: Dokumentacja
Stwórz README.md z:
- Instrukcjami uruchomienia
- Opisem API endpoints
- Przykładami użycia
- Zmiennymi środowiskowymi

## Oczekiwany Rezultat

Po wykonaniu zadania przedstaw:
1. **Pełną strukturę katalogów** projektu sniper-core
2. **Zawartość kluczowych plików** (main.rs, Cargo.toml, Dockerfile)
3. **Przykłady wywołań API** (curl commands)
4. **Instrukcje uruchomienia** lokalnego i w Docker
5. **Potwierdzenie zgodności** z zasadami bezpieczeństwa z AUGMENT_MEMORY_V1.md

## Kontekst Architektoniczny

Ten serwis będzie komunikował się z `strategy-host` (Python) przez REST API. Python będzie wysyłał requests do tego serwisu Rust w celu wykonania transakcji na Solana z maksymalną prędkością.

**Priorytet:** Prostota, wydajność, bezpieczeństwo - w tej kolejności.
