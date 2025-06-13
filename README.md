# 🎯 SolanaSniperV3 - Production HFT Trading System

[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Contabo](https://img.shields.io/badge/optimized-Contabo_VDS-red.svg)](https://contabo.com)

Production-ready High-Frequency Trading system built in Pure Rust, optimized for ultra-low latency trading on Solana blockchain.

## ⚡ Key Features

- **Ultra-low latency** - Microsecond-optimized execution
- **Pure Rust** - Maximum performance & memory safety
- **Production-ready** - Optimized for Contabo VDS (6 vCPU / 24GB RAM)
- **Real-time monitoring** - Comprehensive metrics & health checks
- **Secure by default** - Paper trading mode, externalized secrets

## 🏗️ Architecture

- **sniper-core** (Rust) - Ultra-fast execution engine
- **DragonflyDB** - High-performance cache
- **PostgreSQL** - Transaction history
- **Kestra** - Workflow orchestration
- **Prometheus** - Monitoring & metrics

## 🚀 Production Deployment

### Prerequisites

- Docker 24.0+ & Docker Compose 2.0+
- 24GB RAM minimum, 6 CPU cores
- Ubuntu 20.04+ / Debian 11+

### Quick Deploy

```bash
git clone https://github.com/SynergiaOS/SNIPERCOR.git
cd SNIPERCOR

# Configure environment
cp .env.production .env.production.local
nano .env.production.local  # Set your API keys

# Deploy to production
./deploy-to-production.sh
```

### Endpoints

- **API**: http://localhost:8003
- **Health**: http://localhost:8003/health
- **Metrics**: http://localhost:8003/metrics
- **Kestra**: http://localhost:8080

## 📊 Resource Allocation

| Service | CPU | RAM | Priority |
|---------|-----|-----|----------|
| sniper-core | 4 cores | 12GB | 🔴 Critical |
| dragonfly | Shared | 8GB | 🟡 High |
| postgres | 1 core | 2GB | 🟢 Low |
| kestra | 1 core | 2GB | 🟢 Low |

## 📈 Monitoring

```bash
# System status
./monitor-system.sh

# View logs
docker-compose -f docker-compose.production.yml logs -f sniper-core
```

## 🔒 Security

- **Paper trading** by default
- Externalized secrets via environment
- Non-root containers
- Network isolation

## 📚 Documentation

- [Production Deployment Guide](PRODUCTION-DEPLOYMENT.md)

## ⚠️ Important

System starts in **PAPER TRADING** mode. Change `TRADING_MODE=live` in `.env.production` only after thorough testing.

---

*SolanaSniperV3 v1.0 - Production Ready*
