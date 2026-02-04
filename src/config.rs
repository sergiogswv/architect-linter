#![allow(unused_assignments)]

use crate::ai::SuggestedRule;
use miette::{Diagnostic, IntoDiagnostic, Result, SourceSpan};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Framework {
    NestJS,
    React,
    Angular,
    Express,
    Unknown,
}

impl Framework {
    pub fn as_str(&self) -> &str {
        match self {
            Framework::NestJS => "NestJS",
            Framework::React => "React",
            Framework::Angular => "Angular",
            Framework::Express => "Express",
            Framework::Unknown => "Unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ArchPattern {
    Hexagonal,
    Clean,
    MVC,
    Ninguno,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForbiddenRule {
    pub from: String,
    pub to: String,
}

/// Estructura para mapear el architect.json tal cual está en el disco
#[derive(Debug, Serialize, Deserialize)]
struct ConfigFile {
    pub max_lines_per_function: usize,
    pub architecture_pattern: ArchPattern,
    pub forbidden_imports: Vec<ForbiddenRule>,
}

pub struct LinterContext {
    pub max_lines: usize,
    #[allow(dead_code)]
    pub framework: Framework,
    #[allow(dead_code)]
    pub pattern: ArchPattern,
    pub forbidden_imports: Vec<ForbiddenRule>,
}

/// CARGA SILENCIOSA: Lee architect.json y lo convierte en contexto
pub fn load_config(root: &Path) -> Result<LinterContext> {
    let config_path = root.join("architect.json");

    // Leer el archivo
    let content = fs::read_to_string(&config_path)
        .map_err(|e| ConfigError::new(
            format!("No se pudo leer architect.json: {}", e),
            format!("Asegúrate de que el archivo existe en: {}", config_path.display())
        ))?;

    // Validar que es JSON válido
    let json_value: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| ConfigError::new(
            format!("JSON inválido: {}", e),
            "Verifica que el archivo architect.json tenga sintaxis JSON válida. Usa un validador JSON online si es necesario.".to_string()
        ))?;

    // Validar el esquema antes de deserializar
    validate_schema(&json_value)?;

    // Ahora sí deserializar con mejor manejo de errores
    let config: ConfigFile = serde_json::from_value(json_value)
        .map_err(|e| ConfigError::new(
            format!("Error en la estructura: {}", e),
            "Revisa que todos los campos tengan el tipo correcto.".to_string()
        ))?;

    // Validar los valores
    validate_config_values(&config)?;

    // Re-detectamos el framework para el contexto actual
    let framework = crate::detector::detect_framework(root);

    Ok(LinterContext {
        max_lines: config.max_lines_per_function,
        framework,
        pattern: config.architecture_pattern,
        forbidden_imports: config.forbidden_imports,
    })
}

/// Valida que el JSON tenga todos los campos requeridos
fn validate_schema(json: &serde_json::Value) -> Result<()> {
    let obj = json.as_object().ok_or_else(|| ConfigError::new(
        "El archivo debe ser un objeto JSON".to_string(),
        "El archivo architect.json debe empezar y terminar con llaves { }".to_string()
    ))?;

    // Validar campo: max_lines_per_function
    if !obj.contains_key("max_lines_per_function") {
        return Err(ConfigError::new(
            "Falta el campo requerido: max_lines_per_function".to_string(),
            "Agrega este campo con un número, ejemplo: \"max_lines_per_function\": 40".to_string()
        ).into());
    }

    if !obj["max_lines_per_function"].is_number() {
        return Err(ConfigError::new(
            "El campo 'max_lines_per_function' debe ser un número".to_string(),
            "Ejemplo correcto: \"max_lines_per_function\": 40".to_string()
        ).into());
    }

    // Validar campo: architecture_pattern
    if !obj.contains_key("architecture_pattern") {
        return Err(ConfigError::new(
            "Falta el campo requerido: architecture_pattern".to_string(),
            "Agrega este campo. Valores válidos: \"Hexagonal\", \"Clean\", \"MVC\", \"Ninguno\"".to_string()
        ).into());
    }

    if !obj["architecture_pattern"].is_string() {
        return Err(ConfigError::new(
            "El campo 'architecture_pattern' debe ser un string".to_string(),
            "Valores válidos: \"Hexagonal\", \"Clean\", \"MVC\", \"Ninguno\"".to_string()
        ).into());
    }

    // Validar que el patrón sea uno de los valores aceptados
    let pattern_str = obj["architecture_pattern"].as_str().unwrap();
    let valid_patterns = ["Hexagonal", "Clean", "MVC", "Ninguno"];
    if !valid_patterns.contains(&pattern_str) {
        return Err(ConfigError::new(
            format!("Patrón arquitectónico inválido: '{}'", pattern_str),
            format!("Valores válidos: {}", valid_patterns.join(", "))
        ).into());
    }

    // Validar campo: forbidden_imports
    if !obj.contains_key("forbidden_imports") {
        return Err(ConfigError::new(
            "Falta el campo requerido: forbidden_imports".to_string(),
            "Agrega este campo como un array, ejemplo: \"forbidden_imports\": []".to_string()
        ).into());
    }

    if !obj["forbidden_imports"].is_array() {
        return Err(ConfigError::new(
            "El campo 'forbidden_imports' debe ser un array".to_string(),
            "Ejemplo: \"forbidden_imports\": [{\"from\": \"src/components/**\", \"to\": \"src/services/**\"}]".to_string()
        ).into());
    }

    // Validar cada regla en forbidden_imports
    let rules = obj["forbidden_imports"].as_array().unwrap();
    for (index, rule) in rules.iter().enumerate() {
        if !rule.is_object() {
            return Err(ConfigError::new(
                format!("La regla #{} en forbidden_imports no es un objeto", index + 1),
                "Cada regla debe tener la forma: {\"from\": \"patrón1\", \"to\": \"patrón2\"}".to_string()
            ).into());
        }

        let rule_obj = rule.as_object().unwrap();

        if !rule_obj.contains_key("from") {
            return Err(ConfigError::new(
                format!("La regla #{} no tiene el campo 'from'", index + 1),
                "Ejemplo: {\"from\": \"src/components/**\", \"to\": \"src/services/**\"}".to_string()
            ).into());
        }

        if !rule_obj.contains_key("to") {
            return Err(ConfigError::new(
                format!("La regla #{} no tiene el campo 'to'", index + 1),
                "Ejemplo: {\"from\": \"src/components/**\", \"to\": \"src/services/**\"}".to_string()
            ).into());
        }

        if !rule_obj["from"].is_string() {
            return Err(ConfigError::new(
                format!("El campo 'from' en la regla #{} debe ser un string", index + 1),
                "Ejemplo: \"from\": \"src/components/**\"".to_string()
            ).into());
        }

        if !rule_obj["to"].is_string() {
            return Err(ConfigError::new(
                format!("El campo 'to' en la regla #{} debe ser un string", index + 1),
                "Ejemplo: \"to\": \"src/services/**\"".to_string()
            ).into());
        }
    }

    Ok(())
}

/// Valida los valores de la configuración (rangos, lógica, etc.)
fn validate_config_values(config: &ConfigFile) -> Result<()> {
    // Validar que max_lines_per_function esté en un rango razonable
    if config.max_lines_per_function == 0 {
        return Err(ConfigError::new(
            "max_lines_per_function no puede ser 0".to_string(),
            "Usa un valor entre 10 y 500. Recomendado: 20-60 según tu framework.".to_string()
        ).into());
    }

    if config.max_lines_per_function > 1000 {
        return Err(ConfigError::new(
            format!("max_lines_per_function es muy alto: {}", config.max_lines_per_function),
            "Un valor tan alto desactiva efectivamente esta validación. Máximo recomendado: 500".to_string()
        ).into());
    }

    // Validar que forbidden_imports tenga reglas únicas (no duplicadas)
    for (i, rule1) in config.forbidden_imports.iter().enumerate() {
        for (j, rule2) in config.forbidden_imports.iter().enumerate() {
            if i != j && rule1.from == rule2.from && rule1.to == rule2.to {
                return Err(ConfigError::new(
                    format!("Regla duplicada: from '{}' to '{}'", rule1.from, rule1.to),
                    "Elimina una de las reglas duplicadas en forbidden_imports.".to_string()
                ).into());
            }
        }
    }

    // Advertencia si no hay reglas (aunque técnicamente válido)
    if config.forbidden_imports.is_empty() {
        eprintln!("⚠️  Advertencia: No hay reglas en forbidden_imports. El linter solo validará la longitud de funciones.");
    }

    Ok(())
}

/// Orquestador de configuración: Carga silenciosa o Wizard con IA
pub fn setup_or_load_config(root: &Path) -> Result<std::sync::Arc<LinterContext>> {
    use std::sync::Arc;

    let config_path = root.join("architect.json");

    if config_path.exists() {
        // MODO AUTOMÁTICO: carga silenciosa
        let ctx = load_config(root)?;
        return Ok(Arc::new(ctx));
    }

    // MODO CONFIGURACIÓN (IA Discovery)
    println!("📝 No encontré 'architect.json'. Iniciando descubrimiento asistido por IA...");

    // 1. Discovery (Input local)
    let project_info = crate::discovery::get_architecture_snapshot(root);

    // 2. IA (Procesamiento inteligente)
    let suggestions = crate::ai::sugerir_arquitectura_inicial(project_info)
        .map_err(|e| miette::miette!("Error consultando la IA: {}", e))?;

    // 3. UI (Wizard de confirmación)
    let (selected_rules, max_lines) = crate::ui::ask_user_to_confirm_rules(suggestions)?;

    // 4. Config (Persistencia)
    let final_ctx = save_config_from_wizard(root, selected_rules, max_lines)?;

    println!("✅ Configuración guardada exitosamente.\n");
    Ok(Arc::new(final_ctx))
}

/// PERSISTENCIA: Guarda las reglas de la IA y devuelve el contexto nuevo
pub fn save_config_from_wizard(
    root: &Path,
    rules: Vec<SuggestedRule>,
    max_lines: usize,
) -> Result<LinterContext> {
    let config_path = root.join("architect.json");

    // Convertimos de SuggestedRule (IA) a ForbiddenRule (Linter)
    let forbidden_imports: Vec<ForbiddenRule> = rules
        .into_iter()
        .map(|r| ForbiddenRule {
            from: r.from,
            to: r.to,
        })
        .collect();

    let framework = crate::detector::detect_framework(root);

    // Valores por defecto para el primer architect.json
    let config = ConfigFile {
        max_lines_per_function: max_lines,
        architecture_pattern: ArchPattern::MVC, // O el que detecte la IA
        forbidden_imports: forbidden_imports.clone(),
    };

    let json = serde_json::to_string_pretty(&config).into_diagnostic()?;
    fs::write(config_path, json).into_diagnostic()?;

    Ok(LinterContext {
        max_lines: config.max_lines_per_function,
        framework,
        pattern: config.architecture_pattern,
        forbidden_imports,
    })
}

#[derive(Error, Debug, Diagnostic)]
#[error("Violación de Arquitectura")]
#[diagnostic(code(arch::violation), severity(error))]
pub struct ArchError {
    #[source_code]
    pub src: String,
    #[label("{message}")]
    pub span: SourceSpan,
    pub message: String,
}

#[derive(Error, Debug, Diagnostic)]
#[error("{details}")]
#[diagnostic(code(config::invalid), severity(error))]
pub struct ConfigError {
    #[help]
    pub help: String,
    pub details: String,
}

impl ConfigError {
    pub fn new(details: String, help: String) -> Self {
        Self { details, help }
    }
}
