# === PODRĘCZNIK WDROŻENIA PRODUKCYJNEGO - SNIPERCOR ===
# Wersja 1.0

## 1. CEL DOKUMENTU

Ten dokument opisuje krok po kroku proces wdrożenia systemu HFT `SNIPERCOR` na serwerze produkcyjnym Contabo VDS (6 vCPU, 24 GB RAM). Celem jest zapewnienie bezpiecznego, powtarzalnego i zautomatyzowanego procesu wdrożeniowego.

---

## 2. WYMAGANIA SYSTEMOWE

### Serwer Produkcyjny (Contabo VDS):
- **CPU**: 6 vCPU cores
- **RAM**: 24 GB
- **Storage**: 100 GB SSD NVMe
- **Network**: 1 Gbps
- **OS**: Ubuntu 22.04 LTS (zalecane)

### Wymagane Oprogramowanie:
- Docker 24.0+
- Docker Compose 2.0+
- Git 2.30+
- curl, jq, htop

---

## 3. FAZA 1: PRZYGOTOWANIE SERWERA (JEDNORAZOWO)

**Cel:** Skonfigurowanie czystego serwera Ubuntu pod wymagania naszego stacku.

### Krok 1: Połączenie z Serwerem
```bash
ssh root@TWOJ_ADRES_IP_SERWERA
```

### Krok 2: Automatyczne Przygotowanie Środowiska
```bash
# Pobierz i uruchom skrypt konfiguracyjny
curl -o /root/system.sh https://raw.githubusercontent.com/SynergiaOS/SNIPERCOR/main/system.sh

# Nadaj uprawnienia
chmod +x /root/system.sh

# Uruchom automatyczną konfigurację
./system.sh
```

### Krok 3: Weryfikacja Instalacji
```bash
# Sprawdź wersje zainstalowanego oprogramowania
docker --version
docker compose version
cargo --version
git --version
```

**Kryterium Sukcesu:** Wszystkie komendy działają bez błędów i bez wymagania `sudo` dla Docker.

---

## 4. FAZA 2: WDROŻENIE APLIKACJI

**Cel:** Sklonowanie repozytorium, konfiguracja sekretów i uruchomienie stacku.

### Krok 1: Przygotowanie Katalogu Aplikacji
```bash
# Przejdź do zalecanej lokalizacji
cd /opt

# Sklonuj repozytorium
git clone https://github.com/SynergiaOS/SNIPERCOR.git
cd SNIPERCOR

# Ustaw właściciela (jeśli potrzebne)
chown -R $USER:$USER /opt/SNIPERCOR
```

### Krok 2: Konfiguracja Sekretów (KRYTYCZNY KROK!)
```bash
# Skopiuj szablon konfiguracji
cp .env.production .env.production.local

# Edytuj konfigurację z prawdziwymi sekretami
nano .env.production.local
```

### ⚠️ KRYTYCZNE ZMIENNE DO SKONFIGUROWANIA:
```bash
# Solana Configuration - WYMAGANE
SOLANA_PRIVATE_KEY=your_actual_base58_private_key_here
QUICKNODE_RPC_URL=your_quicknode_premium_endpoint
HELIUS_WS_URL=your_helius_websocket_endpoint
HELIUS_API_KEY=your_helius_api_key

# Security - ZMIEŃ WSZYSTKIE HASŁA
POSTGRES_PASSWORD=ultra_secure_password_2024_CHANGE_ME
JWT_SECRET=your_256_bit_jwt_secret_here
API_KEY=your_secure_api_key_here

# Trading Configuration - BEZPIECZNY START
TRADING_MODE=paper  # ZAWSZE zacznij od 'paper'!
MAX_POSITION_SIZE=1000.0
MAX_DAILY_LOSS=500.0
MIN_CONFIDENCE=0.7
```

### Krok 3: Uruchomienie Stacku Produkcyjnego
```bash
# Nadaj uprawnienia skryptowi wdrożenia
chmod +x deploy-to-production.sh

# Uruchom automatyczne wdrożenie
./deploy-to-production.sh
```

**Alternatywnie - Manualny Deployment:**
```bash
# Build i uruchomienie wszystkich serwisów
docker-compose -f docker-compose.production.yml --env-file .env.production.local up -d --build

# Sprawdzenie statusu wszystkich kontenerów
docker-compose -f docker-compose.production.yml ps
```

**Kryterium Sukcesu:** Wszystkie kontenery (sniper-core, dragonfly, postgres, kestra) są w stanie `running` i przechodzą health checks.

---

## 5. FAZA 3: WERYFIKACJA I MONITORING

**Cel:** Potwierdzenie poprawnego działania systemu i uruchomienie monitoringu.

