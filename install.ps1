# Script de instalacion para Windows
Write-Host "Compilando Architect Linter en modo release..." -ForegroundColor Cyan
cargo build --release

if ($LASTEXITCODE -eq 0) {
    Write-Host "Compilacion exitosa." -ForegroundColor Green
    Write-Host ""

    # Crear carpeta bin en el perfil del usuario si no existe
    $binPath = "$env:USERPROFILE\bin"
    if (!(Test-Path $binPath)) {
        Write-Host "Creando carpeta $binPath..." -ForegroundColor Yellow
        New-Item -ItemType Directory -Path $binPath | Out-Null
    }

    # Copiar el binario
    Write-Host "Copiando binario a $binPath..." -ForegroundColor Cyan
    Copy-Item "target\release\architect-linter.exe" -Destination "$binPath\architect-linter.exe" -Force

    Write-Host ""
    Write-Host "Instalado exitosamente en $binPath" -ForegroundColor Green
    Write-Host ""

    # Verificar si la carpeta esta en el PATH
    $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($currentPath -notlike "*$binPath*") {
        Write-Host "IMPORTANTE: Debes agregar $binPath a tu PATH" -ForegroundColor Yellow
        Write-Host ""
        Write-Host "Opcion 1 - Agregar automaticamente (ejecuta PowerShell como Administrador):" -ForegroundColor Cyan
        Write-Host ""
        $pathCommand = @"
`$oldPath = [Environment]::GetEnvironmentVariable('Path', 'User')
`$newPath = "`$oldPath;$binPath"
[Environment]::SetEnvironmentVariable('Path', `$newPath, 'User')
"@
        Write-Host $pathCommand -ForegroundColor White
        Write-Host ""
        Write-Host "Opcion 2 - Agregar manualmente:" -ForegroundColor Cyan
        Write-Host "  1. Presiona Win + X y selecciona 'Sistema'" -ForegroundColor White
        Write-Host "  2. Click en 'Configuracion avanzada del sistema'" -ForegroundColor White
        Write-Host "  3. Click en 'Variables de entorno'" -ForegroundColor White
        Write-Host "  4. En 'Variables de usuario', selecciona 'Path' y click 'Editar'" -ForegroundColor White
        Write-Host "  5. Click 'Nuevo' y agrega: $binPath" -ForegroundColor White
        Write-Host "  6. Click 'Aceptar' en todas las ventanas" -ForegroundColor White
        Write-Host ""
    } else {
        Write-Host "La carpeta ya esta en tu PATH" -ForegroundColor Green
        Write-Host ""
        Write-Host "Reinicia tu terminal y ejecuta:" -ForegroundColor Cyan
        Write-Host "  architect-linter --help" -ForegroundColor White
        Write-Host ""
    }
} else {
    Write-Host "Error en la compilacion." -ForegroundColor Red
    Write-Host "Asegurate de tener Rust instalado desde: https://rustup.rs/" -ForegroundColor Yellow
}
