# Changelog

Todos los cambios notables en este proyecto serán documentados en este archivo.

El formato está basado en [Keep a Changelog](https://keepachangelog.com/es-ES/1.0.0/),
y este proyecto adhiere a [Versionado Semántico](https://semver.org/lang/es/).

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

[1.0.0]: https://github.com/sergiogswv/architect-linter/releases/tag/v1.0.0
[0.8.0]: https://github.com/sergiogswv/architect-linter/releases/tag/v0.8.0
[0.7.0]: https://github.com/sergiogswv/architect-linter/releases/tag/v0.7.0
[0.6.0]: https://github.com/sergiogswv/architect-linter/releases/tag/v0.6.0
[0.5.0]: https://github.com/sergiogswv/architect-linter/releases/tag/v0.5.0
[0.1.0]: https://github.com/sergiogswv/architect-linter/releases/tag/v0.1.0
