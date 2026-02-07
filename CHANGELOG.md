# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [3.2.0] - 2026-02-07

### 🎉 DeepSeek Integration & Multi-Model Fallback System

This release introduces official support for DeepSeek as an AI provider and a robust fallback system that automatically tries alternative AI models if the primary one fails.

### Added
- **DeepSeek Support**:
  - Official integration with DeepSeek API (OpenAI-compatible).
  - Default URL configuration for `https://api.deepseek.com`.
  - Automatic model discovery for DeepSeek endpoints.
- **Multi-Model Fallback System**:
  - Robust orchestration in `src/ai.rs` that catches API errors and retries with the next available configuration.
  - Automatic re-ordering of configurations to prioritize the user-selected model.
  - Real-time UI feedback when a model fails and a fallback is initiated.
  - Support for multiple AI configurations in `.architect.ai.json`.
- **New AI Providers in Wizard**:
  - Added **Kimi (Moonshot)** and **DeepSeek** to the interactive setup selection.
  - Streamlined provider-specific default URL suggestions.

### Changed
- **Config Architecture**:
  - `LinterContext` now stores `ai_configs` (a collection) instead of a single `ai_config`.
  - Updated AI discovery and auto-fix logic to leverage the fallback orchestrator.
- **Interactive Setup**:
  - Improved AI configuration loop allowing users to add multiple providers in a single session.
  - Explicit optional API Key handling for local providers like Ollama.

### Technical Details
- **Fallback Logic**: Sequential retry mechanism with O(N) complexity where N is the number of configured AI providers.
- **Standardization**: DeepSeek integration follows the OpenAI-compatible interface, ensuring consistency with Groq, Kimi, and Ollama.

## [3.1.0] - 2026-02-06

### 🎉 Multi-Language Support: PHP & Java

This release expands language support from 4 to 6 programming languages with the addition of PHP and Java parsers, along with comprehensive documentation updates and code cleanup.

### Added
- **PHP Parser** (`src/parsers/php.rs`):
  - Full Tree-sitter integration for PHP syntax
  - Support for `use`, `require`, `require_once`, `include`, and `include_once` statements
  - Pattern matching for PHP-specific import/require conventions
  - PHP-specific architectural violation detection
- **Java Parser** (`src/parsers/java.rs`):
  - Complete Tree-sitter grammar support for Java
  - Import statement extraction and analysis
  - Java package path pattern matching
  - Architectural rule enforcement for Java projects
- **Enhanced Documentation**:
  - Added professional project banner (`public/architect-linter-banner.png`)
  - Multi-language support table in README (English and Spanish)
  - Updated language coverage to 6 languages: TypeScript, JavaScript, Python, Go, PHP, Java
  - Improved setup scripts with better error handling
- **Tree-sitter Dependencies**:
  - Added `tree-sitter-php = "0.23.8"` to Cargo.toml
  - Added `tree-sitter-java = "0.23.4"` to Cargo.toml
- **Example Configuration**:
  - Updated `architect.json.example` with PHP and Java rule examples

### Changed
- **Setup Scripts**:
  - Enhanced `setup.sh` with better PATH configuration for Linux/macOS
  - Improved `setup.ps1` with robust Windows PATH handling
  - Better error messages and installation verification
- **Parser Architecture**:
  - Expanded `get_parser_for_file()` to support `.php` and `.java` extensions
  - Updated `supported_languages()` to include PHP and Java
  - Extended `Language` enum with `Php` and `Java` variants
- **File Discovery**:
  - Improved file collection to include PHP and Java files
  - Enhanced extension matching in analyzer modules

### Fixed
- **Dead Code Cleanup**:
  - Removed unused `LanguageInfo` struct from `src/parsers/mod.rs`
  - Eliminated unused `get_language_info()` method from `ArchitectParser` trait
  - Removed unused `language()` method from `ArchitectParser` trait
  - Cleaned up unnecessary imports of `Language` and `LanguageInfo` across all parser modules
  - Reduced codebase by 72 lines of dead code across 6 files
- **Compilation Warnings**:
  - Fixed all `#[warn(dead_code)]` warnings
  - Removed unused methods and structs from trait implementations

