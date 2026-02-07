use crate::config::{AIConfig, ForbiddenRule};
use miette::{IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Representa una violación arquitectónica detectada
#[derive(Debug, Clone)]
pub struct Violation {
    /// Ruta del archivo con la violación
    pub file_path: PathBuf,
    /// Contenido completo del archivo
    pub file_content: String,
    /// Import ofensivo que causa la violación
    pub offensive_import: String,
    /// Regla que fue violada
    pub rule: ForbiddenRule,
    /// Línea donde ocurre la violación
    pub line_number: usize,
}

/// Tipo de fix sugerido por la IA
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "fix_type", rename_all = "snake_case")]
pub enum FixType {
    /// Refactorizar código (cambiar imports, crear interfaces)
    Refactor { old_code: String, new_code: String },
    /// Mover archivo a otra capa
    MoveFile { from: String, to: String },
    /// Crear nueva interfaz/abstracción
    CreateInterface {
        interface_path: String,
        interface_code: String,
        updated_import: String,
    },
}

/// Respuesta estructurada de la IA
#[derive(Debug, Deserialize, Serialize)]
pub struct FixSuggestion {
    pub fix_type: FixType,
    pub explanation: String,
    pub confidence: String, // "high", "medium", "low"
}

/// Consulta a la IA para obtener una sugerencia de fix
pub async fn suggest_fix(
    violation: &Violation,
    project_root: &Path,
    ai_configs: &[AIConfig],
) -> Result<FixSuggestion> {
    // Obtener estructura de carpetas del proyecto
    let folder_structure = get_project_structure(project_root);

    // Construir prompt estructurado
    let prompt = format!(
        r#"Eres un arquitecto de software experto. Analiza esta violación arquitectónica y sugiere una refactorización.

**CONTEXTO DEL PROYECTO:**
Estructura de carpetas:
{}

**VIOLACIÓN DETECTADA:**
Archivo: {}
Regla violada: Archivos en '{}' no pueden importar de '{}'
Import ofensivo (línea {}): {}

**CÓDIGO DEL ARCHIVO:**
```typescript
{}
```

**TAREA:**
Sugiere la mejor refactorización para resolver esta violación. Responde ÚNICAMENTE con un objeto JSON válido usando uno de estos formatos:

1. Para refactorizar código:
{{
  "fix_type": "refactor",
  "old_code": "import {{ X }} from '../infrastructure/...'",
  "new_code": "import {{ X }} from '../domain/interfaces/...'",
  "explanation": "Crear una interfaz en la capa de dominio...",
  "confidence": "high"
}}

2. Para mover archivo:
{{
  "fix_type": "move_file",
  "from": "src/domain/user.entity.ts",
  "to": "src/infrastructure/models/user.entity.ts",
  "explanation": "Este archivo pertenece a la capa de infraestructura...",
  "confidence": "medium"
}}

3. Para crear interfaz:
{{
  "fix_type": "create_interface",
  "interface_path": "src/domain/interfaces/IUserRepository.ts",
  "interface_code": "export interface IUserRepository {{ ... }}",
  "updated_import": "import {{ IUserRepository }} from './interfaces/IUserRepository'",
  "explanation": "Crear una abstracción para desacoplar...",
  "confidence": "high"
}}

Responde SOLO con el JSON, sin texto adicional."#,
        folder_structure,
        violation.file_path.display(),
        violation.rule.from,
        violation.rule.to,
        violation.line_number,
        violation.offensive_import,
        violation.file_content
    );

    // Hacer la petición a la IA usando el sistema de fallback
    let content = crate::ai::consultar_ia_con_fallback(prompt, ai_configs)
        .map_err(|e| miette::miette!("No se pudo obtener sugerencia de ningún modelo: {}", e))?;

    // Limpiar markdown code blocks si existen
    let json_content = content
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    // Buscar el primer '{' y el último '}' para casos donde la IA añade texto
    let json_start = json_content.find('{').ok_or_else(|| {
        miette::miette!("No se encontró JSON en la respuesta de la IA: {}", content)
    })?;
    let json_end = json_content.rfind('}').unwrap_or(json_content.len() - 1) + 1;
    let clean_json = &json_content[json_start..json_end];

    // Parsear la respuesta JSON
    let suggestion: FixSuggestion =
        serde_json::from_str(clean_json)
            .into_diagnostic()
            .map_err(|e| {
                miette::miette!(
                    "Error parseando respuesta de IA: {}. Contenido: {}",
                    e,
                    clean_json
                )
            })?;

    Ok(suggestion)
}

