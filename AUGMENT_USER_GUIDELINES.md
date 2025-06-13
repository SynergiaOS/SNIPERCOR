# === PODRĘCZNIK OPERATORA AUGMENTA (USER GUIDELINES) ===
# Projekt: SNIPERCOR - System HFT w Ruście

## 1. ZŁOTA ZASADA: "Kapitan i Pilot-Asystent"

**Cel:** Zrozumienie ról w interakcji Człowiek-AI.
- **TY (Operator/Kapitan):** Jesteś odpowiedzialny za strategię, architekturę, weryfikację i podejmowanie ostatecznych decyzji. Znasz kontekst biznesowy.
- **AUGMENT (Pilot-Asystent):** Jest odpowiedzialny za wykonywanie precyzyjnych, technicznych zadań: generowanie kodu, refaktoryzację, pisanie testów i analizę. Jest ekspertem technicznym, ale nie zna Twoich intencji.

**Zasada:** Nigdy ślepo nie akceptuj propozycji Augmenta, zwłaszcza w krytycznych modułach (`executor`, `risk_manager`). Zawsze weryfikuj kod i zadawaj pytania. **TY dowodzisz.**

---

## 2. STRUKTURA EFEKTYWNEGO PROMPTU

**Cel:** Formułowanie poleceń, które dają najlepsze i najbezpieczniejsze rezultaty. Każdy prompt musi zawierać trzy elementy:

### **A. KONTEKST (Context Injection):**
Zawsze zaczynaj prompt od odniesienia do naszej bazy wiedzy.
```
Bazując na wiedzy z RULES.md i istniejącym kodzie, wykonaj następujące zadanie:
```

### **B. ZADANIE (Precise Task Definition):**
Bądź tak precyzyjny, jak to tylko możliwe. Zamiast "napraw błąd", napisz:
```
W pliku src/modules/executor.rs, w funkcji send_transaction, dodaj obsługę błędu 
RpcError::TransactionError i zaimplementuj logikę ponawiania próby 3 razy 
z opóźnieniem wykładniczym.
```

**Zasady precyzyjnych zadań:**
- **Używaj numerowanych list** do rozbicia złożonych zadań na mniejsze kroki
- **Podawaj konkretne nazwy** plików, funkcji i zmiennych
- **Określaj oczekiwane zachowanie** i przypadki brzegowe
- **Wskazuj zależności** między modułami

### **C. KRYTERIA AKCEPTACJI (Acceptance Criteria):**
Zawsze kończ prompt jasnym poleceniem weryfikacji:
```
Po wprowadzeniu zmian, uruchom 'cargo test --workspace', aby potwierdzić, 
że wszystkie testy przechodzą. Przedstaw mi finalny diff do zatwierdzenia.
```

---

## 3. WORKFLOW PRACY Z AUGMENTEM

**Cel:** Ustrukturyzowany, powtarzalny proces deweloperski.

### **Etap 1: Implementacja Nowej Funkcjonalności**
1. **Sformułuj zadanie:** Napisz precyzyjny prompt zgodnie ze strukturą z punktu 2
2. **Zleć zadanie Augmentowi:** Wklej prompt do interfejsu
3. **Weryfikuj iteracyjnie:** Augment może przedstawiać wyniki etapami. Prowadź z nim dialog, poprawiając i doprecyzowując jego pracę
4. **Zatwierdź finalne zmiany:** Gdy kod jest zgodny z Twoimi oczekiwaniami, zaakceptuj zmiany

### **Etap 2: Generowanie Testów**
Po każdej nowej implementacji, zleć Augmentowi zadanie napisania testów:
```
Doskonale. Teraz napisz testy jednostkowe dla nowej funkcji 'calculate_slippage' 
w module 'strategy_engine'. Upewnij się, że pokrywają one przypadki brzegowe: 
zerowa płynność, ogromne zlecenie i normalne warunki.
```

### **Etap 3: Refaktoryzacja i Optymalizacja**
Gdy system już działa, możesz użyć Augmenta do jego ulepszania:
```
Przeanalizuj funkcję 'process_market_data'. Jest zbyt długa. Zrefaktoryzuj ją, 
wydzielając logikę parsowania do osobnej, prywatnej funkcji. Pamiętaj o zasadach 
czystego kodu.
```

---

## 4. ZAAWANSOWANE TECHNIKI

### **Praca na Branchach:**
Zanim zlecisz dużą, ryzykowną zmianę, stwórz nowy branch w Gicie:
```bash
git checkout -b feature/new-strategy
```
Możesz poinstruować Augmenta, aby pracował i commitował na tym konkretnym branchu.

