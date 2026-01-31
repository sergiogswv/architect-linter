use indicatif::{ProgressBar, ProgressStyle};
use miette::{GraphicalReportHandler, IntoDiagnostic, Result};
use rayon::prelude::*;
use std::env;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use swc_common::SourceMap;

mod ai;
mod analyzer;
mod config;
mod detector;
mod discovery;
mod ui;

use crate::config::LinterContext;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    // Manejo de flags especiales
    if args.len() > 1 {
        match args[1].as_str() {
            "--version" | "-v" => {
                println!("architect-linter {}", VERSION);
                return Ok(());
            }
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            _ => {}
        }
    }

    println!("🏛️  WELCOME TO ARCHITECT-LINTER");

    // 1. Obtener la ruta del proyecto
    let project_root = if args.len() > 1 {
        PathBuf::from(&args[1]).canonicalize().into_diagnostic()?
    } else {
        ui::get_interactive_path()? // Movido a UI para limpiar el main
    };

    // 2. Cargar o crear configuración asistida por IA
    let ctx = setup_or_load_config(&project_root)?;

    // 3. Recolectar archivos .ts
    let files = discovery::collect_files(&project_root);
    if files.is_empty() {
        println!("✅ No se encontraron archivos .ts para analizar.");
        return Ok(());
    }

    // 4. Barra de progreso y Análisis Paralelo con Rayon
    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .into_diagnostic()?,
    );

    let error_count = Arc::new(Mutex::new(0));
    let cm = Arc::new(SourceMap::default());

    files.par_iter().for_each(|file_path| {
        // Clonamos el Arc del Contexto para cada hilo
        if let Err(e) = analyzer::analyze_file(&cm, file_path, &ctx) {
            let mut count = error_count.lock().unwrap();
            *count += 1;

            let mut out = String::new();
            let _ = GraphicalReportHandler::new().render_report(&mut out, e.as_ref());

            println!("\n📌 Violación en: {}", file_path.display());
            println!("{}", out);
        }
        pb.inc(1);
    });

    pb.finish_and_clear();

    // 5. Resultado final
    let total = *error_count.lock().unwrap();
    if total > 0 {
        println!("❌ Se encontraron {} violaciones arquitectónicas.", total);
        std::process::exit(1);
    } else {
        println!("✨ ¡Proyecto impecable! La arquitectura se respeta.");
        std::process::exit(0);
    }
}

/// Orquestador de configuración: Carga silenciosa o Wizard con IA
fn setup_or_load_config(root: &PathBuf) -> Result<Arc<LinterContext>> {
    let config_path = root.join("architect.json");

    if config_path.exists() {
        // MODO AUTOMÁTICO: delegamos la carga al módulo config
        let ctx = config::load_config(root)?;
        return Ok(Arc::new(ctx));
    }

    // MODO CONFIGURACIÓN (IA Discovery)
    println!("📝 No encontré 'architect.json'. Iniciando descubrimiento asistido por IA...");

    // 1. Discovery (Input local)
    let project_info = discovery::get_architecture_snapshot(root);

    // 2. IA (Procesamiento inteligente)
    // Mostramos un spinner pequeño aquí si gustas, o mensaje directo
    let suggestions = ai::sugerir_arquitectura_inicial(project_info)
        .map_err(|e| miette::miette!("Error consultando la IA: {}", e))?;

    // 3. UI (Wizard de confirmación)
    let (selected_rules, max_lines) = ui::ask_user_to_confirm_rules(suggestions)?;

    // 4. Config (Persistencia)
    let final_ctx = config::save_config_from_wizard(root, selected_rules, max_lines)?;

    println!("✅ Configuración guardada exitosamente.\n");
    Ok(Arc::new(final_ctx))
}

/// Muestra la ayuda del CLI
fn print_help() {
    println!("architect-linter {}", VERSION);
    println!();
    println!("Linter de arquitectura de software para proyectos TypeScript");
    println!();
    println!("USO:");
    println!("  architect-linter [OPCIONES] [RUTA]");
    println!();
    println!("ARGUMENTOS:");
    println!("  [RUTA]    Ruta del proyecto a analizar (opcional, modo interactivo si se omite)");
    println!();
    println!("OPCIONES:");
    println!("  -h, --help       Muestra esta ayuda");
    println!("  -v, --version    Muestra la versión");
    println!();
    println!("EJEMPLOS:");
    println!("  architect-linter                    # Modo interactivo");
    println!("  architect-linter .                  # Analizar directorio actual");
    println!("  architect-linter /ruta/a/proyecto   # Analizar proyecto específico");
    println!();
    println!("DOCUMENTACIÓN:");
    println!("  https://github.com/sergio/architect-linter");
}
