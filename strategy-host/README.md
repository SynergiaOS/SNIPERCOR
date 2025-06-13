# 🐍 Strategy-Host (Python Service)

## 📋 **Status: Placeholder**

Ten katalog jest przygotowany dla przyszłego serwisu `strategy-host` w Pythonie, który będzie odpowiedzialny za:

### **Planowane Funkcjonalności:**
- **🧠 AI Strategy Logic:** Integracja z DeepSeek dla analizy rynku
- **📊 Market Data Processing:** Przetwarzanie danych z Helius WebSockets
- **⚖️ Risk Management:** Zarządzanie ryzykiem i compliance
- **🔗 API Communication:** Komunikacja z `sniper-core` przez REST API

### **Architektura:**
```python
strategy-host/
├── src/
│   ├── main.py              # FastAPI server
│   ├── strategies/          # Trading strategies
│   ├── risk/               # Risk management
│   ├── market_data/        # Data processing
│   └── ai/                 # AI integration
├── requirements.txt        # Python dependencies
├── Dockerfile             # Container configuration
└── tests/                 # Unit tests
```

### **Technologie:**
- **Framework:** FastAPI
- **AI:** DeepSeek API integration
- **WebSockets:** Helius market data streams
- **Cache:** DragonflyDB client
- **Database:** PostgreSQL with SQLAlchemy

### **Komunikacja z sniper-core:**
```python
# Przykład wywołania transakcji
async def execute_trade(token: str, amount: float):
    async with httpx.AsyncClient() as client:
        response = await client.post(
            "http://sniper-core:8003/api/v1/transaction/execute",
            json={"token_address": token, "amount": amount}
        )
        return response.json()
```

## 🚀 **Następne Kroki**

1. **Implementacja FastAPI server** z podstawowymi endpointami
2. **Integracja z DragonflyDB** dla cache'owania danych
3. **WebSocket client** dla Helius market data
4. **AI strategy engine** z DeepSeek integration
5. **Risk management system** z limitami i alertami
6. **Comprehensive testing** z pytest

## 📝 **Notatki Implementacyjne**

### **Environment Variables:**
```bash
# Strategy Host Configuration
STRATEGY_HOST_PORT=8001
SNIPER_CORE_URL=http://sniper-core:8003

# AI Configuration
DEEPSEEK_API_KEY=your_deepseek_key
DEEPSEEK_MODEL=deepseek-chat

# Market Data
HELIUS_WS_URL=wss://atlas-mainnet.helius-rpc.com/
HELIUS_API_KEY=your_helius_key

# Cache & Database
REDIS_URL=redis://dragonfly:6379
DATABASE_URL=postgresql://sniper:password@postgresql:5432/sniper
```

### **Docker Resources (Contabo VDS):**
```yaml
strategy-host:
  deploy:
    resources:
      limits:
        cpus: '2.0'      # 2 z 6 dostępnych rdzeni
        memory: 4G       # ~17% całkowitej pamięci
```

---

**🔄 Ten serwis zostanie zaimplementowany w następnej fazie projektu, po ukończeniu `sniper-core`.**
