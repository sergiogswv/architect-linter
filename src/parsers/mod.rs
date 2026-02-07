//! Parsers module - Multi-language parsing abstraction using Tree-sitter
//!
//! This module provides a unified interface for parsing source code across
//! different programming languages using Tree-sitter.

use crate::autofix::Violation;
use crate::config::LinterContext;
use miette::Result;
use std::path::Path;

pub mod typescript;
pub mod python;
pub mod go;
pub mod php;
pub mod java;

/// Represents an import statement extracted from source code
#[derive(Debug, Clone)]
pub struct Import {
    /// The import source/path (e.g., "../services/user", "apps.user.models")
    pub source: String,
    /// Line number where the import appears
    pub line_number: usize,
    /// Full import statement text
    pub raw_statement: String,
}

/// Language identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    TypeScript,
    JavaScript,
    Python,
    Go,
    Rust,
    Php,
    Java,
}

impl Language {
    /// Get language from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            "ts" | "tsx" => Some(Language::TypeScript),
            "js" | "jsx" => Some(Language::JavaScript),
            "py" => Some(Language::Python),
            "go" => Some(Language::Go),
            "rs" => Some(Language::Rust),
            "php" => Some(Language::Php),
            "java" => Some(Language::Java),
            _ => None,
        }
    }

    /// Get file extensions for this language
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            Language::TypeScript => &["ts", "tsx"],
            Language::JavaScript => &["js", "jsx"],
            Language::Python => &["py"],
            Language::Go => &["go"],
            Language::Rust => &["rs"],
            Language::Php => &["php"],
            Language::Java => &["java"],
        }
    }
}

/// Main trait for language parsers
///
/// Implementations of this trait provide language-specific parsing logic
/// using Tree-sitter to extract imports and detect architectural violations.
pub trait ArchitectParser: Send + Sync {
    /// Extract all imports from source code
    fn extract_imports(&self, source_code: &str, file_path: &Path) -> Result<Vec<Import>>;

    /// Find architectural violations in a file
    fn find_violations(
        &self,
        source_code: &str,
        file_path: &Path,
        context: &LinterContext,
    ) -> Result<Vec<Violation>>;
}

/// Factory function to get appropriate parser for a file
pub fn get_parser_for_file(file_path: &Path) -> Option<Box<dyn ArchitectParser>> {
    let ext = file_path.extension()?.to_str()?;
    let lang = Language::from_extension(ext)?;

    match lang {
        Language::TypeScript | Language::JavaScript => {
            Some(Box::new(typescript::TypeScriptParser::new()))
        }
        Language::Python => {
            Some(Box::new(python::PythonParser::new()))
        }
        Language::Go => {
            Some(Box::new(go::GoParser::new()))
        }
        Language::Php => {
            Some(Box::new(php::PhpParser::new()))
        }
        Language::Java => {
            Some(Box::new(java::JavaParser::new()))
        }
        Language::Rust => {
            // Rust support coming soon
            None
        }
    }
}

/// Get all supported languages
pub fn supported_languages() -> Vec<Language> {
    vec![
        Language::TypeScript,
        Language::JavaScript,
        Language::Python,
        Language::Go,
        Language::Php,
        Language::Java,
    ]
}

/// Get all supported file extensions
pub fn supported_extensions() -> Vec<&'static str> {
    let mut extensions = Vec::new();
    for lang in supported_languages() {
        extensions.extend_from_slice(lang.extensions());
    }
    extensions
}