### Krok 1: Sprawdzenie Logów
```bash
# Monitoruj logi sniper-core w czasie rzeczywistym
docker logs -f sniper-core-prod

# Sprawdź logi wszystkich serwisów
docker-compose -f docker-compose.production.yml logs --tail=50
```

**Oczekiwane logi sniper-core:**
```
🚀 Starting SniperCore v0.1.0 - Ultra-HFT Pure Rust System
📊 Tokio configured for 5 worker threads
🌐 Trading mode: paper
✅ All HFT modules initialized successfully
🌐 Server listening on http://0.0.0.0:8003
```

### Krok 2: Weryfikacja Endpointów
```bash
# Health check sniper-core
curl -s http://localhost:8003/health | jq .

# System metrics
curl -s http://localhost:8003/metrics | jq .

# System status
curl -s http://localhost:8003/status | jq .
```

**Oczekiwane odpowiedzi:**
- Health: `{"status": "healthy", "timestamp": "..."}`
- Metrics: JSON z metrykami systemu
- Status: Status wszystkich modułów

### Krok 3: Dostęp do Kestra UI
```bash
# Sprawdź czy Kestra jest dostępna
curl -s http://localhost:8080/health

# Otwórz w przeglądarce (jeśli port jest otwarty)
# http://TWOJ_ADRES_IP_SERWERA:8080
```

### Krok 4: Monitoring Zasobów
```bash
# Uruchom skrypt monitoringu
./monitor-system.sh

# Sprawdź wykorzystanie zasobów
docker stats --no-stream
```

**Kryterium Sukcesu:** Wszystkie endpointy odpowiadają, logi są czyste, zasoby są w normie.

## 📊 Alokacja Zasobów

### Strategia Alokacji (6 vCPU / 24 GB RAM):

| Serwis | CPU Cores | RAM | Priorytet |
|--------|-----------|-----|-----------|
| **sniper-core** | 4.0 (limit) / 3.5 (reserved) | 12GB (limit) / 8GB (reserved) | 🔴 KRYTYCZNY |
| **dragonfly** | Shared (6 threads) | 8GB (reserved) | 🟡 WYSOKI |
| **postgres** | 1.0 (limit) / 0.5 (reserved) | 2GB (limit) / 1GB (reserved) | 🟢 NISKI |
| **kestra** | 1.0 (limit) / 0.5 (reserved) | 2GB (limit) / 1GB (reserved) | 🟢 NISKI |

## 🏥 Health Checks & Monitoring

### Sprawdzenie Statusu
```bash
# Status wszystkich serwisów
docker-compose -f docker-compose.production.yml ps

# Health check endpointów
curl http://localhost:8003/health
curl http://localhost:8080/health

# Monitoring zasobów
./monitor-system.sh
```

### Dostępne Endpointy
- **SniperCore API**: http://localhost:8003
- **Health Check**: http://localhost:8003/health
- **Metrics**: http://localhost:8003/metrics
- **System Status**: http://localhost:8003/status
- **Kestra UI**: http://localhost:8080
- **Prometheus**: http://localhost:9090 (jeśli włączony)

## 📈 Monitoring & Logs

### Viewing Logs
```bash
# Wszystkie logi
docker-compose -f docker-compose.production.yml logs -f

# Logi konkretnego serwisu
docker-compose -f docker-compose.production.yml logs -f sniper-core

# Logi z ostatniej godziny
docker-compose -f docker-compose.production.yml logs --since 1h sniper-core
```

### Resource Monitoring
```bash
# Real-time resource usage
docker stats

# System resources
htop
free -h
df -h
```

## 🔒 Bezpieczeństwo

### Firewall Configuration
```bash
# Podstawowa konfiguracja UFW
sudo ufw default deny incoming
sudo ufw default allow outgoing

# Zezwolenie na SSH
sudo ufw allow ssh

# Zezwolenie na porty aplikacji (tylko jeśli potrzebne z zewnątrz)
sudo ufw allow 8003  # SniperCore API
sudo ufw allow 8080  # Kestra UI

# Włączenie firewall
sudo ufw enable
```

### SSL/TLS (Opcjonalne)
```bash
# Instalacja Certbot dla Let's Encrypt
sudo apt install certbot

# Generowanie certyfikatu
sudo certbot certonly --standalone -d your-domain.com
```

## 🔄 Backup & Recovery

### Automatyczny Backup
```bash
# Backup jest tworzony automatycznie podczas deploymentu
# Lokalizacja: ./backups/YYYYMMDD_HHMMSS/
```