/// Aplica un fix sugerido
pub fn apply_fix(
    suggestion: &FixSuggestion,
    violation: &Violation,
    project_root: &Path,
) -> Result<String> {
    match &suggestion.fix_type {
        FixType::Refactor { old_code, new_code } => apply_refactor(violation, old_code, new_code),
        FixType::MoveFile { from, to } => apply_move_file(project_root, from, to),
        FixType::CreateInterface {
            interface_path,
            interface_code,
            updated_import,
        } => apply_create_interface(
            project_root,
            violation,
            interface_path,
            interface_code,
            updated_import,
        ),
    }
}

/// Aplica una refactorización de código
fn apply_refactor(violation: &Violation, old_code: &str, new_code: &str) -> Result<String> {
    let content = fs::read_to_string(&violation.file_path).into_diagnostic()?;

    // Reemplazar el código antiguo por el nuevo
    let updated_content = content.replace(old_code.trim(), new_code.trim());

    if content == updated_content {
        return Err(miette::miette!(
            "No se pudo aplicar el fix: el código antiguo no se encontró exactamente"
        ));
    }

    // Escribir el archivo actualizado
    fs::write(&violation.file_path, &updated_content).into_diagnostic()?;

    Ok(format!(
        "✅ Refactorizado: {}",
        violation.file_path.display()
    ))
}

/// Aplica el movimiento de un archivo
fn apply_move_file(project_root: &Path, from: &str, to: &str) -> Result<String> {
    let from_path = project_root.join(from);
    let to_path = project_root.join(to);

    // Crear el directorio destino si no existe
    if let Some(parent) = to_path.parent() {
        fs::create_dir_all(parent).into_diagnostic()?;
    }

    // Mover el archivo
    fs::rename(&from_path, &to_path).into_diagnostic()?;

    Ok(format!("✅ Archivo movido: {} → {}", from, to))
}

/// Crea una nueva interfaz y actualiza el import
fn apply_create_interface(
    project_root: &Path,
    violation: &Violation,
    interface_path: &str,
    interface_code: &str,
    updated_import: &str,
) -> Result<String> {
    let interface_full_path = project_root.join(interface_path);

    // Crear el directorio si no existe
    if let Some(parent) = interface_full_path.parent() {
        fs::create_dir_all(parent).into_diagnostic()?;
    }

    // Crear el archivo de interfaz
    fs::write(&interface_full_path, interface_code).into_diagnostic()?;

    // Actualizar el import en el archivo original
    let content = fs::read_to_string(&violation.file_path).into_diagnostic()?;
    let updated_content = content.replace(&violation.offensive_import, updated_import);
    fs::write(&violation.file_path, updated_content).into_diagnostic()?;

    Ok(format!(
        "✅ Interfaz creada: {} y import actualizado en {}",
        interface_path,
        violation.file_path.display()
    ))
}

/// Obtiene la estructura de carpetas del proyecto
fn get_project_structure(root: &Path) -> String {
    let mut structure = String::new();

    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                let name = entry.file_name().to_string_lossy().to_string();

                // Ignorar node_modules, .git, etc.
                if name.starts_with('.') || name == "node_modules" {
                    continue;
                }

                if metadata.is_dir() {
                    structure.push_str(&format!("  📁 {}/\n", name));

                    // Listar subdirectorios (máximo 2 niveles)
                    if let Ok(sub_entries) = fs::read_dir(entry.path()) {
                        for sub_entry in sub_entries.flatten().take(5) {
                            let sub_name = sub_entry.file_name().to_string_lossy().to_string();
                            if !sub_name.starts_with('.') {
                                structure.push_str(&format!("    - {}\n", sub_name));
                            }
                        }
                    }
                }
            }
        }
    }

    structure
}
