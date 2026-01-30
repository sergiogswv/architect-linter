use dialoguer::{theme::ColorfulTheme, Input, Select};
use indicatif::{ProgressBar, ProgressStyle};
use miette::{Diagnostic, GraphicalReportHandler, IntoDiagnostic, Result, SourceSpan};
use rayon::prelude::*;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;
use walkdir::WalkDir;

// Herramientas de SWC
use swc_common::SourceMap;
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsConfig};

// --- DEFINICIÓN DEL ERROR VISUAL ---
#[derive(Error, Debug, Diagnostic)]
#[error("Violación de Arquitectura: Importación Prohibida")]
#[diagnostic(
    code(arch::forbidden_import),
    severity(error),
    help("Los controladores (Controllers) deben usar Servicios, nunca Repositorios directamente.")
)]
struct ArchError {
    #[source_code]
    src: String,

    #[label("Este import de repositorio no está permitido aquí")]
    span: SourceSpan,

    // Cambiado a snake_case para seguir el estándar de Rust
    file_name: String,
}
#[derive(Deserialize)]
struct Config {
    max_lines_per_function: usize,
    // Podrías expandir esto para los imports luego
}

fn main() -> Result<()> {
    println!("🏛️  WELCOME TO ARCHITECT-LINTER");

    let current_dir = std::env::current_dir().into_diagnostic()?;
    let search_dir = current_dir.parent().unwrap_or(&current_dir);

    let entries = std::fs::read_dir(search_dir).into_diagnostic()?;
    let mut projects: Vec<PathBuf> = Vec::new();

    for entry in entries {
        let entry = entry.into_diagnostic()?;
        let path = entry.path();
        if path.is_dir() && path.join("package.json").exists() {
            projects.push(path);
        }
    }

    let mut options: Vec<String> = projects
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    options.push(">> Ingresar ruta manualmente...".to_string());

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Selecciona el proyecto a auditar")
        .items(&options)
        .default(0)
        .interact()
        .into_diagnostic()?;

    let final_path = if selection == options.len() - 1 {
        let manual: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Escribe la ruta completa")
            .interact_text()
            .into_diagnostic()?;
        PathBuf::from(manual)
    } else {
        projects[selection].clone()
    };

    let files: Vec<PathBuf> = WalkDir::new(&final_path)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            name != "node_modules" && name != "dist" && name != ".git" && name != "target"
        })
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "ts"))
        .map(|e| e.path().to_path_buf())
        .collect();

    println!(
        "🚀 Analizando {} archivos en {:?}...",
        files.len(),
        final_path.file_name().unwrap()
    );

    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
            )
            .into_diagnostic()?
            .progress_chars("#>-"),
    );

    // PROCESAMIENTO PARALELO
    files.par_iter().for_each(|file_path| {
        let cm = Arc::new(SourceMap::default());
        if let Err(e) = check_file_architecture(&cm, file_path) {
            // Renderización simplificada y compatible con Miette 7.x
            let mut out = String::new();
            let handler = GraphicalReportHandler::new();
            let _ = handler.render_report(&mut out, e.as_ref());

            // Imprimimos el encabezado del archivo y el error
            println!("\n📌 Archivo: {}", file_path.display());
            println!("{}", out);
        }
        pb.inc(1);
    });

    pb.finish_with_message("Análisis completado");
    Ok(())
}

fn check_file_architecture(cm: &SourceMap, path: &PathBuf, project_root: &PathBuf) -> Result<()> {
    // 1. CARGAR CONFIGURACIÓN DINÁMICA
    let config_path = project_root.join("architect.json");
    let max_lines = if config_path.exists() {
        let content = std::fs::read_to_string(config_path).unwrap_or_default();
        let json: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();
        json["max_lines_per_function"].as_u64().unwrap_or(200) as usize
    } else {
        200 // Default si no hay JSON
    };

    // 2. PARSEO DE ARCHIVO
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
    let file_path_str = path.to_string_lossy();

    // 3. ANÁLISIS DE CONTENIDO
    for item in &module.body {
        match item {
            // Caso A: Importaciones (Regla de Arquitectura)
            swc_ecma_ast::ModuleItem::ModuleDecl(swc_ecma_ast::ModuleDecl::Import(import)) => {
                let source = &import.src.value;
                if file_path_str.ends_with(".controller.ts") && source.contains(".repository") {
                    let start = (import.span.lo.0 - fm.start_pos.0) as usize;
                    let end = (import.span.hi.0 - fm.start_pos.0) as usize;
                    return Err(ArchError {
                        src: fm.src.to_string(),
                        span: SourceSpan::new(start.into(), (end - start).into()),
                        file_name: file_path_str.into_owned(),
                    }
                    .into());
                }
            }

            // Caso B: Métodos dentro de Clases (Lo más común en NestJS: @Get(), @Post()...)
            swc_ecma_ast::ModuleItem::Stmt(swc_ecma_ast::Stmt::Decl(
                swc_ecma_ast::Decl::Class(class_decl),
            )) => {
                for member in &class_decl.class.body {
                    if let swc_ecma_ast::ClassMember::Method(method) = member {
                        let span = method.span;
                        let start_line = cm.lookup_char_pos(span.lo).line;
                        let end_line = cm.lookup_char_pos(span.hi).line;
                        let lines_count = end_line - start_line;

                        if lines_count > max_lines {
                            let method_name = match &method.key {
                                swc_ecma_ast::PropName::Ident(i) => i.sym.to_string(),
                                _ => "anónimo".to_string(),
                            };
                            println!(
                                "⚠️  [COMPLEJIDAD] Método '{}' en clase '{}' tiene {} líneas.",
                                method_name, class_decl.ident.sym, lines_count
                            );
                        }
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}
