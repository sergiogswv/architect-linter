use indicatif::{ProgressBar, ProgressStyle};
use miette::{GraphicalReportHandler, IntoDiagnostic, Result};
use rayon::prelude::*;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use swc_common::SourceMap;

mod ai;
mod analyzer;
mod circular;
mod cli;
mod config;
mod detector;
mod discovery;
mod ui;

fn main() -> Result<()> {
    // 1. Procesar argumentos de línea de comandos
    let args = match cli::process_args() {
        Some(args) => args,
        None => return Ok(()), // Se procesó --help o --version
    };

    ui::print_banner();

    // 2. Obtener la ruta del proyecto
    let project_root = if args.len() > 1 {
        PathBuf::from(&args[1]).canonicalize().into_diagnostic()?
    } else {
        ui::get_interactive_path()?
    };

    // 3. Cargar o crear configuración asistida por IA
    let ctx = config::setup_or_load_config(&project_root)?;

    // 4. Recolectar archivos .ts, .tsx, .js, .jsx
    let files = discovery::collect_files(&project_root);
    if files.is_empty() {
        println!("✅ No se encontraron archivos TypeScript/JavaScript para analizar.");
        return Ok(());
    }

    // 5. Barra de progreso y Análisis Paralelo con Rayon
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

    // 6. Análisis de Dependencias Cíclicas
    println!("\n🔍 Analizando dependencias cíclicas...");
    let cycles = circular::analyze_circular_dependencies(&files, &project_root, &cm);

    match cycles {
        Ok(detected_cycles) => {
            if !detected_cycles.is_empty() {
                circular::print_circular_dependency_report(&detected_cycles);
                println!("\n⚠️  Se encontraron dependencias cíclicas que deben ser resueltas.");
                std::process::exit(1);
            }
        }
        Err(e) => {
            println!("⚠️  No se pudo analizar dependencias cíclicas: {}", e);
            println!("💡 Continuando con el resto del análisis...");
        }
    }

    // 7. Resultado final
    let total = *error_count.lock().unwrap();
    if total > 0 {
        println!("❌ Se encontraron {} violaciones arquitectónicas.", total);
        std::process::exit(1);
    } else {
        println!("✨ ¡Proyecto impecable! La arquitectura se respeta.");
        std::process::exit(0);
    }
}
