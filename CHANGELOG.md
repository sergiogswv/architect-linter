# Changelog

Todos los cambios notables en este proyecto serán documentados en este archivo.

El formato está basado en [Keep a Changelog](https://keepachangelog.com/es-ES/1.0.0/),
y este proyecto adhiere a [Versionado Semántico](https://semver.org/lang/es/).

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

[0.5.0]: https://github.com/sergio/architect-linter/releases/tag/v0.5.0
[0.1.0]: https://github.com/sergio/architect-linter/releases/tag/v0.1.0
