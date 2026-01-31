use crate::config::{ArchError, LinterContext};
use miette::{IntoDiagnostic, Result, SourceSpan};
use std::path::PathBuf;
use swc_common::SourceMap;
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsConfig};

pub fn analyze_file(cm: &SourceMap, path: &PathBuf, ctx: &LinterContext) -> Result<()> {
    let fm = cm.load_file(path).into_diagnostic()?;

    let lexer = Lexer::new(
        Syntax::Typescript(TsConfig {
            decorators: true,
            ..Default::default()
        }),
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
                let from_pattern = rule.from.to_lowercase();
                let to_pattern = rule.to.to_lowercase();

                // Si el archivo está en la carpeta 'from' y el import contiene 'to'
                if file_path_str.contains(&from_pattern) && source.contains(&to_pattern) {
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
