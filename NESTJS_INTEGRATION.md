# Integración con NestJS

Esta guía te ayudará a integrar Architect Linter en tu proyecto NestJS para validar automáticamente las reglas de arquitectura antes de cada commit.

## Requisitos Previos

- Node.js y npm instalados
- Git inicializado en tu proyecto NestJS
- Architect Linter compilado (ver [README.md](README.md))

## Paso 1: Crear archivo de configuración

En la **raíz de tu proyecto NestJS**, crea un archivo llamado `architect.json`:

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
    }
  ]
}
```

### Reglas Recomendadas para NestJS

```json
{
  "max_lines_per_function": 40,
  "forbidden_imports": [
    {
      "file_pattern": ".controller.ts",
      "prohibited": ".repository",
      "reason": "Controladores → Servicios (no Repositorios directamente)"
    },
    {
      "file_pattern": ".controller.ts",
      "prohibited": ".entity",
      "reason": "Controladores no deben importar entidades directamente"
    },
    {
      "file_pattern": ".service.ts",
      "prohibited": ".controller",
      "reason": "Servicios no deben depender de controladores"
    },
    {
      "file_pattern": ".repository.ts",
      "prohibited": ".controller",
      "reason": "Repositorios no deben conocer la capa de controladores"
    },
    {
      "file_pattern": ".repository.ts",
      "prohibited": ".service",
      "reason": "Repositorios no deben depender de servicios"
    }
  ]
}
```

## Paso 2: Instalar y configurar Husky

En la raíz de tu proyecto NestJS, ejecuta:

```bash
npx husky-init && npm install
```

Esto creará:
- Carpeta `.husky/` con archivos de configuración
- Hook `.husky/pre-commit` básico
- Script en `package.json` para preparar Husky

## Paso 3: Configurar el Hook pre-commit

Edita el archivo `.husky/pre-commit` en tu proyecto NestJS:

```bash
#!/bin/sh
. "$(dirname "$0")/_/husky.sh"

echo "🏗️  Ejecutando Architect Linter..."
echo ""

# IMPORTANTE: Cambia esta ruta a donde compilaste el linter
# Para Windows:
LINTER_PATH="C:/Users/TuUsuario/Projects/architect-linter/target/release/architect-linter.exe"

# Para Linux/Mac:
# LINTER_PATH="/home/tuusuario/projects/architect-linter/target/release/architect-linter"

# Ejecutar el linter en el directorio actual (proyecto NestJS)
"$LINTER_PATH" --path .

# Capturar el código de salida
EXIT_CODE=$?

if [ $EXIT_CODE -ne 0 ]; then
  echo ""
  echo "❌ COMMIT CANCELADO: Se encontraron violaciones de arquitectura"
  echo "💡 Corrige los errores reportados arriba y vuelve a intentar el commit"
  echo ""
  exit 1
fi

echo ""
echo "✅ Validación de arquitectura exitosa"
echo ""
exit 0
```

### En Windows (Git Bash)

Si usas Git Bash en Windows, asegúrate de usar rutas con formato Unix:

```bash
"C:/Users/Sergio/Projects/architect-linter/target/release/architect-linter.exe" --path .
```

### En Linux/Mac

```bash
"/home/sergio/projects/architect-linter/target/release/architect-linter" --path .
```

## Paso 4: Dar permisos de ejecución

### En Linux/Mac:

```bash
chmod +x .husky/pre-commit
```

### En Windows:
No es necesario, Git Bash manejará los permisos automáticamente.

## Paso 5: Probar la integración

Intenta hacer un commit para verificar que todo funciona:

```bash
# Hacer algunos cambios
echo "// test" >> src/app.service.ts

# Agregar al staging
git add .

# Intentar commit
git commit -m "test: verificar architect-linter"
```

### Resultados Esperados

#### ✅ Si no hay violaciones:

```
🏗️  Ejecutando Architect Linter...

🏛️  WELCOME TO ARCHITECT-LINTER
🚀 Analizando 45 archivos en "my-nestjs-project"...
✓ Análisis completado

