# Architect Linter

**Versión:** 0.5.0

Un linter de arquitectura de software escrito en Rust que valida reglas arquitectónicas en proyectos TypeScript, ayudando a mantener la separación de responsabilidades y las mejores prácticas de diseño.

## Características

- **Validación de Importaciones Prohibidas**: Detecta y reporta importaciones que violan las reglas de arquitectura definidas
- **Control de Complejidad**: Valida que las funciones no excedan un límite máximo de líneas
- **Procesamiento Paralelo**: Análisis rápido utilizando procesamiento multi-hilo con Rayon
- **Reportes Visuales**: Errores detallados y coloridos utilizando Miette para fácil identificación de problemas
- **Interfaz Interactiva**: Selección de proyectos mediante menú interactivo
- **Integración con Git Hooks**: Compatible con Husky para validación pre-commit automática

## Guía Rápida para Proyectos NestJS

### 1. Instalar el Linter
```bash
# Clonar el repositorio del linter
git clone https://github.com/sergio/architect-linter.git
cd architect-linter

# Compilar el proyecto
cargo build --release
```

### 2. Configurar tu Proyecto NestJS
```bash
# En la raíz de tu proyecto NestJS
cd /ruta/a/tu/proyecto-nestjs

# Crear archivo de configuración
cat > architect.json << 'EOF'
{
  "max_lines_per_function": 40,
  "forbidden_imports": [
    {
      "file_pattern": ".controller.ts",
      "prohibited": ".repository"
    }
  ]
}
EOF

# Instalar Husky
npx husky-init && npm install
```

### 3. Configurar el Hook pre-commit
```bash
# Editar .husky/pre-commit con la ruta a tu linter
echo '#!/bin/sh
. "$(dirname "$0")/_/husky.sh"

echo "🏗️  Ejecutando Architect Linter..."
"C:/Ruta/A/architect-linter/target/release/architect-linter.exe" --path .

if [ $? -ne 0 ]; then
  echo "❌ El commit fue cancelado debido a violaciones de arquitectura"
  exit 1
fi' > .husky/pre-commit

# Dar permisos (Linux/Mac)
chmod +x .husky/pre-commit
```

### 4. Probar
```bash
# Hacer un commit para probar el linter
git add .
git commit -m "test: verificar architect-linter"
```

## Requisitos

- Rust 1.70 o superior
- Proyecto TypeScript a analizar

## Instalación

```bash
cargo build --release
```

El ejecutable se generará en `target/release/architect-linter`

## Configuración

### Archivo architect.json

Para que el linter funcione correctamente, **debe existir un archivo `architect.json` en la raíz del proyecto que se va a validar** con la siguiente estructura:

```json
{
  "max_lines_per_function": 40,
  "forbidden_imports": [
    {
      "file_pattern": ".controller.ts",
      "prohibited": ".repository"
    }
  ]
}
```

#### Propiedades de Configuración

##### `max_lines_per_function`
- **Tipo**: `number`
- **Descripción**: Número máximo de líneas permitidas por función
- **Ejemplo**: `40` - Las funciones no deben exceder 40 líneas

##### `forbidden_imports`
- **Tipo**: `array` de objetos
- **Descripción**: Lista de reglas que definen qué archivos no pueden importar ciertos módulos

Cada regla contiene:
- `file_pattern`: Patrón que identifica el tipo de archivo (ej. `.controller.ts`)
- `prohibited`: Patrón de módulo prohibido para ese tipo de archivo (ej. `.repository`)

#### Ejemplo de Configuración Completa

```json
{
  "max_lines_per_function": 40,
  "forbidden_imports": [
    {
      "file_pattern": ".controller.ts",
      "prohibited": ".repository",
      "reason": "Los controladores deben usar servicios, no repositorios directamente"
    },
    {
      "file_pattern": ".service.ts",
      "prohibited": ".controller",
      "reason": "Los servicios no deben depender de controladores"
    },
    {
      "file_pattern": ".component.tsx",
      "prohibited": ".repository",
      "reason": "Los componentes no deben acceder a la capa de datos directamente"
    }
  ]
}
```

## Uso

1. Ejecuta el linter:

```bash
./target/release/architect-linter
```

o durante desarrollo:

```bash
cargo run
```

2. Selecciona el proyecto a analizar del menú interactivo, o ingresa la ruta manualmente

3. El linter escaneará todos los archivos `.ts` del proyecto y reportará:
   - Importaciones que violan las reglas de arquitectura definidas en `architect.json`
   - Funciones que exceden el límite de líneas configurado

### Uso con Argumentos CLI

El linter también acepta argumentos de línea de comandos:

```bash
./target/release/architect-linter --path /ruta/al/proyecto
```

Opciones disponibles:
- `--path <RUTA>`: Especifica la ruta del proyecto a analizar (evita el menú interactivo)

## Integración con Git Hooks (Husky)

Para ejecutar automáticamente el linter antes de cada commit en tu proyecto NestJS, puedes integrarlo con Husky.

📖 **Para instrucciones detalladas y solución de problemas, consulta: [NESTJS_INTEGRATION.md](NESTJS_INTEGRATION.md)**

### Resumen de Integración

### 1. Instalar Husky en tu proyecto NestJS

En la raíz de tu proyecto NestJS, ejecuta:

```bash
npx husky-init && npm install
```

