# Architect Linter

**Versión:** 1.0.0

Un linter de arquitectura de software escrito en Rust que valida reglas arquitectónicas en proyectos TypeScript/JavaScript mediante un motor de reglas dinámicas. Asegura que el diseño del software (Hexagonal, Clean, MVC, etc.) se respete sin importar quién escriba el código.

## Características

- **Motor de Reglas Dinámicas**: Define restricciones personalizadas entre capas mediante `architect.json`
- **Detección Automática de Framework**: Reconoce NestJS, React, Angular, Express y sugiere configuraciones óptimas
- **Patrones Arquitectónicos**: Soporte para Hexagonal, Clean Architecture, MVC y más
- **Validación de Importaciones**: Detecta y bloquea importaciones que violan la arquitectura definida
- **Control de Complejidad**: Valida que las funciones no excedan límites configurables de líneas
- **Procesamiento Paralelo**: Análisis ultrarrápido usando procesamiento multi-hilo con Rayon
- **Reportes Visuales**: Errores detallados y coloridos con ubicación exacta del problema
- **Modo Interactivo**: Configuración guiada en primera ejecución
- **Integración con Git Hooks**: Compatible con Husky para validación pre-commit automática

## Inicio Rápido

### Opción 1: Instalación Global (Recomendado)

La instalación global te permite ejecutar `architect-linter` desde cualquier directorio.

#### Linux / macOS
```bash
git clone https://github.com/sergio/architect-linter.git
cd architect-linter
chmod +x setup.sh
./setup.sh
```

#### Windows (PowerShell)
```powershell
git clone https://github.com/sergiogswv/architect-linter.git
cd architect-linter

# Ejecutar el script de instalación (evita errores de políticas de ejecución)
powershell -NoProfile -ExecutionPolicy Bypass -File .\setup.ps1
```

**Después de la instalación**:
1. Abre PowerShell como Administrador
2. Ejecuta los comandos que el script te muestra para agregar al PATH
3. **Cierra TODAS las terminales** y abre una nueva
4. Verifica: `architect-linter --version`

📖 **Guía completa para Windows con solución de problemas**: [INSTALL_WINDOWS.md](INSTALL_WINDOWS.md)

El script `setup.sh` / `setup.ps1` automáticamente:
1. Detecta si es instalación inicial o actualización
2. Compila el proyecto en modo release
3. Mueve el binario a una ubicación global (`/usr/local/bin` en Linux/macOS, `%USERPROFILE%\bin` en Windows)
4. En instalación: Configura el PATH si es necesario
5. En actualización: Muestra la versión anterior y la nueva

### Opción 2: Compilación Manual

#### Linux / macOS
```bash
git clone https://github.com/sergio/architect-linter.git
cd architect-linter
cargo build --release

# Mover a una carpeta en tu PATH
sudo cp target/release/architect-linter /usr/local/bin/
```

#### Windows (Instalación Manual)
```powershell
git clone https://github.com/sergio/architect-linter.git
cd architect-linter
cargo build --release

# Crear carpeta bin si no existe
mkdir $env:USERPROFILE\bin -Force

# Copiar el binario
copy target\release\architect-linter.exe $env:USERPROFILE\bin\

# Agregar al PATH (ejecutar PowerShell como administrador)
$oldPath = [Environment]::GetEnvironmentVariable('Path', 'User')
$newPath = "$oldPath;$env:USERPROFILE\bin"
[Environment]::SetEnvironmentVariable('Path', $newPath, 'User')

# Reinicia tu terminal para que los cambios surtan efecto
```

### Primer Uso

```bash
# Si instalaste globalmente
architect-linter /ruta/a/tu/proyecto

# O si usas el binario local
./target/release/architect-linter /ruta/a/tu/proyecto

# Modo interactivo (te muestra proyectos disponibles)
architect-linter
```

**Primera ejecución**: Si no existe `architect.json`, el linter detectará automáticamente tu framework y te guiará con un wizard interactivo para configurar las reglas arquitectónicas.

## Actualización

Si ya tienes architect-linter instalado y quieres actualizar a la versión más reciente, usa el **mismo script de instalación**:

