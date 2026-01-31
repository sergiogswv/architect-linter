#![allow(unused_assignments)]

use miette::{Diagnostic, SourceSpan};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Framework {
    NestJS,
    React,
    Angular,
    Express,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ArchPattern {
    Hexagonal,
    Clean,
    MVC,
    Ninguno,
}

/// Nueva estructura para las reglas dinámicas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForbiddenRule {
    pub from: String, // Carpeta origen (donde se escribe el código)
    pub to: String,   // Carpeta prohibida (lo que no se puede importar)
}

/// Contexto que incluye las reglas dinámicas cargadas del JSON
pub struct LinterContext {
    pub max_lines: usize,
    #[allow(dead_code)]
    pub framework: Framework,
    #[allow(dead_code)]
    pub pattern: ArchPattern,
    pub forbidden_imports: Vec<ForbiddenRule>, // La lista de reglas
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