### Technical Details
- **Supported Languages**: TypeScript, JavaScript, Python, Go, PHP, Java (6 total)
- **Lines of Code Removed**: 72 lines of dead code eliminated
- **New Parsers**: 2 (PHP: 195 lines, Java: 185 lines)
- **Documentation Updates**: README files in both English and Spanish

### Security
- No security changes in this release

## [2.0.0] - 2026-02-04

### 🎉 Major Release: Circular Dependencies & Security

This major version introduces circular dependency detection, separated AI configuration for security, and improved visual experience.

### Added
- **🔴 Circular dependency detection**:
  - New `circular.rs` module with graph-based analysis
  - DFS (Depth-First Search) algorithm for cycle detection
  - Automatic import extraction from all project files
  - Relative path resolution (`../`, `./`)
  - Detailed cycle reporting with path visualization
  - Suggested solutions for breaking cycles
- **🔐 Separated AI configuration**:
  - `architect.json` for rules (sharable in repo)
  - `.architect.ai.json` for AI config (private, contains API keys)
  - Wizard for AI configuration on first run
  - Environment variable defaults (`ANTHROPIC_AUTH_TOKEN`, `ANTHROPIC_BASE_URL`, `ANTHROPIC_MODEL`)
  - `.gitignore` automatically includes `.architect.ai.json`
- **🪝 Automatic Husky setup**:
  - Pre-commit hook configuration during initial setup
  - Creates `.husky/pre-commit` automatically
  - Windows and Unix-compatible hooks
- **🎨 Enhanced visual experience**:
  - New ASCII art banner in `ui.rs`
  - Improved wizard prompts
  - Better error messages and formatting
- **📁 Example files**:
  - `.architect.ai.json.example` - AI configuration template
  - `.gitignore.example` - Template for projects using architect-linter
  - Updated `architect.json.example` without AI config

### Changed
- **AI configuration**:
  - Moved from environment variables to file-based config
  - More flexible: URL, API key, and model are now configurable
  - Backward compatible with environment variables as defaults
- **Setup flow**:
  - AI config wizard now runs before architecture discovery
  - Clear separation between rules and credentials
- **Documentation**:
  - Updated README with new features
  - Security best practices highlighted
  - Added circular dependency examples

### Security
- ⚠️ API keys are now stored in `.architect.ai.json` which is in `.gitignore`
- ✅ Each developer has their own AI configuration
- ✅ Rules in `architect.json` can be safely shared in repositories

### Technical Details
- **Graph algorithm**: O(V + E) complexity where V = files, E = imports
- **Path resolution**: Handles relative imports, index files, and multiple extensions
- **DFS implementation**: Recursive with recursion stack for cycle detection

### Changed
- **Major refactoring of main.rs**:
  - Reduced from 151 lines to 80 lines (-47%)
  - Moved `setup_or_load_config()` to `config.rs`
  - Moved CLI functions to new `cli.rs` module
  - Cleaner and more maintainable code structure
- **Scripts consolidation**:
  - 4 scripts → 2 scripts (install.sh, install.ps1, update.sh, update.ps1 → setup.sh, setup.ps1)
  - Single command for both installation and updates
- **Documentation language**:
  - All documentation translated to English
  - Code messages remain in Spanish (original language)

### Improved
- Architectural file detection for JavaScript (`.controller.js`, `.service.js`, etc.)
- CLI messages updated to mention "TypeScript/JavaScript"
- More robust and flexible rules validation engine
- Better Windows path handling with separator normalization

### Fixed
- Rules engine now correctly detects violations with relative imports
- Compilation warnings removed with `#[allow(dead_code)]` annotations
- Glob pattern matching works correctly with actual folder structure

### Documentation
- README translated to English
- CHANGELOG translated to English
- CONFIG_ERRORS documentation in English
- Spanish preserved for runtime messages

## [1.1.0] - 2026-02-03 (Deprecated)

### 🚀 Soporte Completo para JavaScript/React + Validación Robusta de Configuración

### Agregado
- **Validación de esquema JSON completa**:
  - Validación de sintaxis JSON antes de parsear
  - Validación de campos requeridos con mensajes específicos
  - Validación de tipos de datos (número, string, array, object)
  - Validación de valores (rangos, opciones válidas)
  - Detección de reglas duplicadas en `forbidden_imports`
  - Mensajes de error claros con sugerencias de solución
  - Cada error incluye ejemplo de código correcto
