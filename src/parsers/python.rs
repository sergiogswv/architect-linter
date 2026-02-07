//! Python parser using Tree-sitter

use super::{ArchitectParser, Import};
use crate::autofix::Violation;
use crate::config::{ForbiddenRule, LinterContext};
use miette::{IntoDiagnostic, Result};
use std::path::Path;
use std::sync::Mutex;
use tree_sitter::{Parser, Query, QueryCursor};

pub struct PythonParser {
    parser: Mutex<Parser>,
}

impl PythonParser {
    pub fn new() -> Self {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_python::language())
            .expect("Failed to load Python grammar");

        Self {
            parser: Mutex::new(parser),
        }
    }

    /// Check if a file/import path matches a pattern (Python-specific)
    fn matches_pattern(path: &str, pattern: &str) -> bool {
        let normalized_path = path.to_lowercase().replace('\\', "/");
        let normalized_pattern = pattern
            .to_lowercase()
            .replace('\\', "/")
            .replace("**", "")
            .replace('*', "");

        // Python imports use dots: apps.user.models
        // Convert to path-like for matching: apps/user/models
        let path_as_dots = normalized_path.replace('/', ".");
        let pattern_as_dots = normalized_pattern.replace('/', ".");

        if path_as_dots.contains(&pattern_as_dots) || normalized_path.contains(&normalized_pattern) {
            return true;
        }

        // Check if import contains pattern folder
        if pattern_as_dots.contains('.') {
            let parts: Vec<&str> = pattern_as_dots.split('.').collect();
            for part in parts {
                if !part.is_empty() && path_as_dots.contains(part) {
                    return true;
                }
            }
        }

        false
    }
}

impl ArchitectParser for PythonParser {
    fn extract_imports(&self, source_code: &str, _file_path: &Path) -> Result<Vec<Import>> {
        let mut imports = Vec::new();

        // Parse the source code
        let tree = self
            .parser
            .lock()
            .unwrap()
            .parse(source_code, None)
            .ok_or_else(|| miette::miette!("Failed to parse Python"))?;

        // Query for different types of imports
        // 1. import x.y.z
        // 2. from x.y import z
        let query_source = r#"
            [
              (import_statement
                name: (dotted_name) @import_path)
              (import_from_statement
                module_name: (dotted_name) @import_path)
            ]
        "#;

        let query = Query::new(&tree_sitter_python::language(), query_source).into_diagnostic()?;

        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&query, tree.root_node(), source_code.as_bytes());

        for match_ in matches {
            for capture in match_.captures {
                let node = capture.node;
                let import_path = node.utf8_text(source_code.as_bytes()).into_diagnostic()?;
                let line_number = node.start_position().row + 1;

                // Get the full import statement
                let mut parent = node.parent();
                while let Some(p) = parent {
                    if p.kind() == "import_statement" || p.kind() == "import_from_statement" {
                        break;
                    }
                    parent = p.parent();
                }

                let raw_statement = if let Some(p) = parent {
                    p.utf8_text(source_code.as_bytes())
                        .unwrap_or(import_path)
                        .to_string()
                } else {
                    format!("import {}", import_path)
                };

                imports.push(Import {
                    source: import_path.to_string(),
                    line_number,
                    raw_statement,
                });
            }
        }

        Ok(imports)
    }

    fn find_violations(
        &self,
        source_code: &str,
        file_path: &Path,
        context: &LinterContext,
    ) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();
        let imports = self.extract_imports(source_code, file_path)?;
        let file_path_str = file_path.to_string_lossy().to_lowercase();

        for import in imports {
            // Check against forbidden rules
            for rule in &context.forbidden_imports {
                let file_matches = Self::matches_pattern(&file_path_str, &rule.from);
                let import_matches = Self::matches_pattern(&import.source, &rule.to);

                if file_matches && import_matches {
                    violations.push(Violation {
                        file_path: file_path.to_path_buf(),
                        file_content: source_code.to_string(),
                        offensive_import: import.raw_statement.clone(),
                        rule: ForbiddenRule {
                            from: rule.from.clone(),
                            to: rule.to.clone(),
                        },
                        line_number: import.line_number,
                    });
                }
            }

            // Python-specific rules (similar to Controller → Repository in TS)
            // Example: views shouldn't import models directly
            if file_path_str.contains("views") && import.source.contains("models") {
                violations.push(Violation {
                    file_path: file_path.to_path_buf(),
                    file_content: source_code.to_string(),
                    offensive_import: import.raw_statement.clone(),
                    rule: ForbiddenRule {
                        from: "views".to_string(),
                        to: "models".to_string(),
                    },
                    line_number: import.line_number,
                });
            }
        }

        Ok(violations)
    }
}
