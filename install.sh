#!/bin/bash

echo "🦀 Compilando Architect Linter en modo release..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "✅ Compilación exitosa."
    echo "📦 Instalando binario en /usr/local/bin..."
    sudo cp target/release/architect-linter /usr/local/bin/

    if [ $? -eq 0 ]; then
        echo "🚀 ¡Listo! Ahora puedes usar 'architect-linter' en cualquier carpeta."
        echo ""
        echo "Para verificar la instalación, ejecuta:"
        echo "  architect-linter --help"
    else
        echo "⚠️  Error al copiar el binario. Intenta manualmente:"
        echo "  sudo cp target/release/architect-linter /usr/local/bin/"
    fi
else
    echo "❌ Error en la compilación. Asegúrate de tener Rust instalado."
    echo "Puedes instalar Rust desde: https://rustup.rs/"
fi
