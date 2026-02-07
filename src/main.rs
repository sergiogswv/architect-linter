use indicatif::{ProgressBar, ProgressStyle};
use miette::{GraphicalReportHandler, IntoDiagnostic, Result};
use rayon::prelude::*;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use swc_common::SourceMap;

mod ai;
mod analyzer;
mod autofix;
mod circular;
mod cli;
mod config;
mod detector;
mod discovery;
mod parsers;
mod ui;
mod watch;

fn main() -> Result<()> {
    // 1. Procesar argumentos de línea de comandos
    let cli_args = match cli::process_args() {
        Some(args) => args,
        None => return Ok(()), // Se procesó --help o --version
    };

    ui::print_banner();

    // 2. Obtener la ruta del proyecto
    let project_root = if let Some(path) = cli_args.project_path {
        PathBuf::from(&path).canonicalize().into_diagnostic()?
    } else {
        ui::get_interactive_path()?
    };

    // 3. Cargar o crear configuración asistida por IA
    let ctx = Arc::new(config::setup_or_load_config(&project_root)?);

    // 4. Decidir entre modo normal, watch o fix
    if cli_args.fix_mode {
        run_fix_mode(&project_root, Arc::clone(&ctx))?;
    } else if cli_args.watch_mode {
        run_watch_mode(&project_root, Arc::clone(&ctx))?;
    } else {
        run_normal_mode(&project_root, Arc::clone(&ctx))?;
    }

    Ok(())
}