- **Documentación de errores**:
  - `CONFIG_ERRORS.md` con guía completa de errores comunes
  - Ejemplos de todos los tipos de errores posibles
  - Soluciones paso a paso para cada error
  - Ejemplos de configuraciones válidas por framework
- **Soporte para archivos JavaScript**:
  - Análisis de archivos `.js` y `.jsx` además de TypeScript
  - Parser automático según extensión (TypeScript vs JavaScript)
  - Soporte para JSX en archivos `.jsx` y `.tsx`
  - Decoradores habilitados en JavaScript
- **Motor de reglas mejorado**:
  - Matching inteligente de imports relativos (`../services/`, `./api/`)
  - Matching de imports con alias (`@/services/`, `@/api/`)
  - Normalización de patrones glob (`src/components/**` → `src/components/`)
  - Funciones helper `normalize_pattern()` y `matches_pattern()`
- **Scripts de actualización**:
  - `update.sh` para Linux/macOS
  - `update.ps1` para Windows
  - Muestran versión anterior y nueva después de actualizar
- **Documentación de actualización**:
  - Sección completa en README sobre cómo actualizar
  - Instrucciones para actualización automática y manual

### Mejorado
- Detección de archivos arquitectónicos para JavaScript (`.controller.js`, `.service.js`, etc.)
- Mensajes del CLI actualizados para mencionar "TypeScript/JavaScript"
- Motor de validación de reglas más robusto y flexible
- Mejor manejo de rutas en Windows con normalización de separadores

### Corregido
- Motor de reglas ahora detecta correctamente violaciones con imports relativos
- Warnings de compilación eliminados con anotaciones `#[allow(dead_code)]`
- Matching de patrones glob funciona correctamente con estructura de carpetas real

### Documentación
- README actualizado con soporte de JavaScript en FAQ
- Roadmap actualizado moviendo "Soporte JavaScript" a completado
- Ejemplos de uso para proyectos React con JavaScript

## [1.0.0] - 2026-01-31

### 🎉 Primera Versión Estable

Esta es la primera versión estable de Architect Linter, lista para uso en producción.

### Agregado
- **Flags CLI completos**:
  - `--version` / `-v`: Muestra la versión del linter
  - `--help` / `-h`: Muestra ayuda completa con ejemplos
- **Instalación mejorada para Windows**:
  - Script `install.ps1` optimizado sin emojis para evitar problemas de codificación
  - Instrucciones claras con flag `-NoProfile` para evitar errores de perfiles de PowerShell
  - Guía paso a paso para agregar al PATH (automático y manual)
- **Documentación completa de instalación**:
  - `INSTALL_WINDOWS.md` actualizado con instrucciones detalladas
  - Solución de problemas comunes
  - Verificación de instalación paso a paso
- Constante `VERSION` usando `CARGO_PKG_VERSION` para versiones consistentes

### Mejorado
- Función `print_help()` con formato claro y ejemplos de uso
- Manejo de argumentos CLI más robusto
- Validación de flags antes de procesar proyectos
- Documentación actualizada con comandos exactos probados en Windows

### Corregido
- Error al ejecutar `architect-linter --version` (antes trataba `--version` como ruta de archivo)
- Problemas de sintaxis en `install.ps1` con comillas y caracteres especiales
- Instrucciones de instalación ahora reflejan el proceso real probado

### Técnico
- Detección temprana de flags `--version` y `--help` antes de inicializar el linter
- Uso de `env!("CARGO_PKG_VERSION")` para obtener versión del Cargo.toml
- Función `print_help()` centralizada para mantener ayuda consistente

## [0.8.0] - 2026-01-31

### Agregado
- **Configuración Asistida por IA**: Integración con Claude (Anthropic API) para sugerencias arquitectónicas inteligentes
  - Módulo `ai.rs` con función `sugerir_arquitectura_inicial()`
  - Análisis automático del contexto del proyecto (framework, dependencias, estructura)
  - Sugerencias de patrón arquitectónico basadas en el análisis
  - Recomendaciones de reglas `forbidden_imports` específicas para el proyecto
- **Discovery Inteligente**: Nuevo módulo `discovery.rs` que:
  - Escanea la estructura del proyecto automáticamente
  - Extrae dependencias de `package.json`
  - Identifica archivos arquitectónicos clave (controllers, services, entities, etc.)
  - Genera snapshot del proyecto para análisis de IA
