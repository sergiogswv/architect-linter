use crate::autofix::Violation;
use crate::config::{ArchError, LinterContext};
use crate::parsers;
use miette::{IntoDiagnostic, Result, SourceSpan};
use std::fs;
use std::path::PathBuf;
use swc_common::SourceMap;
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsConfig, EsConfig};

pub fn analyze_file(cm: &SourceMap, path: &PathBuf, ctx: &LinterContext) -> Result<()> {
    // Try to use multi-language parser first
    if let Some(parser) = parsers::get_parser_for_file(path) {
        let source_code = fs::read_to_string(path).into_diagnostic()?;
        let violations = parser.find_violations(&source_code, path, ctx)?;

        if let Some(first_violation) = violations.first() {
            // Return error for the first violation
            let fm = cm.load_file(path).into_diagnostic()?;
            return Err(create_error_from_violation(&fm, first_violation));
        }

        // Validate method length for TypeScript/JavaScript files
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if matches!(extension, "ts" | "tsx" | "js" | "jsx") {
            validate_method_length(cm, path, ctx)?;
        }

        return Ok(());
    }

    // Fallback to old swc parser for unsupported files
    let fm = cm.load_file(path).into_diagnostic()?;

    // Detectar si es TypeScript o JavaScript según la extensión
    let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let syntax = match extension {
        "ts" | "tsx" => Syntax::Typescript(TsConfig {
            decorators: true,
            tsx: extension == "tsx",
            ..Default::default()
        }),
        "js" | "jsx" => Syntax::Es(EsConfig {
            decorators: true,
            jsx: extension == "jsx",
            ..Default::default()
        }),
        _ => Syntax::Typescript(TsConfig::default()),
    };

    let lexer = Lexer::new(
        syntax,
        Default::default(),
        StringInput::from(&*fm),
        None,
    );

    let mut parser = Parser::new_from(lexer);
    let module = parser
        .parse_module()
        .map_err(|e| miette::miette!("Syntax Error: {:?}", e))?;

    let file_path_str = path.to_string_lossy().to_lowercase();

    for item in &module.body {
        // --- VALIDACIÓN DE IMPORTACIONES DINÁMICAS ---
        if let swc_ecma_ast::ModuleItem::ModuleDecl(swc_ecma_ast::ModuleDecl::Import(import)) = item
        {
            let source = import.src.value.to_string().to_lowercase();

            // 1. Validamos las reglas dinámicas del JSON
            for rule in &ctx.forbidden_imports {
                // Normalizar patrones: quitar '**', '*', y '/' al final para matching flexible
                let from_pattern = normalize_pattern(&rule.from);
                let to_pattern = normalize_pattern(&rule.to);

                // Verificar si el archivo coincide con el patrón 'from'
                let file_matches = matches_pattern(&file_path_str, &from_pattern);

                // Verificar si el import coincide con el patrón 'to'
                let import_matches = matches_pattern(&source, &to_pattern);

                if file_matches && import_matches {
                    return Err(create_error(
                        &fm,
                        import.span,
                        &format!(
                            "Restricción: Archivos en '{}' no pueden importar de '{}'.",
                            rule.from, rule.to
                        ),
                    ));
                }
            }

            // 2. Regla extra: Siempre prohibir Repository en Controller (Standard NestJS)
            if file_path_str.contains("controller") && source.contains(".repository") {
                return Err(create_error(
                    &fm,
                    import.span,
                    "MVC: Prohibido importar Repositorios en Controladores.",
                ));
            }
        }

        // --- VALIDACIÓN DE LÍNEAS POR MÉTODO ---
        if let swc_ecma_ast::ModuleItem::Stmt(swc_ecma_ast::Stmt::Decl(
            swc_ecma_ast::Decl::Class(c),
        )) = item
        {
            for member in &c.class.body {
                if let swc_ecma_ast::ClassMember::Method(m) = member {
                    let lo = cm.lookup_char_pos(m.span.lo).line;
                    let hi = cm.lookup_char_pos(m.span.hi).line;
                    let lines = hi - lo;

                    if lines > ctx.max_lines {
                        return Err(create_error(
                            &fm,
                            m.span,
                            &format!(
                                "Método demasiado largo ({} líneas). Máximo: {}.",
                                lines, ctx.max_lines
                            ),
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}

/// Normaliza un patrón glob para hacer matching simple
/// Ejemplos:
/// - "src/components/**" → "src/components/"
/// - "**/*.tsx" → ".tsx"
/// - "src/services/**" → "src/services/"
fn normalize_pattern(pattern: &str) -> String {
    let normalized = pattern
        .to_lowercase()
        .replace("\\", "/")  // Normalizar separadores de Windows
        .replace("**", "")   // Quitar comodines globales
        .replace("*", "");   // Quitar comodines simples

    // Si el patrón termina en /, dejarlo; si no, mantenerlo como está
    normalized
}

/// Verifica si un path coincide con un patrón normalizado
/// Usa matching flexible para soportar diferentes formatos de import
fn matches_pattern(path: &str, pattern: &str) -> bool {
    let normalized_path = path.to_lowercase().replace("\\", "/");
    let normalized_pattern = pattern.to_lowercase();

    // Si el patrón está vacío después de normalización, no matchear nada
    if normalized_pattern.is_empty() {
        return false;
    }

    // Matching flexible para rutas absolutas y relativas
    // Ejemplos:
    // - Path: "c:/proyecto/src/components/button.jsx" con Pattern: "src/components/"
    // - Import: "../services/userservice" con Pattern: "src/services/"
    //   → Extraer "services/" del pattern y buscar "/services/" o "../services/" en el import
    // - Import: "@/api/posts" con Pattern: "src/api/"
    //   → Buscar "/api/" en el import

    if normalized_path.contains(&normalized_pattern) {
        return true;
    }

    // Para imports: si el patrón contiene "src/", extraer solo la carpeta después de src/
    // Ejemplo: "src/services/" → buscar también "/services/" o "services/"
    if normalized_pattern.contains("src/") {
        // Extraer la parte después de "src/"
        if let Some(folder_part) = normalized_pattern.strip_prefix("src/") {
            // Buscar "/folder/" o "../folder/" en el path (para imports relativos)
            let with_slash = format!("/{}", folder_part);
            let with_relative = format!("../{}", folder_part);
            let with_at = format!("@/{}", folder_part); // Para alias como @/services

            if normalized_path.contains(&with_slash)
                || normalized_path.contains(&with_relative)
                || normalized_path.contains(&with_at)
                || normalized_path.contains(folder_part) {
                return true;
            }
        }
    }

    false
}

fn create_error(fm: &swc_common::SourceFile, span: swc_common::Span, msg: &str) -> miette::Report {
    let start = (span.lo.0 - fm.start_pos.0) as usize;
    let end = (span.hi.0 - fm.start_pos.0) as usize;

    ArchError {
        src: fm.src.to_string(),
        span: SourceSpan::new(start.into(), (end - start).into()),
        message: msg.to_string(),
    }
    .into()
}

/// Create a miette error from a Violation
fn create_error_from_violation(fm: &swc_common::SourceFile, violation: &Violation) -> miette::Report {
    // Try to find the import line in the source
    let lines: Vec<&str> = fm.src.lines().collect();
    let line_idx = violation.line_number.saturating_sub(1);

    if line_idx < lines.len() {
        let line_content = lines[line_idx];
        // Find the position of the import statement in the file
        let mut char_offset = 0;
        for (idx, line) in lines.iter().enumerate() {
            if idx == line_idx {
                break;
            }
            char_offset += line.len() + 1; // +1 for newline
        }

        ArchError {
            src: fm.src.to_string(),
            span: SourceSpan::new(char_offset.into(), line_content.len().into()),
            message: format!(
                "Restricción: Archivos en '{}' no pueden importar de '{}'.",
                violation.rule.from, violation.rule.to
            ),
        }
        .into()
    } else {
        // Fallback if line number is out of bounds
        miette::miette!(
            "Restricción: Archivos en '{}' no pueden importar de '{}' (línea {}).",
            violation.rule.from,
            violation.rule.to,
            violation.line_number
        )
    }
}

/// Validate method length for TypeScript/JavaScript files using swc
fn validate_method_length(cm: &SourceMap, path: &PathBuf, ctx: &LinterContext) -> Result<()> {
    let fm = cm.load_file(path).into_diagnostic()?;

    let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let syntax = match extension {
        "ts" | "tsx" => Syntax::Typescript(TsConfig {
            decorators: true,
            tsx: extension == "tsx",
            ..Default::default()
        }),
        "js" | "jsx" => Syntax::Es(EsConfig {
            decorators: true,
            jsx: extension == "jsx",
            ..Default::default()
        }),
        _ => return Ok(()),
    };

    let lexer = Lexer::new(
        syntax,
        Default::default(),
        StringInput::from(&*fm),
        None,
    );

    let mut parser = Parser::new_from(lexer);
    let module = match parser.parse_module() {
        Ok(m) => m,
        Err(_) => return Ok(()), // Skip on syntax error
    };

    for item in &module.body {
        if let swc_ecma_ast::ModuleItem::Stmt(swc_ecma_ast::Stmt::Decl(
            swc_ecma_ast::Decl::Class(c),
        )) = item
        {
            for member in &c.class.body {
                if let swc_ecma_ast::ClassMember::Method(m) = member {
                    let lo = cm.lookup_char_pos(m.span.lo).line;
                    let hi = cm.lookup_char_pos(m.span.hi).line;
                    let lines = hi - lo;

                    if lines > ctx.max_lines {
                        return Err(create_error(
                            &fm,
                            m.span,
                            &format!(
                                "Método demasiado largo ({} líneas). Máximo: {}.",
                                lines, ctx.max_lines
                            ),
                        ));
                    }
                }
            }
        }
    }

    Ok(())
}

/// Analiza un archivo y recolecta todas las violaciones sin fallar
/// Útil para el modo --fix donde queremos procesar todas las violaciones
pub fn collect_violations_from_file(
    _cm: &SourceMap,
    path: &PathBuf,
    ctx: &LinterContext,
) -> Result<Vec<Violation>> {
    // Try to use multi-language parser first
    if let Some(parser) = parsers::get_parser_for_file(path) {
        let source_code = fs::read_to_string(path).into_diagnostic()?;
        return parser.find_violations(&source_code, path, ctx);
    }

    // Fallback: return empty violations for unsupported files
    Ok(Vec::new())
}
