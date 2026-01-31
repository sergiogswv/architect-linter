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
    pub framework: Framework,
    pub pattern: ArchPattern,
    pub forbidden_imports: Vec<ForbiddenRule>,
}

/// CARGA SILENCIOSA: Lee architect.json y lo convierte en contexto
pub fn load_config(root: &Path) -> Result<LinterContext> {
    let config_path = root.join("architect.json");
    let content = fs::read_to_string(config_path).into_diagnostic()?;
    let config: ConfigFile = serde_json::from_str(&content).into_diagnostic()?;

    // Re-detectamos el framework para el contexto actual
    let framework = crate::detector::detect_framework(root);

    Ok(LinterContext {
        max_lines: config.max_lines_per_function,
        framework,
        pattern: config.architecture_pattern,
        forbidden_imports: config.forbidden_imports,
    })
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