- **Scripts de Instalación Automatizada**:
  - `install.sh` para Linux/macOS con instalación en `/usr/local/bin`
  - `install.ps1` para Windows con instalación en `%USERPROFILE%\bin`
  - Detección automática de PATH y configuración
- **Módulo UI**: Nueva separación de la lógica de interfaz de usuario
  - Función `get_interactive_path()` para selección de proyectos
  - Wizard `ask_user_to_confirm_rules()` para confirmación de sugerencias de IA
- **FAQ Completa**: Sección de preguntas frecuentes en el README
- **Documentación del Flujo Completo**: Descripción detallada del flujo de trabajo desde el primer commit

### Mejorado
- Reorganización de `main.rs` con separación clara de responsabilidades:
  - Uso de `discovery::collect_files()` para recolección de archivos
  - Delegación a módulos `ui`, `ai`, `config` para mejor mantenibilidad
- `save_config_from_wizard()` ahora acepta parámetro `max_lines` personalizable
- Función `detect_framework()` ahora acepta `&Path` en lugar de `&PathBuf` (más flexible)
- Enum `Framework` con método `as_str()` para conversión a String

### Corregido
- Error de tipos en `discovery.rs`: conversión correcta de `Framework` enum a `String`
- Errores de compilación relacionados con tipos incompatibles `&Path` vs `&PathBuf`
- Falta de dependencia `anyhow` en `Cargo.toml`

### Técnico
- Nueva dependencia: `anyhow = "1.0"` para manejo de errores
- Integración con API de Anthropic usando `reqwest` y `tokio`
- Función `consultar_claude()` con soporte para:
  - Variables de entorno `ANTHROPIC_AUTH_TOKEN` y `ANTHROPIC_BASE_URL`
  - Modelo Claude Opus 4.5
  - Parseo robusto de respuestas JSON de la IA
- Estructura `ProjectContext` para snapshot del proyecto
- Estructura `AISuggestionResponse` para mapeo de respuestas de IA
- Función `collect_files()` movida a módulo `discovery` con filtrado de `.d.ts`

## [0.7.0] - 2026-01-30

### Agregado
- **Motor de Reglas Dinámicas**: Sistema completamente funcional de `forbidden_imports` con formato `from` → `to`
- **Detección Automática de Framework**: Nuevo módulo `detector.rs` que reconoce NestJS, React, Angular, Express
- **Configuración Interactiva**: Modo guiado en primera ejecución que:
  - Detecta el framework del proyecto
  - Sugiere patrón arquitectónico (Hexagonal, Clean, MVC)
  - Propone límite de líneas basado en el framework
  - Genera `architect.json` automáticamente
- **Soporte para Patrones Arquitectónicos**:
  - Hexagonal
  - Clean Architecture
  - MVC
  - Ninguno (sin patrón específico)
- Documentación completa del motor de reglas dinámicas con ejemplos por patrón
- Tabla comparativa de restricciones por arquitectura
- Sugerencias LOC específicas por framework

### Corregido
- **Error de compilación**: Campo faltante `forbidden_imports` en `LinterContext` (líneas 97 y 162 de main.rs)
- Eliminada función duplicada `load_config` no utilizada
- Todas las advertencias del compilador (warnings) eliminadas
- Formato de `architect.json` corregido en documentación (`from`/`to` en lugar de `file_pattern`/`prohibited`)

### Mejorado
- Función `setup_or_load_config` ahora maneja ambos modos: automático (con archivo existente) y configuración interactiva
- Carga dinámica de `forbidden_imports` desde JSON
- Validación de reglas más robusta con conversión a minúsculas
- Documentación completamente actualizada y sin duplicaciones

### Técnico
- Módulo `detector.rs` con funciones `detect_framework()` y `get_loc_suggestion()`
- Estructura `ForbiddenRule` con campos `from` y `to`
- `LinterContext` ahora incluye `forbidden_imports: Vec<ForbiddenRule>`
- Deserialización mejorada del JSON con manejo de arrays

## [0.6.0] - 2026-01-30

