//! Go parser using Tree-sitter

use super::{ArchitectParser, Import};
use crate::autofix::Violation;
use crate::config::{ForbiddenRule, LinterContext};
use miette::{IntoDiagnostic, Result};
use std::path::Path;
use std::sync::Mutex;
use tree_sitter::{Parser, Query, QueryCursor};

pub struct GoParser {
    parser: Mutex<Parser>,
}

impl GoParser {
    pub fn new() -> Self {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_go::language())
            .expect("Failed to load Go grammar");

        Self {
            parser: Mutex::new(parser),
        }
    }

    /// Check if a file/import path matches a pattern (Go-specific)
    fn matches_pattern(path: &str, pattern: &str) -> bool {
        let normalized_path = path.to_lowercase().replace('\\', "/");
        let normalized_pattern = pattern
            .to_lowercase()
            .replace('\\', "/")
            .replace("**", "")
            .replace('*', "");

        if normalized_path.contains(&normalized_pattern) {
            return true;
        }

        // Go imports are package paths: github.com/user/repo/pkg
        // Check if import ends with pattern folder
        if let Some(last_segment) = normalized_path.split('/').last() {
            if let Some(pattern_last) = normalized_pattern.split('/').last() {
                if last_segment.contains(pattern_last) {
                    return true;
                }
            }
        }

        false
    }
}

impl ArchitectParser for GoParser {
    fn extract_imports(&self, source_code: &str, _file_path: &Path) -> Result<Vec<Import>> {
        let mut imports = Vec::new();

        // Parse the source code
        let tree = self
            .parser
            .lock()
            .unwrap()
            .parse(source_code, None)
            .ok_or_else(|| miette::miette!("Failed to parse Go"))?;

        // Query for import declarations
        // Go has both single and grouped imports
        let query_source = r#"
            [
              (import_declaration
                (import_spec
                  path: (interpreted_string_literal) @import_path))
            ]
        "#;

        let query = Query::new(&tree_sitter_go::language(), query_source).into_diagnostic()?;

        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&query, tree.root_node(), source_code.as_bytes());

        for match_ in matches {
            for capture in match_.captures {
                let node = capture.node;
                let import_path_raw = node.utf8_text(source_code.as_bytes()).into_diagnostic()?;

                // Remove quotes from Go string literal
                let import_path = import_path_raw.trim_matches('"');
                let line_number = node.start_position().row + 1;

                // Get the full import statement
                let mut parent = node.parent();
                while let Some(p) = parent {
                    if p.kind() == "import_spec" || p.kind() == "import_declaration" {
                        break;
                    }
                    parent = p.parent();
                }

                let raw_statement = if let Some(p) = parent {
                    p.utf8_text(source_code.as_bytes())
                        .unwrap_or(import_path)
                        .to_string()
                } else {
                    format!("import \"{}\"", import_path)
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

            // Go-specific rules
            // Example: handlers shouldn't import database directly
            if file_path_str.contains("handlers") && import.source.contains("database") {
                violations.push(Violation {
                    file_path: file_path.to_path_buf(),
                    file_content: source_code.to_string(),
                    offensive_import: import.raw_statement.clone(),
                    rule: ForbiddenRule {
                        from: "handlers".to_string(),
                        to: "database".to_string(),
                    },
                    line_number: import.line_number,
                });
            }
        }

        Ok(violations)
    }
}
