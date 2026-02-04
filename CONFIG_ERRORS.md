# Configuration Errors Guide

This guide documents the most common errors when configuring `architect.json` and how to resolve them.

## Table of Contents

- [Basic Structure](#basic-structure)
- [Common Errors](#common-errors)
- [Validations](#validations)

## Basic Structure

A valid `architect.json` file must have this structure:

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

## Common Errors

### 1. Invalid JSON Syntax

**❌ Error:**
```
× JSON inválido: expected `,` or `}` at line 4 column 3
```

**Cause:** Missing comma, brace, or extra character.

**❌ Incorrect example:**
```json
{
  "max_lines_per_function": 50,
  "architecture_pattern": "MVC"  // ← Missing comma here
  "forbidden_imports": []
}
```

**✅ Solution:**
```json
{
  "max_lines_per_function": 50,
  "architecture_pattern": "MVC",  // ← Comma added
  "forbidden_imports": []
}
```

**💡 Tip:** Use an online JSON validator like [jsonlint.com](https://jsonlint.com/) to verify syntax.

---

### 2. Missing Field: max_lines_per_function

**❌ Error:**
```
× Falta el campo requerido: max_lines_per_function
help: Agrega este campo con un número, ejemplo: "max_lines_per_function": 40
```

**✅ Solution:**
```json
{
  "max_lines_per_function": 40,  // ← Field added
  "architecture_pattern": "MVC",
  "forbidden_imports": []
}
```

**Recommended values:**
- React: 20-30 (small components)
- NestJS: 30-50 (class methods)
- Angular: 40-60 (complex components)
- Express: 50-80 (handlers and middleware)

---

### 3. Incorrect Data Type in max_lines_per_function

**❌ Error:**
```
× El campo 'max_lines_per_function' debe ser un número
help: Ejemplo correcto: "max_lines_per_function": 40
```

**❌ Incorrect example:**
```json
{
  "max_lines_per_function": "50",  // ← String instead of number
  ...
}
```

**✅ Solution:**
```json
{
  "max_lines_per_function": 50,  // ← Number without quotes
  ...
}
```

---

### 4. Zero Value in max_lines_per_function

**❌ Error:**
```
× max_lines_per_function no puede ser 0
help: Usa un valor entre 10 y 500. Recomendado: 20-60 según tu framework.
```

**✅ Solution:** Use a value greater than 0. If you want to disable this validation, use a very high value (500+).

---

### 5. Invalid Architecture Pattern

**❌ Error:**
```
× Patrón arquitectónico inválido: 'layered'
help: Valores válidos: Hexagonal, Clean, MVC, Ninguno
```

**❌ Incorrect example:**
```json
{
  "architecture_pattern": "layered",  // ← Not a valid value
  ...
}
```

**✅ Solution:**
```json
{
  "architecture_pattern": "MVC",  // ← Use one of the valid values
  ...
}
```

**Valid values:**
- `"Hexagonal"` - For hexagonal architecture/ports and adapters
- `"Clean"` - For Clean Architecture
- `"MVC"` - For Model-View-Controller
- `"Ninguno"` - No specific pattern

**⚠️ Note:** Values are case-sensitive.

---

### 6. Missing Field: architecture_pattern

**❌ Error:**
```
× Falta el campo requerido: architecture_pattern
help: Agrega este campo. Valores válidos: "Hexagonal", "Clean", "MVC", "Ninguno"
```

**✅ Solution:**
```json
{
  "max_lines_per_function": 50,
  "architecture_pattern": "MVC",  // ← Field added
  "forbidden_imports": []
}
```

---

### 7. forbidden_imports is not an Array

**❌ Error:**
```
× El campo 'forbidden_imports' debe ser un array
help: Ejemplo: "forbidden_imports": [{"from": "src/components/**", "to": "src/services/**"}]
```

**❌ Incorrect example:**
```json
{
  "forbidden_imports": {  // ← Object instead of array
    "from": "src/components/**",
    "to": "src/services/**"
  }
}
```

**✅ Solution:**
```json
{
  "forbidden_imports": [  // ← Array with brackets []
    {
      "from": "src/components/**",
      "to": "src/services/**"
    }
  ]
}
```

---

### 8. Rule Missing 'from' or 'to' Field

**❌ Error:**
```
× La regla #1 no tiene el campo 'to'
help: Ejemplo: {"from": "src/components/**", "to": "src/services/**"}
```

**❌ Incorrect example:**
```json
{
  "forbidden_imports": [
    {
      "from": "src/components/**"
      // ← Missing "to" field
    }
  ]
}
```

**✅ Solution:**
```json
{
  "forbidden_imports": [
    {
      "from": "src/components/**",
      "to": "src/services/**"  // ← Field added
    }
  ]
}
```

---

### 9. Duplicate Rules

**❌ Error:**
```
× Regla duplicada: from 'src/components/**' to 'src/services/**'
help: Elimina una de las reglas duplicadas en forbidden_imports.
```

**❌ Incorrect example:**
```json
{
  "forbidden_imports": [
    {
      "from": "src/components/**",
      "to": "src/services/**"
    },
    {
      "from": "src/components/**",  // ← Duplicate
      "to": "src/services/**"       // ← Duplicate
    }
  ]
}
```

**✅ Solution:** Remove one of the duplicate rules.

---

## Validations

The linter automatically validates:

### JSON Structure
- ✅ Valid JSON syntax
- ✅ File is an object (between `{}`)
- ✅ All required fields present

### Required Fields
- ✅ `max_lines_per_function` (number)
- ✅ `architecture_pattern` (string)
- ✅ `forbidden_imports` (array)

### Value Validations
- ✅ `max_lines_per_function` > 0
- ✅ `max_lines_per_function` ≤ 1000
- ✅ `architecture_pattern` is one of: Hexagonal, Clean, MVC, Ninguno
- ✅ Each rule has `from` and `to`
- ✅ No duplicate rules

### Warnings (Non-Blocking)
- ⚠️ If `forbidden_imports` is empty, only function length is validated

## Complete Examples

### Minimal Valid Configuration

```json
{
  "max_lines_per_function": 50,
  "architecture_pattern": "Ninguno",
  "forbidden_imports": []
}
```

### Configuration for React

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

### Configuration for NestJS (Hexagonal)

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

## Additional Help

If you encounter an error not documented here:

1. Read the complete error message - it always includes a solution suggestion
2. Verify JSON syntax with [jsonlint.com](https://jsonlint.com/)
3. Compare your configuration with the examples in this document
4. Check the [README.md](README.md) for more information about architectural patterns

## Report Issues

If you believe you found a bug in the validation:
- Open an issue at: https://github.com/sergiogswv/architect-linter/issues
- Include your `architect.json` file and the complete error message
