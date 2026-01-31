mod analyzer;
mod config;
mod detector;

use dialoguer::{theme::ColorfulTheme, Input, Select};
use indicatif::{ProgressBar, ProgressStyle};
use miette::{GraphicalReportHandler, IntoDiagnostic, Result};
use rayon::prelude::*;
use std::env;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use swc_common::SourceMap;
use walkdir::WalkDir;

// Importamos lo que definiremos en config.rs
use crate::config::{ArchPattern, Framework, LinterContext};

fn main() -> Result<()> {
    println!("🏛️  WELCOME TO ARCHITECT-LINTER");

    // 1. Obtener la ruta del proyecto
    let args: Vec<String> = env::args().collect();
    let project_root = if args.len() > 1 {
        PathBuf::from(&args[1]).canonicalize().into_diagnostic()?
    } else {
        get_interactive_path()?
    };

    // 2. Cargar o crear configuración
    let ctx = setup_or_load_config(&project_root)?;

    // 3. Recolectar archivos .ts
    let files = collect_files(&project_root);
    if files.is_empty() {
        println!("✅ No se encontraron archivos .ts.");
        return Ok(());
    }

    // 4. Barra de progreso y Análisis
    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len}")
            .into_diagnostic()?,
    );

    let error_count = Arc::new(Mutex::new(0));

    files.par_iter().for_each(|file_path| {
        let cm = Arc::new(SourceMap::default());

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
        println!("❌ Se encontraron {} violaciones.", total);
        std::process::exit(1);
    } else {
        println!("✨ ¡Proyecto impecable!");
        std::process::exit(0);
    }
}

// Nueva función que maneja la lógica que pediste
fn setup_or_load_config(root: &PathBuf) -> Result<Arc<LinterContext>> {
    let config_path = root.join("architect.json");

    if config_path.exists() {
        // MODO AUTOMÁTICO: Carga silenciosa
        let content = std::fs::read_to_string(config_path).into_diagnostic()?;
        let json: serde_json::Value = serde_json::from_str(&content).into_diagnostic()?;

        let framework = detector::detect_framework(root);
        let max_lines = json["max_lines_per_function"].as_u64().unwrap_or(40) as usize;
        let pattern_str = json["architecture_pattern"].as_str().unwrap_or("MVC");

        let pattern = match pattern_str {
            "Hexagonal" => ArchPattern::Hexagonal,
            "Clean" => ArchPattern::Clean,
            _ => ArchPattern::MVC,
        };

        // Cargar las forbidden_imports del JSON si existen
        let forbidden_imports = if let Some(rules) = json["forbidden_imports"].as_array() {
            rules
                .iter()
                .filter_map(|rule| {
                    Some(crate::config::ForbiddenRule {
                        from: rule["from"].as_str()?.to_string(),
                        to: rule["to"].as_str()?.to_string(),
                    })
                })
                .collect()
        } else {
            Vec::new()
        };

        return Ok(Arc::new(LinterContext {
            max_lines,
            framework,
            pattern,
            forbidden_imports,
        }));
    }

    // MODO CONFIGURACIÓN: Preguntar antes de crear
    println!("📝 No encontré 'architect.json'. Vamos a configurar tu proyecto.");

    // A. Confirmar Framework
    let detected_fw = detector::detect_framework(root);
    let fw_options = vec!["NestJS", "React", "Angular", "Express", "Unknown"];
    let fw_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "Confirmar Framework (Detectado: {:?})",
            detected_fw
        ))
        .items(&fw_options)
        .default(0)
        .interact()
        .into_diagnostic()?;

    let framework = match fw_idx {
        0 => Framework::NestJS,
        1 => Framework::React,
        2 => Framework::Angular,
        3 => Framework::Express,
        _ => Framework::Unknown,
    };

    // B. Seleccionar Arquitectura
    let arch_options = vec!["Hexagonal", "Clean", "MVC", "Ninguno"];
    let arch_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("¿Qué patrón arquitectónico quieres aplicar?")
        .items(&arch_options)
        .default(2) // MVC por defecto
        .interact()
        .into_diagnostic()?;

    let pattern = match arch_idx {
        0 => ArchPattern::Hexagonal,
        1 => ArchPattern::Clean,
        2 => ArchPattern::MVC,
        _ => ArchPattern::Ninguno,
    };

    // C. Líneas de código
    let suggestion = detector::get_loc_suggestion(&framework);
    let max_lines: usize = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Límite de líneas por método")
        .default(suggestion)
        .interact()
        .into_diagnostic()?;

    // GUARDAR JSON
    let final_config = serde_json::json!({
        "max_lines_per_function": max_lines,
        "architecture_pattern": format!("{:?}", pattern),
        "forbidden_imports": []
    });

    let json_str = serde_json::to_string_pretty(&final_config).into_diagnostic()?;
    std::fs::write(&config_path, json_str).into_diagnostic()?;
    println!("✅ Configuración guardada en 'architect.json'\n");

    Ok(Arc::new(LinterContext {
        max_lines,
        framework,
        pattern,
        forbidden_imports: Vec::new(),
    }))
}

fn collect_files(root: &PathBuf) -> Vec<PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| {
            !["node_modules", "dist", ".git", "target"]
                .contains(&e.file_name().to_str().unwrap_or(""))
        })
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "ts"))
        .map(|e| e.path().to_path_buf())
        .collect()
}

fn get_interactive_path() -> Result<PathBuf> {
    let current_dir = env::current_dir().into_diagnostic()?;
    let search_dir = current_dir.parent().unwrap_or(&current_dir);
    let projects: Vec<PathBuf> = std::fs::read_dir(search_dir)
        .into_diagnostic()?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir() && p.join("package.json").exists())
        .collect();

    let mut options: Vec<String> = projects
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    options.push(">> Ingresar ruta manualmente...".into());

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Selecciona proyecto")
        .items(&options)
        .interact()
        .into_diagnostic()?;

    if selection == options.len() - 1 {
        let path: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Ruta completa")
            .interact_text()
            .into_diagnostic()?;
        Ok(PathBuf::from(path))
    } else {
        Ok(projects[selection].clone())
    }
}