Esto creará la carpeta `.husky` con la configuración inicial.

### 2. Configurar el Hook pre-commit

Abre el archivo `.husky/pre-commit` que se creó en tu proyecto NestJS y cámbialo para que llame a tu ejecutable de Rust.

Puedes usar el archivo `pre-commit.example` incluido en este repositorio como plantilla:

```bash
#!/bin/sh
. "$(dirname "$0")/_/husky.sh"

echo "🏗️  Ejecutando Architect Linter..."
# Cambia esta ruta a donde tengas el .exe de tu linter
"C:/Ruta/A/Tu/Proyecto/Rust/target/release/architect-linter.exe" --path .

# Si el linter encuentra errores, el commit se cancelará
if [ $? -ne 0 ]; then
  echo "❌ El commit fue cancelado debido a violaciones de arquitectura"
  exit 1
fi
```

### 3. Dar permisos de ejecución (Linux/Mac)

```bash
chmod +x .husky/pre-commit
```

### 4. Probar la integración

Intenta hacer un commit en tu proyecto NestJS. El linter se ejecutará automáticamente y:
- ✅ Si no hay violaciones, el commit continuará normalmente
- ❌ Si hay violaciones, el commit será cancelado y verás los errores

### Ejemplo de flujo con Husky

```bash
git add .
git commit -m "feat: add new user endpoint"

# Salida:
🏗️  Ejecutando Architect Linter...
🏛️  WELCOME TO ARCHITECT-LINTER
🚀 Analizando 145 archivos en "my-nestjs-project"...

📌 Archivo: src/controllers/user.controller.ts
  × Violación de Arquitectura: Importación Prohibida
  ...

❌ El commit fue cancelado debido a violaciones de arquitectura
```

## Ejemplo de Salida

```
🏛️  WELCOME TO ARCHITECT-LINTER
? Selecciona el proyecto a auditar › my-backend-project
🚀 Analizando 145 archivos en "my-backend-project"...

📌 Archivo: src/controllers/user.controller.ts
  × Violación de Arquitectura: Importación Prohibida
   ╭─[src/controllers/user.controller.ts:3:1]
   │
 3 │ import { UserRepository } from '../repositories/user.repository'
   │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   │ Este import de repositorio no está permitido aquí
   ╰────
  help: Los controladores (Controllers) deben usar Servicios, nunca Repositorios directamente.

⚠️  [COMPLEJIDAD] Función 'processUserData' es muy larga: 52 líneas (Máximo: 40)

✓ Análisis completado
```

## Estructura del Proyecto

```
architect-linter/
├── src/
│   └── main.rs                 # Código principal del linter
├── Cargo.toml                  # Configuración de dependencias
├── Cargo.lock                  # Lock de versiones
├── README.md                   # Documentación principal
├── CHANGELOG.md                # Registro de cambios
├── NESTJS_INTEGRATION.md       # Guía detallada de integración con NestJS
├── architect.json.example      # Ejemplo de configuración
└── pre-commit.example          # Ejemplo de hook para Husky
```

## Dependencias Principales

- **swc_ecma_parser**: Parser de TypeScript/JavaScript
- **rayon**: Procesamiento paralelo
- **miette**: Reportes de error elegantes
- **walkdir**: Traversal de directorios
- **dialoguer**: Interfaz interactiva de usuario
- **indicatif**: Barras de progreso

## Reglas de Arquitectura Implementadas

### 1. Separación de Capas
Los archivos `.controller.ts` no deben importar directamente archivos `.repository`. Deben usar la capa de servicios como intermediario.

**Incorrecto:**
```typescript
// user.controller.ts
import { UserRepository } from '../repositories/user.repository';
```

**Correcto:**
```typescript
// user.controller.ts
import { UserService } from '../services/user.service';
```

### 2. Complejidad de Funciones
Las funciones no deben exceder el límite configurado en `max_lines_per_function` para mantener la legibilidad y facilitar el mantenimiento.

## Roadmap

### Completado ✅
- [x] Documentación completa del proyecto
- [x] Integración con Git Hooks (Husky)
- [x] Soporte para argumentos CLI (--path)
- [x] Procesamiento paralelo para análisis rápido

### En Progreso 🚧
- [ ] Implementación de lectura del archivo `architect.json`
- [ ] Aplicación dinámica de reglas configurables
- [ ] Validación de esquema del archivo de configuración

### Futuro 🔮
- [ ] Más reglas de arquitectura predefinidas
- [ ] Soporte para JavaScript (.js, .jsx)
- [ ] Exportación de reportes en JSON/HTML/Markdown
- [ ] Integración nativa con CI/CD (GitHub Actions, GitLab CI, etc.)
- [ ] Reglas personalizadas mediante plugins
- [ ] Caché de resultados para análisis incremental
- [ ] Modo watch para desarrollo continuo
- [ ] Configuración de severidad por regla (error, warning, info)

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

### v0.5.0 (2026-01-29)
- Documentación completa del proyecto
- Especificación del archivo de configuración `architect.json`
- Soporte para reglas de importaciones prohibidas configurables
- Configuración de límite de líneas por función

### v0.1.0
- Versión inicial
- Validación de importaciones prohibidas (hardcoded)
- Detección de funciones largas
- Procesamiento paralelo
- Interfaz interactiva