### Linux / macOS
```bash
cd /ruta/al/repositorio/architect-linter
git pull origin master  # O la rama que uses
./setup.sh
```

### Windows (PowerShell)
```powershell
cd C:\ruta\al\repositorio\architect-linter
git pull origin master  # O la rama que uses
powershell -NoProfile -ExecutionPolicy Bypass -File .\setup.ps1
```

**El script detecta automáticamente** si ya tienes architect-linter instalado:
- ✅ Si existe: Modo actualización (muestra versión anterior → compila → instala → muestra nueva versión)
- ✅ Si no existe: Modo instalación (compila → instala → configura PATH si es necesario)

**Importante para Windows**: Después de actualizar, cierra y vuelve a abrir tu terminal para que los cambios surtan efecto.

### Instalación/Actualización Manual

Si prefieres hacerlo manualmente sin usar el script:

```bash
# 1. Actualizar el código (si ya lo tienes clonado)
git pull origin master

# 2. Compilar
cargo build --release

# 3. Copiar el binario

# Linux/macOS
sudo cp target/release/architect-linter /usr/local/bin/

# Windows PowerShell
copy target\release\architect-linter.exe $env:USERPROFILE\bin\
```

### Integración con Git Hooks (Recomendado)

Valida la arquitectura automáticamente antes de cada commit usando Husky.

#### Paso 1: Instalar Husky en tu proyecto
```bash
cd /ruta/a/tu/proyecto
npx husky-init && npm install
```

#### Paso 2: Configurar el Pre-Commit Hook

**Opción A: Con instalación global (Recomendado)**
```bash
#!/bin/sh
. "$(dirname "$0")/_/husky.sh"

echo "🏗️  Validando arquitectura antes del commit..."
architect-linter .
```

**Opción B: Con ruta específica**
```bash
#!/bin/sh
. "$(dirname "$0")/_/husky.sh"

echo "🏗️  Validando arquitectura antes del commit..."
"/ruta/completa/architect-linter/target/release/architect-linter" .
```

Edita el archivo `.husky/pre-commit` con el contenido de tu preferencia y dale permisos de ejecución:

```bash
chmod +x .husky/pre-commit
```

📖 **Guía completa de integración**: [NESTJS_INTEGRATION.md](NESTJS_INTEGRATION.md)

## Motor de Reglas Dinámicas

El architect-linter utiliza un sistema de reglas dinámicas definidas en `architect.json` que permiten restringir qué carpetas pueden interactuar entre sí, asegurando que el diseño arquitectónico se respete.

### Concepto

Una regla prohibida define una relación **Origen (from)** → **Destino (to)**:
- Si un archivo ubicado en la ruta **"Origen"** intenta importar algo de la ruta **"Destino"**, el linter generará un error de arquitectura.

### Estructura en architect.json

```json
{
  "max_lines_per_function": 40,
  "architecture_pattern": "Hexagonal",
  "forbidden_imports": [
    {
      "from": "/domain/",
      "to": "/infrastructure/"
    }
  ]
}
```

#### Propiedades

- **`max_lines_per_function`** (número): Límite de líneas por método/función
- **`architecture_pattern`** (string): Patrón arquitectónico (`"Hexagonal"`, `"Clean"`, `"MVC"`, `"Ninguno"`)
- **`forbidden_imports`** (array): Lista de reglas con:
  - **`from`**: Patrón de carpeta/archivo donde se aplica la restricción
  - **`to`**: Patrón de carpeta/archivo prohibido importar

### Cómo Funciona el Motor

1. **Escaneo**: Convierte todas las rutas a minúsculas para evitar errores de mayúsculas
2. **Match**: Por cada archivo, verifica si su ruta contiene el texto definido en `from`
3. **Validación**: Si hay coincidencia, analiza cada `import`. Si el origen del import contiene `to`, dispara una violación

### Casos de Uso Comunes

#### A. Arquitectura Hexagonal (Preservar el Core)

Evita que la lógica de negocio dependa de detalles de implementación (Base de datos, APIs externas).

```json
{
  "from": "/domain/",
  "to": "/infrastructure/"
}
```

**Resultado**: Si intentas importar un TypeORM Repository dentro de una Entity de dominio, el linter bloqueará el commit.

