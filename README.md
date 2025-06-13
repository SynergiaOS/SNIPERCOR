# 🎯 SolanaSniperV3 - High-Performance Trading System

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![Python](https://img.shields.io/badge/python-3.9+-blue.svg)](https://www.python.org)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Contabo](https://img.shields.io/badge/optimized-Contabo_VDS-red.svg)](https://contabo.com)

Zaawansowany system automatycznego tradingu na sieci Solana z architekturą hybrydową Rust + Python, zoptymalizowany pod serwer Contabo VDS (6 CPU / 24GB RAM).

## 🏗️ **Architektura**

### **Filozofia: Hybrid Rust + Python**
- **🦀 sniper-core (Rust):** Ultra-szybkie wykonywanie transakcji
- **🐍 strategy-host (Python):** Elastyczna logika biznesowa i AI
- **🔗 Komunikacja:** Proste, wydajne REST API

### **Stos Technologiczny**
- **Backend:** Rust (Axum, Tokio), Python (FastAPI)
- **Blockchain:** Solana SDK, Helius SDK (smart transactions)
- **Cache:** DragonflyDB (25x szybszy niż Redis)
- **Database:** PostgreSQL
- **Orkiestracja:** Kestra z Docker
- **Deployment:** Docker Compose

## 🖥️ **Optymalizacja Contabo VDS**

### **Specyfikacja Serwera**
- **CPU:** 6 rdzeni (vCPU)
- **RAM:** 24 GB
- **Strategia:** Inteligentna alokacja zasobów

### **Alokacja Zasobów**
```yaml
sniper-core (Rust):    4 CPU + 6GB RAM   # Krytyczne operacje
DragonflyDB:           6 CPU + 10GB RAM  # Ultra-szybki cache
Pozostałe serwisy:     2 CPU + 8GB RAM   # Python, PostgreSQL, Kestra
```

## 🚀 **Szybki Start**

### **1. Klonowanie Repozytorium**
```bash
git clone https://github.com/SynergiaOS/SNIPERCOR.git
cd SNIPERCOR
```

### **2. Konfiguracja Środowiska**
```bash
# Skopiuj przykładową konfigurację
cp sniper-core/.env.example sniper-core/.env

# Edytuj konfigurację (dodaj swoje klucze API)
nano sniper-core/.env
```

### **3. Uruchomienie z Docker Compose**
```bash
# Uruchomienie wszystkich serwisów
docker-compose up -d

# Sprawdzenie statusu
docker-compose ps

# Logi sniper-core
docker-compose logs -f sniper-core
```

### **4. Testowanie API**
```bash
# Health check
curl http://localhost:8003/health

# Test paper trading (domyślnie bezpieczny)
curl -X POST http://localhost:8003/api/v1/transaction/execute \
  -H "Content-Type: application/json" \
  -d '{"token_address":"So11111111111111111111111111111111111111112","amount":0.1}'
```

## 📁 **Struktura Projektu**

```
SniperCore/
├── 📚 Dokumentacja/
│   ├── AUGMENT_MEMORY_V2.1_CONTABO_OPTIMIZED.md  # Baza wiedzy
│   ├── REMOTE_AGENT_PROMPT.md                    # Prompt dla AI
│   ├── CONTABO_OPTIMIZATION_SUMMARY.md           # Optymalizacje
│   └── TECHNICAL_UPDATES_SUMMARY.md              # Aktualizacje tech
├── 🦀 sniper-core/                               # Rust microservice
│   ├── src/                                      # Kod źródłowy
│   ├── Cargo.toml                                # Zależności Rust
│   ├── .env.example                              # Konfiguracja
│   └── Dockerfile                                # Kontener
├── 🐍 strategy-host/                             # Python service
├── 🔧 scripts/                                   # Skrypty pomocnicze
├── 🐳 docker-compose.yml                         # Orkiestracja
└── 📋 README.md                                  # Ten plik
```
