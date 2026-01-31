# Changelog

Todos los cambios notables en este proyecto serán documentados en este archivo.

El formato está basado en [Keep a Changelog](https://keepachangelog.com/es-ES/1.0.0/),
y este proyecto adhiere a [Versionado Semántico](https://semver.org/lang/es/).

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

[0.7.0]: https://github.com/sergio/architect-linter/releases/tag/v0.7.0
[0.6.0]: https://github.com/sergio/architect-linter/releases/tag/v0.6.0
[0.5.0]: https://github.com/sergio/architect-linter/releases/tag/v0.5.0
[0.1.0]: https://github.com/sergio/architect-linter/releases/tag/v0.1.0
