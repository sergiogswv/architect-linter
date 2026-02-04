# Script unificado de instalación/actualización para Windows
# Detecta automáticamente si es instalación inicial o actualización

Write-Host "===========================================  ARCHITECT-LINTER SETUP" -ForegroundColor Cyan
Write-Host ""

# Detectar si ya está instalado
$binPath = "$env:USERPROFILE\bin\architect-linter.exe"
$isUpdate = Test-Path $binPath

if ($isUpdate) {
    $mode = "actualizacion"
    Write-Host "Actualizando..." -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Version actual instalada:" -ForegroundColor Cyan
    & $binPath --version
    Write-Host ""
} else {
    $mode = "instalacion"
    Write-Host "Primera instalacion detectada" -ForegroundColor Green
    Write-Host ""
}

Write-Host "Compilando en modo release..." -ForegroundColor Cyan
cargo build --release

if ($LASTEXITCODE -eq 0) {
    Write-Host "Compilacion exitosa." -ForegroundColor Green
    Write-Host ""

    # Crear carpeta bin si no existe
    $destPath = "$env:USERPROFILE\bin"
    if (!(Test-Path $destPath)) {
        Write-Host "Creando carpeta $destPath..." -ForegroundColor Yellow
        New-Item -ItemType Directory -Path $destPath | Out-Null
    }

    # Copiar el binario
    if ($isUpdate) {
        Write-Host "Actualizando binario en $destPath..." -ForegroundColor Cyan
    } else {
        Write-Host "Instalando binario en $destPath..." -ForegroundColor Cyan
    }

    Copy-Item "target\release\architect-linter.exe" -Destination "$destPath\architect-linter.exe" -Force

    Write-Host ""
    if ($isUpdate) {
        Write-Host "Actualizacion exitosa!" -ForegroundColor Green
    } else {
        Write-Host "Instalacion exitosa!" -ForegroundColor Green
    }
    Write-Host ""

    Write-Host "Nueva version:" -ForegroundColor Cyan
    & "$destPath\architect-linter.exe" --version
    Write-Host ""

    if ($isUpdate) {
        Write-Host "La nueva version ya esta disponible." -ForegroundColor Green
        Write-Host "Cierra y vuelve a abrir tu terminal para usarla." -ForegroundColor Yellow
    } else {
        # Verificar si está en el PATH solo en instalación
        $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
        if ($currentPath -notlike "*$destPath*") {
            Write-Host "IMPORTANTE: Debes agregar $destPath a tu PATH" -ForegroundColor Yellow
            Write-Host ""
            Write-Host "Opcion 1 - Agregar automaticamente (ejecuta PowerShell como Administrador):" -ForegroundColor Cyan
            Write-Host ""
            $pathCommand = @"
`$oldPath = [Environment]::GetEnvironmentVariable('Path', 'User')
`$newPath = "`$oldPath;$destPath"
[Environment]::SetEnvironmentVariable('Path', `$newPath, 'User')
"@
            Write-Host $pathCommand -ForegroundColor White
            Write-Host ""
            Write-Host "Opcion 2 - Agregar manualmente:" -ForegroundColor Cyan
            Write-Host "  1. Presiona Win + X y selecciona 'Sistema'" -ForegroundColor White
            Write-Host "  2. Click en 'Configuracion avanzada del sistema'" -ForegroundColor White
            Write-Host "  3. Click en 'Variables de entorno'" -ForegroundColor White
            Write-Host "  4. En 'Variables de usuario', selecciona 'Path' y click 'Editar'" -ForegroundColor White
            Write-Host "  5. Click 'Nuevo' y agrega: $destPath" -ForegroundColor White
            Write-Host "  6. Click 'Aceptar' en todas las ventanas" -ForegroundColor White
            Write-Host ""
        } else {
            Write-Host "Ahora puedes usar 'architect-linter' en cualquier carpeta." -ForegroundColor Green
            Write-Host ""
            Write-Host "Para verificar la instalacion:" -ForegroundColor Cyan
            Write-Host "  architect-linter --help" -ForegroundColor White
            Write-Host ""
        }
    }
    Write-Host ""
} else {
    Write-Host "Error en la compilacion." -ForegroundColor Red
    Write-Host "Asegurate de:" -ForegroundColor Yellow
    Write-Host "  1. Tener Rust instalado (https://rustup.rs/)" -ForegroundColor White
    Write-Host "  2. Estar en el directorio del proyecto architect-linter" -ForegroundColor White
}
