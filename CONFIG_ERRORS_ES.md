# Guía de Errores de Configuración

Esta guía documenta los errores más comunes al configurar `architect.json` y cómo resolverlos.

## Índice

- [Estructura Básica](#estructura-básica)
- [Errores Comunes](#errores-comunes)
- [Validaciones](#validaciones)

## Estructura Básica

Un archivo `architect.json` válido debe tener esta estructura:

```json
{
  "max_lines_per_function": 50,
  "architecture_pattern": "MVC",
  "forbidden_imports": [
    {
      "from": "src/components/**",
      "to": "src/services/**"
    }
  ]
}
```

## Errores Comunes

### 1. JSON con Sintaxis Inválida

**❌ Error:**
```
× JSON inválido: expected `,` or `}` at line 4 column 3
```

**Causa:** Falta una coma, llave, o hay un carácter extra.

**❌ Ejemplo incorrecto:**
```json
{
  "max_lines_per_function": 50,
  "architecture_pattern": "MVC"  // ← Falta coma aquí
  "forbidden_imports": []
}
```

**✅ Solución:**
```json
{
  "max_lines_per_function": 50,
  "architecture_pattern": "MVC",  // ← Coma agregada
  "forbidden_imports": []
}
```

**💡 Consejo:** Usa un validador JSON online como [jsonlint.com](https://jsonlint.com/) para verificar la sintaxis.

---

### 2. Campo Faltante: max_lines_per_function

**❌ Error:**
```
× Falta el campo requerido: max_lines_per_function
help: Agrega este campo con un número, ejemplo: "max_lines_per_function": 40
```

**✅ Solución:**
```json
{
  "max_lines_per_function": 40,  // ← Campo agregado
  "architecture_pattern": "MVC",
  "forbidden_imports": []
}
```

**Valores recomendados:**
- React: 20-30 (componentes pequeños)
- NestJS: 30-50 (métodos de clase)
- Angular: 40-60 (componentes complejos)
- Express: 50-80 (handlers y middleware)

---

### 3. Tipo de Dato Incorrecto en max_lines_per_function

**❌ Error:**
```
× El campo 'max_lines_per_function' debe ser un número
help: Ejemplo correcto: "max_lines_per_function": 40
```

**❌ Ejemplo incorrecto:**
```json
{
  "max_lines_per_function": "50",  // ← String en lugar de número
  ...
}
```

**✅ Solución:**
```json
{
  "max_lines_per_function": 50,  // ← Número sin comillas
  ...
}
```

---

### 4. Valor Cero en max_lines_per_function

**❌ Error:**
```
× max_lines_per_function no puede ser 0
help: Usa un valor entre 10 y 500. Recomendado: 20-60 según tu framework.
```

**✅ Solución:** Usa un valor mayor a 0. Si quieres desactivar esta validación, usa un valor muy alto (500+).

---

### 5. Patrón Arquitectónico Inválido

**❌ Error:**
```
× Patrón arquitectónico inválido: 'layered'
help: Valores válidos: Hexagonal, Clean, MVC, Ninguno
```

**❌ Ejemplo incorrecto:**
```json
{
  "architecture_pattern": "layered",  // ← No es un valor válido
  ...
}
```

**✅ Solución:**
```json
{
  "architecture_pattern": "MVC",  // ← Usar uno de los valores válidos
  ...
}
```

**Valores válidos:**
- `"Hexagonal"` - Para arquitectura hexagonal/puertos y adaptadores
- `"Clean"` - Para Clean Architecture
- `"MVC"` - Para Model-View-Controller
- `"Ninguno"` - Sin patrón específico

**⚠️ Nota:** Los valores distinguen mayúsculas y minúsculas.

---

### 6. Campo Faltante: architecture_pattern

**❌ Error:**
```
× Falta el campo requerido: architecture_pattern
help: Agrega este campo. Valores válidos: "Hexagonal", "Clean", "MVC", "Ninguno"
```

**✅ Solución:**
```json
{
  "max_lines_per_function": 50,
  "architecture_pattern": "MVC",  // ← Campo agregado
  "forbidden_imports": []
}
```

---

### 7. forbidden_imports no es un Array

**❌ Error:**
```
× El campo 'forbidden_imports' debe ser un array
help: Ejemplo: "forbidden_imports": [{"from": "src/components/**", "to": "src/services/**"}]
```

**❌ Ejemplo incorrecto:**
```json
{
  "forbidden_imports": {  // ← Objeto en lugar de array
    "from": "src/components/**",
    "to": "src/services/**"
  }
}
```

**✅ Solución:**
```json
{
  "forbidden_imports": [  // ← Array con corchetes []
    {
      "from": "src/components/**",
      "to": "src/services/**"
    }
  ]
}
```

---

### 8. Regla sin Campo 'from' o 'to'

**❌ Error:**
```
× La regla #1 no tiene el campo 'to'
help: Ejemplo: {"from": "src/components/**", "to": "src/services/**"}
```

**❌ Ejemplo incorrecto:**
```json
{
  "forbidden_imports": [
    {
      "from": "src/components/**"
      // ← Falta el campo "to"
    }
  ]
}
```

**✅ Solución:**
```json
{
  "forbidden_imports": [
    {
      "from": "src/components/**",
      "to": "src/services/**"  // ← Campo agregado
    }
  ]
}
```

---

### 9. Reglas Duplicadas

**❌ Error:**
```
× Regla duplicada: from 'src/components/**' to 'src/services/**'
help: Elimina una de las reglas duplicadas en forbidden_imports.
```

**❌ Ejemplo incorrecto:**
```json
{
  "forbidden_imports": [
    {
      "from": "src/components/**",
      "to": "src/services/**"
    },
    {
      "from": "src/components/**",  // ← Duplicado
      "to": "src/services/**"       // ← Duplicado
    }
  ]
}
```

**✅ Solución:** Elimina una de las reglas duplicadas.

---

## Validaciones

El linter valida automáticamente:

### Estructura del JSON
- ✅ Sintaxis JSON válida
- ✅ Archivo es un objeto (entre `{}`)
- ✅ Todos los campos requeridos presentes

### Campos Requeridos
- ✅ `max_lines_per_function` (número)
- ✅ `architecture_pattern` (string)
- ✅ `forbidden_imports` (array)

### Validaciones de Valores
- ✅ `max_lines_per_function` > 0
- ✅ `max_lines_per_function` ≤ 1000
- ✅ `architecture_pattern` es uno de: Hexagonal, Clean, MVC, Ninguno
- ✅ Cada regla tiene `from` y `to`
- ✅ No hay reglas duplicadas

### Advertencias (No Bloqueantes)
- ⚠️ Si `forbidden_imports` está vacío, solo se valida la longitud de funciones

## Ejemplos Completos

### Configuración Mínima Válida

```json
{
  "max_lines_per_function": 50,
  "architecture_pattern": "Ninguno",
  "forbidden_imports": []
}
```

### Configuración para React

```json
{
  "max_lines_per_function": 30,
  "architecture_pattern": "MVC",
  "forbidden_imports": [
    {
      "from": "src/components/**",
      "to": "src/services/**"
    },
    {
      "from": "src/components/**",
      "to": "src/api/**"
    },
    {
      "from": "src/hooks/**",
      "to": "src/components/**"
    }
  ]
}
```

### Configuración para NestJS (Hexagonal)

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

## Ayuda Adicional

Si encuentras un error no documentado aquí:

1. Lee el mensaje de error completo - siempre incluye una sugerencia de solución
2. Verifica la sintaxis JSON con [jsonlint.com](https://jsonlint.com/)
3. Compara tu configuración con los ejemplos en este documento
4. Revisa el [README.md](README.md) para más información sobre patrones arquitectónicos

## Reportar Problemas

Si crees que encontraste un bug en la validación:
- Abre un issue en: https://github.com/sergiogswv/architect-linter/issues
- Incluye tu archivo `architect.json` y el mensaje de error completo
