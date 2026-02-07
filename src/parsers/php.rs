//! PHP parser using Tree-sitter

use super::{ArchitectParser, Import};
use crate::autofix::Violation;
use crate::config::{ForbiddenRule, LinterContext};
use miette::{IntoDiagnostic, Result};
use std::path::Path;
use std::sync::Mutex;
use tree_sitter::{Parser, Query, QueryCursor};

pub struct PhpParser {
    parser: Mutex<Parser>,
}

impl PhpParser {
    pub fn new() -> Self {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_php::language_php())
            .expect("Failed to load PHP grammar");

        Self {
            parser: Mutex::new(parser),
        }
    }

    /// Check if a file/import path matches a pattern (PHP-specific)
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

        // PHP uses namespaces with backslashes: App\Controllers\UserController
        // Also handle PSR-4 autoloading paths
        let path_with_backslash = normalized_path.replace('/', "\\");
        if path_with_backslash.contains(&normalized_pattern.replace('/', "\\")) {
            return true;
        }

        // Check if import contains pattern folder
        if let Some(last_segment) = normalized_path.split(&['/', '\\']).last() {
            if let Some(pattern_last) = normalized_pattern.split(&['/', '\\']).last() {
                if last_segment.contains(pattern_last) {
                    return true;
                }
            }
        }

        false
    }
}

impl ArchitectParser for PhpParser {
    fn extract_imports(&self, source_code: &str, _file_path: &Path) -> Result<Vec<Import>> {
        let mut imports = Vec::new();

        // Parse the source code
        let tree = self
            .parser
            .lock()
            .unwrap()
            .parse(source_code, None)
            .ok_or_else(|| miette::miette!("Failed to parse PHP"))?;

        // Query for use declarations and require/include statements
        let query_source = r#"
            [
              (namespace_use_declaration
                (namespace_use_clause
                  (qualified_name) @import_path))
              (require_expression
                (string) @import_path)
              (require_once_expression
                (string) @import_path)
              (include_expression
                (string) @import_path)
              (include_once_expression
                (string) @import_path)
            ]
        "#;

        let query = Query::new(&tree_sitter_php::language_php(), query_source).into_diagnostic()?;

        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&query, tree.root_node(), source_code.as_bytes());

        for match_ in matches {
            for capture in match_.captures {
                let node = capture.node;
                let import_path_raw = node.utf8_text(source_code.as_bytes()).into_diagnostic()?;

                // Remove quotes from string literals (for require/include)
                let import_path = import_path_raw.trim_matches('"').trim_matches('\'');
                let line_number = node.start_position().row + 1;

                // Get the full import statement
                let mut parent = node.parent();
                while let Some(p) = parent {
                    if p.kind() == "namespace_use_declaration"
                        || p.kind() == "require_expression"
                        || p.kind() == "require_once_expression"
                        || p.kind() == "include_expression"
                        || p.kind() == "include_once_expression"
                    {
                        break;
                    }
                    parent = p.parent();
                }

                let raw_statement = if let Some(p) = parent {
                    p.utf8_text(source_code.as_bytes())
                        .unwrap_or(import_path)
                        .to_string()
                } else {
                    format!("use {};", import_path)
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

            // PHP-specific rules
            // Example: Controllers shouldn't import Models directly
            if file_path_str.contains("controller") && import.source.to_lowercase().contains("model") {
                violations.push(Violation {
                    file_path: file_path.to_path_buf(),
                    file_content: source_code.to_string(),
                    offensive_import: import.raw_statement.clone(),
                    rule: ForbiddenRule {
                        from: "controller".to_string(),
                        to: "model".to_string(),
                    },
                    line_number: import.line_number,
                });
            }
        }

        Ok(violations)
    }
}