#### B. Desacoplamiento de Capas (NestJS/MVC)

Evita que los Controladores se salten la capa de servicio.

```json
{
  "from": ".controller.ts",
  "to": ".repository"
}
```

**Resultado**: Obliga a inyectar un Service en lugar de consultar la base de datos directamente desde el entry point.

## Guía de Reglas por Patrón Arquitectónico

### Tabla Comparativa de Restricciones

| Patrón | Capa Origen (`from`) | Carpeta Prohibida (`to`) | Razón Técnica |
|--------|---------------------|--------------------------|---------------|
| **Hexagonal** | `/domain/` | `/infrastructure/` | El núcleo no debe conocer la base de datos o APIs externas |
| **Hexagonal** | `/domain/` | `/application/` | El dominio no debe depender de casos de uso específicos |
| **Clean** | `/entities/` | `/use-cases/` | Las reglas de negocio de alto nivel no deben conocer la orquestación |
| **Clean** | `/use-cases/` | `/controllers/` | La lógica no debe saber quién la llama (web, CLI, etc.) |
| **MVC** | `.controller.ts` | `.repository` | Desacoplamiento: El controlador solo habla con servicios |
| **MVC** | `.service.ts` | `.controller.ts` | Evitar dependencias circulares y mantener lógica pura |

### Ejemplo: Clean Architecture

```json
{
  "max_lines_per_function": 35,
  "architecture_pattern": "Clean",
  "forbidden_imports": [
    {
      "from": "/entities/",
      "to": "/use-cases/",
      "reason": "Las entidades son el corazón y deben ser agnósticas a los casos de uso."
    },
    {
      "from": "/use-cases/",
      "to": "/infrastructure/",
      "reason": "La lógica de aplicación no debe importar implementaciones directas como TypeORM o Axios."
    }
  ]
}
```

### Ejemplo: Arquitectura Hexagonal

```json
{
  "max_lines_per_function": 40,
  "architecture_pattern": "Hexagonal",
  "forbidden_imports": [
    {
      "from": "/domain/",
      "to": "/infrastructure/"
    },
    {
      "from": "/application/",
      "to": "/infrastructure/"
    }
  ]
}
```

## Uso

### Modo Interactivo (Primera Ejecución)

```bash
./target/release/architect-linter
```

Si no existe `architect.json`, el linter:
1. Detecta automáticamente el framework (NestJS, React, Angular, Express)
2. Sugiere un patrón arquitectónico
3. Propone un límite de líneas basado en el framework detectado
4. Crea el archivo `architect.json` con la configuración seleccionada

### Modo Automático (Ejecuciones Posteriores)

Cuando ya existe `architect.json`, el linter ejecuta silenciosamente:

```bash
./target/release/architect-linter /ruta/al/proyecto
```

o

```bash
cargo run -- /ruta/al/proyecto
```

### Argumentos CLI

```bash
architect-linter [OPCIONES] [RUTA]
```

**Opciones**:
- `-v, --version`: Muestra la versión del linter
- `-h, --help`: Muestra la ayuda completa
- **Sin argumentos**: Modo interactivo, muestra menú de proyectos disponibles
- **Con ruta**: `architect-linter /ruta/proyecto` - Analiza el proyecto especificado

**Ejemplos**:
```bash
architect-linter --version          # Muestra: architect-linter 1.0.0
architect-linter --help             # Muestra ayuda completa
architect-linter                    # Modo interactivo
architect-linter .                  # Analiza directorio actual
architect-linter /ruta/proyecto     # Analiza proyecto específico
```

## El Flujo de Trabajo Completo

### Primera vez usando el linter

1. **Commit inicial**: Al ejecutar `git commit`, Husky lanza el linter automáticamente
2. **Discovery automático**: Si es la primera vez (no existe `architect.json`), el linter:
   - Lee tu `package.json` y estructura de carpetas
   - Detecta el framework (NestJS, React, Angular, Express)
   - Consulta la IA para sugerir límites de líneas y reglas arquitectónicas
3. **Configuración guiada**: Te muestra las sugerencias y solicita confirmación
4. **Persistencia**: Una vez aceptas, crea `architect.json` y valida el código
5. **Resultado**: Si no hay violaciones, el commit continúa; si las hay, se aborta mostrando los errores