### **Użycie Remote Agent:**
Dla zadań długotrwałych (kompilacja Rusta, uruchamianie pełnego suite'u testów E2E) lub wymagających czystego środowiska, zawsze używaj `Remote Agent`.

### **Iteracyjne Ulepszanie:**
```
1. Zaimplementuj podstawową funkcjonalność
2. Napisz testy
3. Zoptymalizuj wydajność
4. Dodaj error handling
5. Udokumentuj API
```

---

## 5. ZASADY BEZPIECZEŃSTWA PRACY Z AI

### **Ochrona Sekretów:**
- **Nigdy nie wklejaj sekretów do promptu:** Augment ma je pobierać z pliku `RULES.md` lub zmiennych środowiskowych
- **Używaj placeholder'ów:** `YOUR_API_KEY_HERE` zamiast prawdziwych kluczy
- **Weryfikuj .env pliki:** Upewnij się, że nie zawierają prawdziwych sekretów

### **Krytyczne Myślenie:**
- **Traktuj AI jako narzędzie, nie jako prawdę ostateczną:** Jeśli sugestia Augmenta wydaje Ci się dziwna lub nieintuicyjna, poproś o wyjaśnienie lub zignoruj ją
- **Twoja intuicja jako architekta jest najważniejsza**
- **Zawsze weryfikuj kod w modułach krytycznych** (`executor`, `risk_manager`)

---

## 6. PRZYKŁADY DOBRYCH PROMPTÓW

### **Implementacja Nowej Funkcji:**
```
Bazując na RULES.md, zaimplementuj w src/modules/data_ingestor.rs funkcję 
connect_helius_websocket(), która:

1. Łączy się z Helius WebSocket używając URL z HELIUS_WS_URL
2. Subskrybuje się na aktualizacje cen tokenów
3. Parsuje otrzymane dane do struktury MarketDataMessage
4. Wysyła dane przez market_data_tx channel
5. Obsługuje reconnection w przypadku utraty połączenia

Dodaj odpowiednie error handling i logging. Po implementacji uruchom cargo check.
```

### **Optymalizacja Wydajności:**
```
Przeanalizuj funkcję execute_market_order() w src/modules/executor.rs pod kątem 
optymalizacji latency. Zidentyfikuj bottlenecki i zaproponuj ulepszenia, 
zachowując bezpieczeństwo transakcji. Zmierz czas wykonania przed i po zmianach.
```

### **Pisanie Testów:**
```
Napisz kompletne testy jednostkowe dla RiskManager w src/modules/risk_manager.rs:

1. Test sprawdzania limitów pozycji
2. Test kalkulacji portfolio risk
3. Test emergency stop functionality
4. Mock'uj zewnętrzne zależności
5. Pokryj przypadki brzegowe

Uruchom cargo test i upewnij się, że coverage wynosi >90%.
```

---

## 7. CZERWONE FLAGI - KIEDY ZATRZYMAĆ AUGMENTA

### **Natychmiast Przerwij, Gdy:**
- Augment proponuje zmiany w kluczach prywatnych lub sekretach
- Sugeruje wyłączenie zabezpieczeń "dla szybkości"
- Modyfikuje krytyczne funkcje bez wyjaśnienia
- Proponuje użycie niesprawdzonych bibliotek w production

### **Bądź Ostrożny, Gdy:**
- Augment wprowadza nowe zależności
- Zmienia architekturę komunikacji między modułami
- Modyfikuje logikę risk managementu
- Proponuje "eksperymentalne" rozwiązania

---

## 8. CHECKLIST PRZED AKCEPTACJĄ KODU

### **Zawsze Sprawdź:**
- [ ] Kod kompiluje się bez warnings (`cargo check`)
- [ ] Wszystkie testy przechodzą (`cargo test`)
- [ ] Nie ma hardcoded sekretów
- [ ] Error handling jest kompletny
- [ ] Logging jest odpowiedni
- [ ] Dokumentacja jest aktualna
- [ ] Performance nie uległ pogorszeniu

### **Dla Krytycznych Modułów (executor, risk_manager):**
- [ ] Logika biznesowa jest poprawna
- [ ] Zabezpieczenia są zachowane
- [ ] Nie ma race conditions
- [ ] Memory safety jest zachowane
- [ ] Transakcje są atomowe

---

## 9. KOMUNIKACJA Z AUGMENTEM

### **Dobre Praktyki:**
- **Bądź konkretny:** "Dodaj logging" → "Dodaj debug logging dla każdej transakcji z timestamp i amount"
- **Zadawaj pytania:** "Dlaczego wybrałeś tę implementację?"
- **Proś o alternatywy:** "Pokaż mi 2 różne sposoby rozwiązania tego problemu"
- **Weryfikuj zrozumienie:** "Podsumuj, co zaimplementowałeś"

### **Unikaj:**
- Niejasnych poleceń: "Zrób to lepiej"
- Zbyt ogólnych zadań: "Napraw wszystkie błędy"
- Pracy bez kontekstu: Zawsze odwołuj się do RULES.md

---

**PAMIĘTAJ:** Augment to potężne narzędzie, ale TY jesteś architektem i ostatecznym decydentem. Używaj go mądrze, weryfikuj wszystko i nigdy nie rezygnuj z krytycznego myślenia.

---

**WERSJA:** 1.0  
**DATA:** 2024-12-13  
**PROJEKT:** SNIPERCOR HFT System
