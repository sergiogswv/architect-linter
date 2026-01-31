# Guía de Instalación para Windows

## Instalación Rápida (Recomendada)

### Paso 1: Clonar el repositorio
```powershell
git clone https://github.com/sergio/architect-linter.git
cd architect-linter
```

### Paso 2: Ejecutar el script de instalación

**Nota**: Es normal que aparezca un error sobre políticas de ejecución. Usa este comando:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\install.ps1
```

**Explicación de los flags**:
- `-NoProfile`: Evita cargar tu perfil de PowerShell (previene errores de `oh-my-posh` u otros)
- `-ExecutionPolicy Bypass`: Permite ejecutar el script una sola vez sin cambiar configuraciones del sistema

### Paso 3: Agregar al PATH

El script te mostrará instrucciones para agregar la carpeta al PATH. Tienes 2 opciones:

#### Opción A: Automáticamente (Más rápido)
1. Abre **PowerShell como Administrador** (Win + X → "Terminal (Administrador)")
2. Ejecuta estos comandos:
```powershell
$oldPath = [Environment]::GetEnvironmentVariable('Path', 'User')
$newPath = "$oldPath;C:\Users\TU_USUARIO\bin"
[Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
```
**Reemplaza `TU_USUARIO` con tu nombre de usuario de Windows**

#### Opción B: Manualmente
1. Presiona `Win + X` → "Sistema"
2. Click en "Configuración avanzada del sistema"
3. Click en "Variables de entorno"
4. En "Variables de usuario", selecciona "Path" → "Editar"
5. Click "Nuevo" y agrega: `C:\Users\TU_USUARIO\bin`
6. Click "Aceptar" en todas las ventanas

### Paso 4: Reiniciar la terminal

**IMPORTANTE**: Cierra TODAS las ventanas de PowerShell/Terminal y abre una nueva.

### Paso 5: Verificar la instalación
```powershell
architect-linter --version
```

Deberías ver: `architect-linter 0.8.0`

---

## Problema Común: "La ejecución de scripts está deshabilitada"

Si intentas ejecutar `.\install.ps1` directamente y recibes:
```
No se puede cargar el archivo porque la ejecución de scripts está deshabilitada en este sistema.
```

Esto es normal en Windows por razones de seguridad. **Usa la solución de la Instalación Rápida** arriba (con `-ExecutionPolicy Bypass`).

### Alternativas si el script no funciona

---

## Solución 1: Bypass Temporal (Más Rápido) ✅

Ejecuta el script con un bypass de una sola vez:

```powershell
powershell -ExecutionPolicy Bypass -File .\install.ps1
```

**Ventaja**: No cambia ninguna configuración de tu sistema, solo ejecuta este script.

---

## Solución 2: Habilitar Scripts para tu Usuario

Si planeas ejecutar scripts de PowerShell regularmente, habilita la política para tu usuario:

```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

Luego ejecuta:
```powershell
.\install.ps1
```

**Qué hace**: Permite ejecutar scripts locales, pero aún bloquea scripts descargados de internet (a menos que estén firmados).

---

## Solución 3: Instalación Manual (Sin Scripts)

Si prefieres no usar PowerShell scripts en absoluto:

### Paso 1: Compilar el proyecto
```powershell
cargo build --release
```

### Paso 2: Crear carpeta para binarios
```powershell
mkdir $env:USERPROFILE\bin -Force
```

### Paso 3: Copiar el ejecutable
```powershell
copy target\release\architect-linter.exe $env:USERPROFILE\bin\architect-linter.exe
```

### Paso 4: Agregar al PATH

**Opción A - Usar PowerShell (Requiere ejecutar como Administrador)**:
```powershell
$oldPath = [Environment]::GetEnvironmentVariable('Path', 'User')
$newPath = "$oldPath;$env:USERPROFILE\bin"
[Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
```

**Opción B - Manualmente**:
1. Presiona `Win + X` y selecciona "Sistema"
2. Click en "Configuración avanzada del sistema"
3. Click en "Variables de entorno"
4. En "Variables de usuario", selecciona "Path" y click "Editar"
5. Click "Nuevo" y agrega: `C:\Users\TU_USUARIO\bin`
6. Click "Aceptar" en todas las ventanas

### Paso 5: Verificar
Reinicia tu terminal y ejecuta:
```powershell
architect-linter --help
```

---

## Verificación y Uso

### Verificar instalación
```powershell
architect-linter --version
# Salida: architect-linter 0.8.0

architect-linter --help
# Muestra la ayuda completa
```

### Primer uso
```powershell
cd C:\ruta\a\tu\proyecto
architect-linter
```

El linter:
1. Detectará tu framework automáticamente
2. Te guiará para crear el `architect.json` (primera vez)
3. Analizará tu código y mostrará violaciones arquitectónicas

---

## Desinstalar

Si quieres desinstalar el linter:

```powershell
# Eliminar el binario
del $env:USERPROFILE\bin\architect-linter.exe

# Opcionalmente, eliminar la carpeta bin si está vacía
rmdir $env:USERPROFILE\bin
```

Luego elimina `%USERPROFILE%\bin` de tu PATH siguiendo los pasos de la Opción B al revés.

---

## Problemas Comunes

### "cargo: command not found"
Necesitas instalar Rust primero:
1. Descarga desde: https://rustup.rs/
2. Ejecuta el instalador
3. Reinicia tu terminal
4. Verifica con: `cargo --version`

### "El binario no se encuentra después de instalarlo"
1. **Verifica que agregaste al PATH correctamente**:
   ```powershell
   $env:Path -split ';' | Select-String 'bin'
   ```
   Deberías ver `C:\Users\TU_USUARIO\bin` en la lista

2. **IMPORTANTE**: Cierra TODAS las ventanas de PowerShell/Terminal
   - Las variables de entorno solo se recargan en nuevas sesiones
   - No basta con abrir una nueva pestaña, debes cerrar todas las ventanas

3. **Abre una nueva terminal** y prueba de nuevo:
   ```powershell
   architect-linter --version
   ```

4. Si usas VSCode, recarga la ventana (Ctrl+Shift+P → "Reload Window")

### "Access Denied" al agregar al PATH
Necesitas ejecutar PowerShell como Administrador:
1. Busca "PowerShell" en el menú inicio
2. Click derecho → "Ejecutar como administrador"
3. Ejecuta el comando de agregar al PATH

---

## Soporte

Si tienes problemas, abre un issue en: https://github.com/sergio/architect-linter/issues