✅ Validación de arquitectura exitosa

[main abc1234] test: verificar architect-linter
 1 file changed, 1 insertion(+)
```

#### ❌ Si hay violaciones:

```
🏗️  Ejecutando Architect Linter...

🏛️  WELCOME TO ARCHITECT-LINTER
🚀 Analizando 45 archivos en "my-nestjs-project"...

📌 Archivo: src/controllers/user.controller.ts
  × Violación de Arquitectura: Importación Prohibida
   ╭─[src/controllers/user.controller.ts:3:1]
   │
 3 │ import { UserRepository } from '../repositories/user.repository'
   │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   │ Este import de repositorio no está permitido aquí
   ╰────
  help: Los controladores (Controllers) deben usar Servicios, nunca Repositorios directamente.

❌ COMMIT CANCELADO: Se encontraron violaciones de arquitectura
💡 Corrige los errores reportados arriba y vuelve a intentar el commit
```

## Desactivar temporalmente el hook

Si necesitas hacer un commit sin ejecutar el linter (no recomendado):

```bash
git commit -m "mensaje" --no-verify
```

## Solución de Problemas

### Error: "command not found: architect-linter"

**Problema:** La ruta al ejecutable es incorrecta.

**Solución:** Verifica que la ruta en `.husky/pre-commit` apunte correctamente al ejecutable compilado:

```bash
# Verifica que el archivo existe
ls -la "C:/Ruta/A/Tu/Proyecto/architect-linter/target/release/architect-linter.exe"
```

### Error: "Permission denied"

**Problema:** El hook no tiene permisos de ejecución (Linux/Mac).

**Solución:**
```bash
chmod +x .husky/pre-commit
```

### El hook no se ejecuta

**Problema:** Husky no está configurado correctamente.

**Solución:** Verifica que existe el script en `package.json`:

```json
{
  "scripts": {
    "prepare": "husky install"
  }
}
```

Ejecuta manualmente:
```bash
npm run prepare
```

### El linter se ejecuta en el directorio incorrecto

**Problema:** El linter busca `architect.json` en el lugar equivocado.

**Solución:** Asegúrate de usar `--path .` para indicar el directorio actual:

```bash
"$LINTER_PATH" --path .
```

## Configuración Avanzada

### Ejecutar solo en archivos modificados (futuro)

```bash
#!/bin/sh
. "$(dirname "$0")/_/husky.sh"

# Obtener archivos .ts modificados
CHANGED_FILES=$(git diff --cached --name-only --diff-filter=ACM | grep '\.ts$')

if [ -z "$CHANGED_FILES" ]; then
  echo "No hay archivos TypeScript modificados"
  exit 0
fi

echo "🏗️  Ejecutando Architect Linter en archivos modificados..."
"$LINTER_PATH" --path . --files $CHANGED_FILES
```

### Integrar con lint-staged

Puedes combinar con lint-staged para mayor control:

```bash
npm install --save-dev lint-staged
```

En `package.json`:
```json
{
  "lint-staged": {
    "*.ts": [
      "C:/Ruta/architect-linter/target/release/architect-linter.exe --path ."
    ]
  }
}
```

En `.husky/pre-commit`:
```bash
#!/bin/sh
. "$(dirname "$0")/_/husky.sh"

npx lint-staged
```

## Buenas Prácticas

1. **Comparte la configuración:** Commitea el archivo `architect.json` para que todo el equipo use las mismas reglas
2. **Documenta excepciones:** Si necesitas usar `--no-verify`, documenta por qué en el mensaje del commit
3. **Actualiza las reglas gradualmente:** Empieza con reglas permisivas y endurecelas progresivamente
4. **Revisa las violaciones regularmente:** No ignores los warnings, son oportunidades de mejora

## Recursos Adicionales

- [Documentación de Husky](https://typicode.github.io/husky/)
- [Documentación de NestJS - Arquitectura](https://docs.nestjs.com/fundamentals/architecture)
- [README principal de Architect Linter](README.md)