### Refactorizado
- Separación del código en módulos para mejor organización y mantenibilidad:
  - **src/analyzer.rs**: Lógica de análisis de archivos TypeScript movida a módulo dedicado
  - **src/config.rs**: Definiciones de configuración y tipos de error (`LinterConfig`, `ArchError`)
  - **src/main.rs**: Simplificado, enfocado en orquestación y flujo principal
- Mejora en la estructura del proyecto siguiendo mejores prácticas de Rust

### Agregado
- Dependencias para soporte asíncrono futuro:
  - `tokio` v1.0 con features completas para operaciones async
  - `reqwest` v0.11 con soporte JSON para cliente HTTP
  - `async-trait` v0.1 para traits asíncronos
- Preparación de infraestructura para futuras funcionalidades de red y procesamiento async

### Técnico
- Modularización del código base para facilitar testing y extensibilidad
- Configuración centralizada en módulo `config` con `LinterConfig` y `ArchError`
- Función `analyze_file` ahora exportada desde módulo `analyzer`

## [0.5.0] - 2026-01-29

### Agregado
- Documentación completa del proyecto en README.md
- Guía rápida de instalación y configuración para proyectos NestJS
- Especificación del archivo de configuración `architect.json`
- Archivo de ejemplo `architect.json.example` con múltiples reglas recomendadas
- Archivo CHANGELOG.md para seguimiento de versiones
- Metadatos adicionales en Cargo.toml (authors, description, license, etc.)
- Documentación de integración con Git Hooks usando Husky
- Guía detallada NESTJS_INTEGRATION.md con:
  - Instrucciones paso a paso para configurar pre-commit hooks
  - Reglas recomendadas específicas para arquitectura NestJS
  - Solución de problemas comunes
  - Configuración avanzada con lint-staged
  - Buenas prácticas de uso
- Archivo pre-commit.example como plantilla para hooks de Husky
- Soporte documentado para argumentos CLI (`--path`) para integración con herramientas externas

### Documentado
- Estructura requerida del archivo `architect.json` en la raíz del proyecto a validar
- Propiedad `max_lines_per_function` para configurar el límite de líneas por función
- Propiedad `forbidden_imports` para definir reglas de importaciones prohibidas con:
  - `file_pattern`: Patrón del archivo fuente
  - `prohibited`: Patrón del módulo prohibido
  - `reason`: (Opcional) Razón de la restricción
- Ejemplos de configuración y uso
- Estructura del proyecto y dependencias
- Reglas de arquitectura implementadas

### Planificado
- Implementación de lectura y parseo del archivo `architect.json`
- Aplicación dinámica de reglas configurables
- Validación de esquema del archivo de configuración

## [0.1.0] - 2026-01-XX

### Agregado
- Versión inicial del proyecto
- Análisis de archivos TypeScript (.ts)
- Validación de importaciones prohibidas (hardcoded)
  - Regla: archivos `.controller.ts` no pueden importar `.repository`
- Detección de funciones que exceden 200 líneas
- Procesamiento paralelo con Rayon para análisis rápido
- Interfaz interactiva para selección de proyectos con Dialoguer
- Reportes visuales de errores con Miette
- Barra de progreso con Indicatif
- Exclusión automática de directorios: node_modules, dist, .git, target
- Parser TypeScript usando SWC

### Técnico
- Uso de swc_ecma_parser para análisis de código TypeScript
- Implementación de error personalizado `ArchError` con soporte Diagnostic
- SourceMap para ubicación precisa de errores
- Filtrado inteligente de directorios durante el walkdir

[3.1.0]: https://github.com/sergiogswv/architect-linter/releases/tag/v3.1.0
[2.0.0]: https://github.com/sergiogswv/architect-linter/releases/tag/v2.0.0
[1.0.0]: https://github.com/sergiogswv/architect-linter/releases/tag/v1.0.0
[0.8.0]: https://github.com/sergiogswv/architect-linter/releases/tag/v0.8.0
[0.7.0]: https://github.com/sergiogswv/architect-linter/releases/tag/v0.7.0
[0.6.0]: https://github.com/sergiogswv/architect-linter/releases/tag/v0.6.0
[0.5.0]: https://github.com/sergiogswv/architect-linter/releases/tag/v0.5.0
[0.1.0]: https://github.com/sergiogswv/architect-linter/releases/tag/v0.1.0