### Ejecuciones posteriores

Una vez existe `architect.json`:
- El linter carga silenciosamente la configuración
- Valida el código instantáneamente (gracias a Rust)
- Muestra violaciones si existen o permite el commit

## FAQ (Preguntas Frecuentes)

### ¿Qué hago si obtengo un error de configuración en architect.json?

El linter valida automáticamente el archivo `architect.json` y muestra mensajes de error claros con sugerencias de cómo arreglarlos. Los errores más comunes son:

- **JSON con sintaxis inválida**: Falta una coma, llave o hay caracteres extra
- **Campos faltantes**: `max_lines_per_function`, `architecture_pattern` o `forbidden_imports`
- **Tipos incorrectos**: Por ejemplo, poner `"50"` (string) en lugar de `50` (número)
- **Valores inválidos**: Patrón arquitectónico que no existe, o `max_lines_per_function` en 0

**Cada error incluye:**
- ✅ Descripción clara del problema
- ✅ Sugerencia de cómo arreglarlo
- ✅ Ejemplo de código correcto

**Guía completa de errores:** Ver [CONFIG_ERRORS.md](CONFIG_ERRORS.md) para ejemplos detallados de todos los errores posibles.

### ¿Qué pasa si los tests fallan?
El commit se aborta automáticamente. Git te mostrará exactamente qué archivo y línea está rompiendo la arquitectura, con contexto visual del error.

### ¿Puedo saltarme el linter en caso de emergencia?
Sí, puedes usar `git commit --no-verify` para omitir los hooks, pero ¡úsalo con responsabilidad! El Arquitecto Virtual se sentirá decepcionado 😢

### ¿Necesito internet para usar el linter?
Solo la **primera vez** para que la IA sugiera las reglas (configuración inicial asistida). Una vez creado el `architect.json`, el linter funciona **100% offline** y es instantáneo.

### ¿Funciona con JavaScript además de TypeScript?
Sí, el linter soporta tanto TypeScript (`.ts`, `.tsx`) como JavaScript (`.js`, `.jsx`).

### ¿Cómo actualizo las reglas después de la configuración inicial?
Simplemente edita el archivo `architect.json` manualmente. El linter cargará automáticamente los cambios en la próxima ejecución.

### ¿Qué variables de entorno necesito para la IA?
Para la configuración asistida por IA necesitas:
- `ANTHROPIC_AUTH_TOKEN`: Tu API key de Anthropic
- `ANTHROPIC_BASE_URL`: URL del endpoint de la API

Si no están configuradas, el linter te lo indicará en la primera ejecución.

## Ejemplo de Salida

### Primera Ejecución (Modo Configuración)
```
🏛️  WELCOME TO ARCHITECT-LINTER
📝 No encontré 'architect.json'. Vamos a configurar tu proyecto.
? Confirmar Framework (Detectado: NestJS) › NestJS
? ¿Qué patrón arquitectónico quieres aplicar? › Hexagonal
? Límite de líneas por método › 40
✅ Configuración guardada en 'architect.json'
```

### Ejecuciones Posteriores (Modo Automático)
```
🏛️  WELCOME TO ARCHITECT-LINTER

📌 Violación en: src/domain/user.entity.ts

  × Violación de Arquitectura
   ╭─[src/domain/user.entity.ts:3:1]
   │
 3 │ import { Repository } from 'typeorm';
   │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   │ Restricción: Archivos en '/domain/' no pueden importar de '/infrastructure/'.
   ╰────

❌ Se encontraron 1 violaciones.
```

## Estructura del Proyecto

```
architect-linter/
├── src/
│   ├── main.rs                 # Orquestación, configuración interactiva, recolección de archivos
│   ├── analyzer.rs             # Análisis de TypeScript, validación de reglas dinámicas
│   ├── config.rs               # Tipos: LinterContext, ArchPattern, Framework, ForbiddenRule
│   └── detector.rs             # Detección de framework y sugerencias LOC
├── Cargo.toml                  # Dependencias y configuración del proyecto
├── README.md                   # Esta documentación
├── CHANGELOG.md                # Historial de versiones
├── NESTJS_INTEGRATION.md       # Guía de integración con Git Hooks
└── pre-commit.example          # Plantilla para Husky
```

