//! TypeScript/JavaScript parser using Tree-sitter

use super::{ArchitectParser, Import};
use crate::autofix::Violation;
use crate::config::{ForbiddenRule, LinterContext};
use miette::{IntoDiagnostic, Result};
use std::path::Path;
use std::sync::Mutex;
use tree_sitter::{Parser, Query, QueryCursor};

pub struct TypeScriptParser {
    parser: Mutex<Parser>,
}

impl TypeScriptParser {
    pub fn new() -> Self {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_typescript::language_typescript())
            .expect("Failed to load TypeScript grammar");

        Self {
            parser: Mutex::new(parser),
        }
    }

    /// Check if a file/import path matches a pattern
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

        // For imports: if pattern contains "src/", extract folder part
        if normalized_pattern.contains("src/") {
            if let Some(folder_part) = normalized_pattern.strip_prefix("src/") {
                let with_slash = format!("/{}", folder_part);
                let with_relative = format!("../{}", folder_part);
                let with_at = format!("@/{}", folder_part);

                if normalized_path.contains(&with_slash)
                    || normalized_path.contains(&with_relative)
                    || normalized_path.contains(&with_at)
                    || normalized_path.contains(folder_part)
                {
                    return true;
                }
            }
        }

        false
    }
}

impl ArchitectParser for TypeScriptParser {
    fn extract_imports(&self, source_code: &str, _file_path: &Path) -> Result<Vec<Import>> {
        let mut imports = Vec::new();

        // Parse the source code
        let tree = self
            .parser
            .lock()
            .unwrap()
            .parse(source_code, None)
            .ok_or_else(|| miette::miette!("Failed to parse TypeScript"))?;

        // Query for import declarations
        let query_source = r#"
            (import_statement
              source: (string (string_fragment) @import_path))
        "#;

        let query = Query::new(&tree_sitter_typescript::language_typescript(), query_source)
            .into_diagnostic()?;

        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&query, tree.root_node(), source_code.as_bytes());

        for match_ in matches {
            for capture in match_.captures {
                let node = capture.node;
                let import_path = node.utf8_text(source_code.as_bytes()).into_diagnostic()?;
                let line_number = node.start_position().row + 1;

                // Get the full import statement
                let parent = node.parent();
                let raw_statement = if let Some(p) = parent {
                    p.utf8_text(source_code.as_bytes())
                        .unwrap_or(import_path)
                        .to_string()
                } else {
                    format!("import ... from '{}'", import_path)
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
                let import_matches = Self::matches_pattern(&import.source.to_lowercase(), &rule.to);

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

            // Extra rule: Controller → Repository
            if file_path_str.contains("controller") && import.source.to_lowercase().contains(".repository") {
                violations.push(Violation {
                    file_path: file_path.to_path_buf(),
                    file_content: source_code.to_string(),
                    offensive_import: import.raw_statement.clone(),
                    rule: ForbiddenRule {
                        from: "controller".to_string(),
                        to: ".repository".to_string(),
                    },
                    line_number: import.line_number,
                });
            }
        }

        Ok(violations)
    }
}
