#!/bin/bash
set -e # Zatrzymuje skrypt w przypadku błędu

echo "🚀 --- Przygotowywanie środowiska dla Rdzenia w Ruście --- 🚀"

# Sprawdzenie czy Rust jest już zainstalowany
if command -v rustc &> /dev/null; then
    echo "✅ Rust jest już zainstalowany: $(rustc --version)"
else
    echo "📦 Instalacja Rusta..."
    # Instalacja Rusta
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    echo "✅ Rust został zainstalowany: $(rustc --version)"
fi

# Sprawdzenie czy Maturin jest już zainstalowany
if command -v maturin &> /dev/null; then
    echo "✅ Maturin jest już zainstalowany: $(maturin --version)"
else
    echo "📦 Instalacja Maturin do budowania rozszerzeń Pythona..."
    # Instalacja Maturin do budowania rozszerzeń Pythona
    pip install maturin
    echo "✅ Maturin został zainstalowany: $(maturin --version)"
fi

# Instalacja dodatkowych narzędzi Rust
echo "📦 Instalacja dodatkowych narzędzi Rust..."
rustup component add rustfmt clippy

# Sprawdzenie czy cargo-watch jest zainstalowany (przydatne do development)
if ! cargo install --list | grep -q "cargo-watch"; then
    echo "📦 Instalacja cargo-watch dla hot reload..."
    cargo install cargo-watch
fi

echo "🔧 Konfiguracja środowiska..."
# Dodanie cargo bin do PATH jeśli nie jest już dodane
if ! echo "$PATH" | grep -q "$HOME/.cargo/bin"; then
    echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
    export PATH="$HOME/.cargo/bin:$PATH"
fi

echo "✅ Środowisko Rusta jest w pełni gotowe do pracy z projektem sniper-core!"
echo "📋 Zainstalowane komponenty:"
echo "   - Rust: $(rustc --version)"
echo "   - Cargo: $(cargo --version)"
echo "   - Maturin: $(maturin --version)"
echo "   - Rustfmt: $(rustfmt --version)"
echo "   - Clippy: $(cargo clippy --version)"
echo ""
echo "🎯 Projekt gotowy do stworzenia architektury hybrydowej Rust + Python!"
echo "📖 Sprawdź AUGMENT_MEMORY_V1.md dla pełnej specyfikacji."
