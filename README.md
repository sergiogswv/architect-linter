# Architect Linter

**Versión:** 0.7.0

Un linter de arquitectura de software escrito en Rust que valida reglas arquitectónicas en proyectos TypeScript mediante un motor de reglas dinámicas. Asegura que el diseño del software (Hexagonal, Clean, MVC, etc.) se respete sin importar quién escriba el código.

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

### 1. Compilar el Linter
```bash
git clone https://github.com/sergio/architect-linter.git
cd architect-linter
cargo build --release
```

### 2. Ejecutar en tu Proyecto
```bash
# Primera ejecución: Modo interactivo de configuración
./target/release/architect-linter

# O especificar ruta directamente
./target/release/architect-linter /ruta/a/tu/proyecto
```

La primera vez que ejecutes el linter en un proyecto, detectará automáticamente el framework y te guiará para crear el archivo `architect.json` con reglas recomendadas.

### 3. Integración con Git Hooks (Opcional)
```bash
# En tu proyecto
npx husky-init && npm install

# Editar .husky/pre-commit
echo '#!/bin/sh
. "$(dirname "$0")/_/husky.sh"
echo "🏗️  Ejecutando Architect Linter..."
"/ruta/architect-linter/target/release/architect-linter" --path .
' > .husky/pre-commit
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

- **Sin argumentos**: Modo interactivo, muestra menú de proyectos disponibles
- **Con ruta**: `./architect-linter /ruta/proyecto` - Analiza el proyecto especificado

## Integración con Git Hooks

📖 **Guía completa**: [NESTJS_INTEGRATION.md](NESTJS_INTEGRATION.md)

```bash
# En tu proyecto
npx husky-init && npm install

# Editar .husky/pre-commit
echo '#!/bin/sh
. "$(dirname "$0")/_/husky.sh"
"/ruta/architect-linter/target/release/architect-linter" --path .
' > .husky/pre-commit

chmod +x .husky/pre-commit
```

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

### Próximamente 🚧
- [ ] Soporte para JavaScript (.js, .jsx)
- [ ] Validación de esquema JSON con mensajes de error claros
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

Sergio - [GitHub](https://github.com/sergio)

## Changelog

Ver [CHANGELOG.md](CHANGELOG.md) para el historial completo de versiones.

### v0.7.0 (2026-01-30) - Motor de Reglas Dinámicas
- ✨ Motor de reglas dinámicas completamente funcional
- 🔍 Detección automática de framework con módulo `detector.rs`
- 🎯 Configuración interactiva en primera ejecución
- 📐 Soporte para patrones arquitectónicos: Hexagonal, Clean, MVC
- 🛠️ Corrección de errores de compilación y warnings
- 📚 Documentación actualizada con ejemplos por patrón