## Tecnologías

- **swc_ecma_parser**: Parser de TypeScript/JavaScript de alto rendimiento
- **rayon**: Procesamiento paralelo automático
- **miette**: Reportes de diagnóstico elegantes con contexto
- **walkdir**: Traversal eficiente de directorios
- **dialoguer**: UI interactiva para terminal
- **indicatif**: Barras de progreso
- **serde_json**: Parseo de configuración JSON

## Reglas Implementadas

### 1. Importaciones Prohibidas (Dinámicas)
Definidas en `architect.json` con el formato `from` → `to`. El motor valida cada `import` contra las reglas configuradas.

### 2. Complejidad de Funciones
Cuenta las líneas de cada método/función y alerta si excede `max_lines_per_function`.

### 3. Regla Extra: Controller → Repository (NestJS)
Prohibición hardcoded: archivos que contienen `"controller"` no pueden importar `".repository"`, reforzando el patrón MVC.

## Roadmap

### Completado ✅
- [x] Motor de reglas dinámicas con `forbidden_imports`
- [x] Detección automática de framework (NestJS, React, Angular, Express)
- [x] Configuración interactiva en primera ejecución
- [x] Soporte para patrones: Hexagonal, Clean, MVC
- [x] Procesamiento paralelo con Rayon
- [x] Integración con Git Hooks (Husky)
- [x] Arquitectura modular (analyzer, config, detector)
- [x] Reportes elegantes con Miette
- [x] Soporte para JavaScript (.js, .jsx)
- [x] Validación de esquema JSON con mensajes de error claros

### Próximamente 🚧
- [ ] Exportación de reportes (JSON, HTML, Markdown)
- [ ] Modo watch para desarrollo continuo
- [ ] Análisis incremental con caché

### Futuro 🔮
- [ ] Reglas personalizadas mediante plugins en Rust/WASM
- [ ] Integración nativa con CI/CD (GitHub Actions, GitLab CI)
- [ ] Configuración de severidad por regla (error, warning, info)
- [ ] Dashboard web para visualizar violaciones históricas
- [ ] Soporte para más lenguajes (Python, Go, Java)

## Contribuir

Las contribuciones son bienvenidas. Por favor:

1. Fork el repositorio
2. Crea una rama para tu feature (`git checkout -b feature/amazing-feature`)
3. Commit tus cambios (`git commit -m 'Add amazing feature'`)
4. Push a la rama (`git push origin feature/amazing-feature`)
5. Abre un Pull Request

## Licencia

Este proyecto está bajo la licencia MIT.

## Autor

Sergio Guadarrama - [GitHub](https://github.com/sergiogswv)

## Changelog

Ver [CHANGELOG.md](CHANGELOG.md) para el historial completo de versiones.

### v1.0.0 (2026-01-31) - Primera Versión Estable
- 🎉 Primera versión estable lista para producción
- 🚀 Flags CLI: `--version` y `--help` implementados
- 📦 Instalación optimizada para Windows con scripts mejorados
- 📚 Documentación completa de instalación en Windows con solución de problemas
- ✅ Validación completa en proyectos reales

### v0.8.0 (2026-01-31) - Configuración Asistida por IA
- 🤖 Integración con Claude (Anthropic API) para sugerencias arquitectónicas inteligentes
- 🔍 Discovery automático del proyecto con análisis de dependencias y estructura
- 📦 Scripts de instalación automatizada para Linux/macOS y Windows
- 💡 Wizard interactivo para confirmación de reglas sugeridas por IA
- 📚 FAQ completa y documentación del flujo de trabajo
- 🎯 Módulo UI separado para mejor organización del código

### v0.7.0 (2026-01-30) - Motor de Reglas Dinámicas
- ✨ Motor de reglas dinámicas completamente funcional
- 🔍 Detección automática de framework con módulo `detector.rs`
- 🎯 Configuración interactiva en primera ejecución
- 📐 Soporte para patrones arquitectónicos: Hexagonal, Clean, MVC
- 🛠️ Corrección de errores de compilación y warnings
- 📚 Documentación actualizada con ejemplos por patrón
