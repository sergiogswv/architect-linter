#!/bin/bash

# Script unificado de instalación/actualización para Linux/macOS
# Detecta automáticamente si es instalación inicial o actualización

echo "🏛️  ARCHITECT-LINTER SETUP"
echo ""

# Detectar si ya está instalado
if command -v architect-linter &> /dev/null; then
    MODE="actualización"
    echo "📦 Versión actual instalada:"
    architect-linter --version
    echo ""
else
    MODE="instalación"
    echo "📦 Primera instalación detectada"
    echo ""
fi

echo "🦀 Compilando en modo release..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "✅ Compilación exitosa."
    echo ""

    if [ "$MODE" = "actualización" ]; then
        echo "📋 Actualizando binario en /usr/local/bin..."
    else
        echo "📋 Instalando binario en /usr/local/bin..."
    fi

    sudo cp target/release/architect-linter /usr/local/bin/

    if [ $? -eq 0 ]; then
        echo ""
        if [ "$MODE" = "actualización" ]; then
            echo "✨ ¡Actualización exitosa!"
        else
            echo "✨ ¡Instalación exitosa!"
        fi
        echo ""
        echo "Nueva versión:"
        architect-linter --version
        echo ""

        if [ "$MODE" = "instalación" ]; then
            echo "🚀 Ahora puedes usar 'architect-linter' en cualquier carpeta."
            echo ""
            echo "Para verificar la instalación, ejecuta:"
            echo "  architect-linter --help"
        else
            echo "💡 La nueva versión ya está disponible en tu terminal."
        fi
        echo ""
    else
        echo "⚠️  Error al copiar el binario. Intenta manualmente:"
        echo "  sudo cp target/release/architect-linter /usr/local/bin/"
    fi
else
    echo "❌ Error en la compilación."
    echo "Asegúrate de:"
    echo "  1. Tener Rust instalado (https://rustup.rs/)"
    echo "  2. Estar en el directorio del proyecto architect-linter"
fi
