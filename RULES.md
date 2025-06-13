# === ZASADY I ARCHITEKTURA PROJEKTU SNIPERCOR ===
# Wersja 3.0 - Monolit HFT w Ruście

## 1. GŁÓWNA FILOZOFIA ARCHITEKTONICZNA

**Cel:** Stworzenie jednego, ultra-wydajnego, monolitycznego (ale wewnętrznie modułowego) systemu HFT w Ruście.
**Priorytet:** Maksymalna wydajność i minimalizacja opóźnień poprzez eliminację komunikacji sieciowej między komponentami.
**Komunikacja Wewnętrzna:** Moduły komunikują się ze sobą **wyłącznie** za pomocą asynchronicznych kanałów `tokio::sync::mpsc`.

---

## 2. STOS TECHNOLOGICZNY (TECHNOLOGY STACK)

- **Rdzeń Aplikacji:** Rust (stable toolchain).
- **Runtime Asynchroniczny:** Tokio (skonfigurowany na 5 worker threads dla Contabo VDS).
- **Interakcja z Solaną:**
  - **Dostawca RPC:** **QuickNode Premium Endpoint**.
  - **Dostawca WebSockets:** **Helius**.
  - **Biblioteki:** `solana-client`, `solana-sdk`, `spl-token`.
  - **Protokół Sieciowy:** Klient RPC MUSI być skonfigurowany do używania transportu **QUIC**.
- **Baza Danych (Audyt):** **PostgreSQL** (dostęp przez `sqlx`).
- **Cache (Ultra-szybki):** **DragonflyDB** (25x szybszy niż Redis).
- **Orkiestracja i Deployment:** **Kestra** i **Docker Compose**.

---

## 3. STRUKTURA PROJEKTU `SNIPERCOR`

Projekt jest jednym repozytorium z następującą, modułową strukturą w `src/`:
- `main.rs`: Inicjalizuje i łączy wszystkie moduły.
- `config/mod.rs`: Zarządza konfiguracją z plików i zmiennych środowiskowych.
- `modules/`: Katalog zawierający całą logikę biznesową:
  - `data_ingestor.rs`: Pobieranie danych (Helius/QuickNode).
  - `strategy_engine.rs`: Logika decyzyjna (w tym integracja z AI).
  - `risk_manager.rs`: Zarządzanie ryzykiem i pozycjami.
  - `executor.rs`: Wykonywanie transakcji na DEX.
  - `cache_manager.rs`: Integracja z DragonflyDB.
  - `a2a_server.rs`: Protokół A2A (przygotowany na przyszłość).

---

## 4. ZASADY KRYTYCZNE (DO BEZWZGLĘDNEGO PRZESTRZEGANIA)

1. **BEZPIECZEŃSTWO SEKRETÓW:** Wszystkie klucze API i klucze prywatne MUSZĄ być ładowane ze zmiennych środowiskowych. NIGDY nie umieszczaj ich w kodzie.
2. **TRYB DOMYŚLNY "PAPER TRADING":** System domyślnie działa w trybie symulacji. Przełączenie na `live` wymaga jawnej zmiany w konfiguracji.
3. **NIEZAWODNOŚĆ:** Każda operacja I/O (sieć, baza danych) musi mieć solidną obsługę błędów i logikę ponawiania prób (retry).
4. **TESTOWANIE:** Każdy moduł i funkcja publiczna musi mieć testy jednostkowe. Cały system musi być testowany end-to-end.

---

## 5. ALOKACJA ZASOBÓW (CONTABO VDS 6 vCPU / 24GB RAM)

### Strategia Alokacji:
- **sniper-core (Rust):** 4 CPU cores + 12GB RAM (najwyższy priorytet)
- **DragonflyDB:** Shared CPU + 8GB RAM (wysoki priorytet pamięci)
- **PostgreSQL + Kestra:** 2 CPU cores + 4GB RAM (niski priorytet)

### Docker Compose Limits:
```yaml
sniper-core:
  deploy:
    resources:
      limits: { cpus: '4.0', memory: 12G }
      reservations: { cpus: '3.5', memory: 8G }

dragonfly:
  deploy:
    resources:
      limits: { memory: 9G }
      reservations: { memory: 8G }
```

---

## 6. KOMUNIKACJA MIĘDZY MODUŁAMI

### Kanały Tokio (Wewnętrzne):
```rust
// Przykład struktury kanałów
pub struct Channels {
    pub market_data_tx: mpsc::Sender<MarketDataMessage>,
    pub signal_tx: mpsc::Sender<SignalMessage>,
    pub execution_tx: mpsc::Sender<ExecutionMessage>,
    pub cache_tx: mpsc::Sender<CacheMessage>,
}
```

### Przepływ Danych:
1. **DataIngestor** → `market_data_tx` → **StrategyEngine**
2. **StrategyEngine** → `signal_tx` → **RiskManager**
3. **RiskManager** → `execution_tx` → **Executor**
4. **Wszystkie moduły** → `cache_tx` → **CacheManager**

---

## 7. INTEGRACJA AI

### Strategia AI:
- **Zintegrowane w StrategyEngine** (nie zewnętrzne)
- **Lokalne modele** lub **API calls** (DeepSeek, OpenAI)
- **PyO3 bindings** dla Python ML libraries (opcjonalnie)

### Zastosowania AI:
- Analiza sentymentu tokenów
- Wykrywanie wzorców cenowych
- Ocena ryzyka "rug pull"
- Predykcja volatility

---

## 8. PROTOKÓŁ A2A (PRZYSZŁOŚĆ)

### Obecny Status:
- **Zaimplementowany** ale **nieaktywny**
- **Gotowy na przyszłą skalowalność**
- **Endpoints dostępne** w `/api/v1/a2a/*`

### Kiedy Używać:
- Gdy system rozrośnie się do mikroservices
- Gdy potrzebujemy zewnętrznych agentów
- Gdy wymagana jest komunikacja między instancjami

---

## 9. ŚRODOWISKA

### Development:
- **Solana Devnet**
- **Paper trading mode**
- **Local Docker Compose**

### Production:
- **Solana Mainnet**
- **Live trading** (po testach)
- **Contabo VDS deployment**
- **Monitoring & alerting**

---

## 10. DEPLOYMENT

### Pliki Kluczowe:
- `docker-compose.production.yml` - Konfiguracja produkcyjna
- `.env.production` - Zmienne środowiskowe
- `deploy-to-production.sh` - Skrypt wdrożenia

### Proces Wdrożenia:
1. Konfiguracja `.env.production`
2. Uruchomienie `./deploy-to-production.sh`
3. Monitoring przez `./monitor-system.sh`
4. Health checks na `/health`

---

## 11. MONITORING & METRYKI

### Endpoints:
- `GET /health` - Health check
- `GET /metrics` - Prometheus metrics
- `GET /status` - System status

### Kluczowe Metryki:
- Latency wykonania transakcji
- Success rate
- Pozycje i PnL
- Wykorzystanie zasobów

---

## 12. AUGMENT AGENT INTEGRATION

### Rola Augment Agent:
- **Development tool** (nie runtime)
- **Code generation & testing**
- **Docker image building**
- **CI/CD automation**

### Nie Jest Częścią Produkcji:
- Augment Agent nie działa na serwerze produkcyjnym
- Używany tylko do development i deployment
- Wszystka logika biznesowa w Rust monolicie

---

**WERSJA:** 3.0 - Finalna  
**DATA:** 2024-12-13  
**STATUS:** Production Ready