/// Ejecuta el análisis en modo normal (una sola vez)
fn run_normal_mode(project_root: &PathBuf, ctx: Arc<config::LinterContext>) -> Result<()> {
    // Recolectar archivos de todos los lenguajes soportados
    let files = discovery::collect_files(project_root, &ctx.ignored_paths);

    // Mostrar información de directorios ignorados
    if !ctx.ignored_paths.is_empty() {
        println!("📂 Ignorando directorios: {}", ctx.ignored_paths.join(", "));
    }

    if files.is_empty() {
        println!("✅ No se encontraron archivos para analizar (TypeScript, JavaScript, Python, Go, PHP, Java).");
        return Ok(());
    }

    // Barra de progreso y Análisis Paralelo con Rayon
    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .into_diagnostic()?,
    );

    let error_count = Arc::new(Mutex::new(0));
    let cm = Arc::new(SourceMap::default());

    files.par_iter().for_each(|file_path| {
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

    // Análisis de Dependencias Cíclicas
    println!("\n🔍 Analizando dependencias cíclicas...");
    let cycles = circular::analyze_circular_dependencies(&files, project_root, &cm);

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

    // Resultado final
    let total = *error_count.lock().unwrap();
    if total > 0 {
        println!("❌ Se encontraron {} violaciones arquitectónicas.", total);
        std::process::exit(1);
    } else {
        println!("✨ ¡Proyecto impecable! La arquitectura se respeta.");
        std::process::exit(0);
    }
}

/// Ejecuta el análisis en modo watch (observación continua)
fn run_watch_mode(project_root: &PathBuf, ctx: Arc<config::LinterContext>) -> Result<()> {
    println!("🚀 Iniciando modo watch...\n");

    // Análisis inicial completo
    let files = discovery::collect_files(project_root, &ctx.ignored_paths);

    // Mostrar información de directorios ignorados
    if !ctx.ignored_paths.is_empty() {
        println!("📂 Ignorando directorios: {}", ctx.ignored_paths.join(", "));
    }

    if files.is_empty() {
        println!("✅ No se encontraron archivos para analizar (TypeScript, JavaScript, Python, Go, PHP, Java).");
        return Ok(());
    }

    println!("📊 Análisis inicial de {} archivos...", files.len());
    let cm = Arc::new(SourceMap::default());

    // Construir grafo de dependencias inicial
    let mut analyzer = circular::CircularDependencyAnalyzer::new(project_root);
    analyzer.build_graph(&files, &cm)?;

    // Análisis inicial de violaciones
    let mut error_count = 0;
    for file_path in &files {
        if let Err(e) = analyzer::analyze_file(&cm, file_path, &ctx) {
            error_count += 1;
            let mut out = String::new();
            let _ = GraphicalReportHandler::new().render_report(&mut out, e.as_ref());
            println!("\n📌 Violación en: {}", file_path.display());
            println!("{}", out);
        }
    }

    // Análisis de ciclos inicial
    let cycles = analyzer.detect_cycles();
    if !cycles.is_empty() {
        circular::print_circular_dependency_report(&cycles);
        println!(
            "\n⚠️  Se encontraron {} dependencias cíclicas.",
            cycles.len()
        );
    }

    if error_count > 0 {
        println!(
            "\n❌ Se encontraron {} violaciones arquitectónicas.",
            error_count
        );
    } else {
        println!("\n✨ ¡Proyecto impecable! La arquitectura se respeta.");
    }

    // Iniciar observación de archivos
    let analyzer = Arc::new(Mutex::new(analyzer));
    let project_root_arc = Arc::new(project_root.clone());
    let ignored_paths = ctx.ignored_paths.clone();

    watch::start_watch_mode(project_root_arc.as_ref(), ignored_paths, |changed_files| {
        let analyzer = Arc::clone(&analyzer);
        let ctx = Arc::clone(&ctx);
        let cm = Arc::clone(&cm);
        let project_root = Arc::clone(&project_root_arc);

        // Re-analizar solo archivos cambiados
        let mut error_count = 0;
        for file_path in changed_files {
            // Validar reglas arquitectónicas
            if let Err(e) = analyzer::analyze_file(&cm, file_path, &ctx) {
                error_count += 1;
                let mut out = String::new();
                let _ = GraphicalReportHandler::new().render_report(&mut out, e.as_ref());
                println!("\n📌 Violación en: {}", file_path.display());
                println!("{}", out);
            }

            // Actualizar grafo de dependencias
            let mut analyzer = analyzer.lock().unwrap();
            if let Err(e) = analyzer.update_file(file_path, &cm) {
                eprintln!("⚠️  Error actualizando grafo: {}", e);
                continue;
            }

            // Análisis incremental de ciclos
            // Normalizar path relativo al proyecto
            let normalized_path =
                if let Ok(relative) = file_path.strip_prefix(project_root.as_ref()) {
                    relative.to_string_lossy().replace('\\', "/").to_lowercase()
                } else {
                    file_path
                        .to_string_lossy()
                        .replace('\\', "/")
                        .to_lowercase()
                };

            let affected_nodes = analyzer.get_affected_nodes(&normalized_path);

            if !affected_nodes.is_empty() {
                let cycles = analyzer.detect_cycles_in_subgraph(&affected_nodes);
                if !cycles.is_empty() {
                    circular::print_circular_dependency_report(&cycles);
                    println!(
                        "\n⚠️  Se encontraron {} dependencias cíclicas.",
                        cycles.len()
                    );
                }
            }
        }

        if error_count > 0 {
            println!(
                "\n❌ Se encontraron {} violaciones arquitectónicas.",
                error_count
            );
        } else {
            println!("\n✨ Todo correcto!");
        }

        Ok(())
    })?;

    Ok(())
}

/// Ejecuta el análisis en modo fix (auto-reparación con IA)
fn run_fix_mode(project_root: &PathBuf, ctx: Arc<config::LinterContext>) -> Result<()> {
    use dialoguer::Confirm;

    println!("🔧 Modo Fix: Auto-reparación con IA\n");

    // Verificar que hay configuración de IA
    if ctx.ai_configs.is_empty() {
        return Err(miette::miette!(
            "No se encontró configuración de IA (.architect.ai.json).\n\
             El modo --fix requiere configuración de IA para funcionar."
        ));
    }

    // Recolectar archivos
    let files = discovery::collect_files(project_root, &ctx.ignored_paths);

    if !ctx.ignored_paths.is_empty() {
        println!("📂 Ignorando directorios: {}", ctx.ignored_paths.join(", "));
    }

    if files.is_empty() {
        println!("✅ No se encontraron archivos para analizar (TypeScript, JavaScript, Python, Go, PHP, Java).");
        return Ok(());
    }

    println!("📊 Analizando {} archivos...\n", files.len());

    // Recolectar todas las violaciones
    let cm = Arc::new(SourceMap::default());
    let mut all_violations = Vec::new();

    for file_path in &files {
        match analyzer::collect_violations_from_file(&cm, file_path, &ctx) {
            Ok(violations) => {
                all_violations.extend(violations);
            }
            Err(e) => {
                eprintln!("⚠️  Error analizando {}: {}", file_path.display(), e);
            }
        }
    }

    if all_violations.is_empty() {
        println!("✨ ¡No se encontraron violaciones! Tu código está perfecto.");
        return Ok(());
    }

    println!(
        "🔍 Encontradas {} violación(es) arquitectónicas\n",
        all_violations.len()
    );

    // Procesar cada violación
    let mut fixed_count = 0;
    let mut skipped_count = 0;

    for (index, violation) in all_violations.iter().enumerate() {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Violación #{}/{}", index + 1, all_violations.len());
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📄 Archivo: {}", violation.file_path.display());
        println!("📍 Línea: {}", violation.line_number);
        println!(
            "🚫 Regla violada: '{}' no puede importar de '{}'",
            violation.rule.from, violation.rule.to
        );
        println!("💥 Import ofensivo: {}", violation.offensive_import);
        println!();

        // Consultar a la IA con fallback
        println!("🤖 Consultando sugerencia de fix (usando sistema de fallback multimodelo)...");

        let runtime = tokio::runtime::Runtime::new().into_diagnostic()?;
        let suggestion = match runtime.block_on(autofix::suggest_fix(
            violation,
            project_root,
            &ctx.ai_configs,
        )) {
            Ok(s) => s,
            Err(_e) => {
                eprintln!("❌ No se pudo obtener ninguna sugerencia de los modelos configurados.");
                println!("⏭️  Saltando esta violación...\n");
                skipped_count += 1;
                continue;
            }
        };

        // Mostrar la sugerencia
        println!();
        println!(
            "💡 Sugerencia de la IA (confianza: {}):",
            suggestion.confidence
        );
        println!("{}", suggestion.explanation);
        println!();

        match &suggestion.fix_type {
            autofix::FixType::Refactor { old_code, new_code } => {
                println!("📝 Tipo: Refactorización de código");
                println!("Cambiar:");
                println!("  ❌ {}", old_code);
                println!("Por:");
                println!("  ✅ {}", new_code);
            }
            autofix::FixType::MoveFile { from, to } => {
                println!("📦 Tipo: Mover archivo");
                println!("  De: {}", from);
                println!("  A:  {}", to);
            }
            autofix::FixType::CreateInterface {
                interface_path,
                interface_code,
                updated_import,
            } => {
                println!("🎯 Tipo: Crear interfaz");
                println!("  Nueva interfaz: {}", interface_path);
                println!("  Código: {} líneas", interface_code.lines().count());
                println!("  Nuevo import: {}", updated_import);
            }
        }

        println!();

        // Pedir confirmación
        let should_apply = Confirm::new()
            .with_prompt("¿Aplicar este fix?")
            .default(false)
            .interact()
            .into_diagnostic()?;

        if should_apply {
            match autofix::apply_fix(&suggestion, violation, project_root) {
                Ok(message) => {
                    println!("{}", message);
                    fixed_count += 1;
                }
                Err(e) => {
                    eprintln!("❌ Error aplicando fix: {}", e);
                    skipped_count += 1;
                }
            }
        } else {
            println!("⏭️  Fix omitido");
            skipped_count += 1;
        }

        println!();
    }

    // Resumen final
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 RESUMEN");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ Fixes aplicados: {}", fixed_count);
    println!("⏭️  Omitidos: {}", skipped_count);
    println!("📝 Total violaciones: {}", all_violations.len());
    println!();

    if fixed_count > 0 {
        println!("🎉 ¡Se aplicaron {} fix(es) exitosamente!", fixed_count);
        println!("💡 Tip: Ejecuta el linter nuevamente para verificar que todo esté correcto.");
    }

    Ok(())
}