### Manualny Backup
```bash
# Backup PostgreSQL
docker-compose -f docker-compose.production.yml exec postgres pg_dump -U sniper_user sniper_db > backup_$(date +%Y%m%d).sql

# Backup DragonflyDB
docker-compose -f docker-compose.production.yml exec dragonfly redis-cli BGSAVE
```

### Recovery
```bash
# Restore PostgreSQL
docker-compose -f docker-compose.production.yml exec -T postgres psql -U sniper_user sniper_db < backup_20241213.sql

# Restart services
docker-compose -f docker-compose.production.yml restart
```

## 🚨 Troubleshooting

### Częste Problemy

#### 1. Brak pamięci
```bash
# Sprawdzenie użycia pamięci
free -h
docker stats

# Restart serwisów z wysokim użyciem
docker-compose -f docker-compose.production.yml restart dragonfly
```

#### 2. Problemy z siecią
```bash
# Sprawdzenie portów
netstat -tulpn | grep :8003

# Restart sieci Docker
docker network prune
```

#### 3. Błędy kompilacji
```bash
# Czyszczenie cache Docker
docker system prune -a

# Rebuild bez cache
docker-compose -f docker-compose.production.yml build --no-cache sniper-core
```

## 📞 Support & Maintenance

### Regularne Zadania
- **Codziennie**: Sprawdzenie logów i metryk
- **Tygodniowo**: Backup danych, aktualizacja systemu
- **Miesięcznie**: Przegląd bezpieczeństwa, optymalizacja

### Kontakt
- **GitHub Issues**: https://github.com/SynergiaOS/SNIPERCOR/issues
- **Documentation**: https://github.com/SynergiaOS/SNIPERCOR/wiki

---

## ⚠️ WAŻNE OSTRZEŻENIA

1. **NIGDY nie commituj pliku .env.production do repozytorium!**
2. **Zacznij ZAWSZE w trybie 'paper' przed przejściem na 'live'**
3. **Regularnie twórz backupy przed aktualizacjami**
4. **Monitoruj zasoby systemowe - HFT wymaga stabilności**
5. **Testuj wszystkie endpointy po każdym deploymencie**

---

---

## 6. PROCEDURA AKTUALIZACJI SYSTEMU

**Cel:** Bezpieczne wdrażanie nowych wersji kodu.

### Krok 1: Przygotowanie Aktualizacji
```bash
# Połącz się z serwerem
ssh user@TWOJ_ADRES_IP_SERWERA
cd /opt/SNIPERCOR

# Utwórz backup przed aktualizacją (automatycznie w deploy-to-production.sh)
```

### Krok 2: Pobranie Nowych Zmian
```bash
# Pobierz najnowsze zmiany
git fetch origin
git pull origin main

# Sprawdź co się zmieniło
git log --oneline -10
```

### Krok 3: Wdrożenie Aktualizacji
```bash
# Uruchom skrypt aktualizacji (z automatycznym backup)
./deploy-to-production.sh

# Monitoruj proces aktualizacji
docker-compose -f docker-compose.production.yml logs -f
```

### Krok 4: Weryfikacja Po Aktualizacji
```bash
# Sprawdź health checks
curl -s http://localhost:8003/health
curl -s http://localhost:8080/health

# Monitoruj logi przez 5 minut
docker logs -f sniper-core-prod
```

---

## 7. TROUBLESHOOTING

### Problem: Kontenery nie startują
```bash
# Sprawdź logi błędów
docker-compose -f docker-compose.production.yml logs

# Sprawdź zasoby systemowe
free -h && df -h

# Restart całego stacku
docker-compose -f docker-compose.production.yml down
docker-compose -f docker-compose.production.yml up -d
```

### Problem: Brak połączenia z Solana
```bash
# Sprawdź konfigurację RPC
grep QUICKNODE_RPC_URL .env.production

# Test połączenia
curl -X POST -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}' \
  $QUICKNODE_RPC_URL
```

### Problem: Wysokie użycie zasobów
```bash
# Sprawdź wykorzystanie przez kontenery
docker stats

# Restart problematycznego kontenera
docker-compose -f docker-compose.production.yml restart sniper-core
```

---

## ⚠️ WAŻNE OSTRZEŻENIA

1. **ZAWSZE zaczynaj w trybie PAPER TRADING**
2. **NIGDY nie commituj plików .env do repozytorium**
3. **ZAWSZE twórz backup przed aktualizacją**
4. **MONITORUJ system przez pierwsze 24h po wdrożeniu**
5. **TESTUJ wszystkie endpointy po każdej aktualizacji**

---

**WERSJA:** 1.0
**DATA:** 2024-12-13
**STATUS:** Production Ready
**KONTAKT:** GitHub Issues - https://github.com/SynergiaOS/SNIPERCOR/issues
