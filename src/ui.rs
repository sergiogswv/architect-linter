use crate::ai::{AISuggestionResponse, SuggestedRule};
use crate::config::AIConfig;
use console::style;
use dialoguer::{theme::ColorfulTheme, Input, MultiSelect, Select};
use miette::{IntoDiagnostic, Result};
use std::env;
use std::path::PathBuf;

/// Imprime el banner de bienvenida con ASCII art y estilo de alto impacto
pub fn print_banner() {
    println!();
    println!(
        "{}",
        style(
            "╔══════════════════════════════════════════════════════════════════════════════════╗"
        )
        .cyan()
    );
    println!(
        "{}",
        style(
            r"
    ___    ____  ______ __  __________________  ______ ______ 
   /   |  / __ \/ ____// / / /  _/_  __/ ____/ / ____//_  __/ 
  / /| | / /_/ / /    / /_/ // /  / / / __/   / /      / /    
 / ___ |/ _, _/ /___ / __  // /  / / / /___  / /___   / /     
/_/  |_/_/ |_|\____//_/ /_/___/ /_/ /_____/  \____/  /_/      
                                                              
    __     _____  _   __ ______ ______ ____           
   / /    /  _/ / | / //_  __// ____// __ \          
  / /     / /  /  |/ /  / /  / __/  / /_/ /          
 / /___ _/ /  / /|  /  / /  / /___ / _, _/           
/_____//___/ /_/ |_/  /_/  /_____//_/ |_|            
"
        )
        .cyan()
        .bold()
    );
    println!(
        "{}",
        style(
            "╚══════════════════════════════════════════════════════════════════════════════════╝"
        )
        .cyan()
    );
    println!();
    println!(
        "{}",
        style("                 Manteniendo la arquitectura de tu código ⚡")
            .white()
            .bold()
    );
    println!();
}

/// Solicita al usuario la configuración de IA
pub fn ask_ai_config() -> Result<AIConfig> {
    println!("🤖 CONFIGURACIÓN DE LA IA");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Para analizar tu arquitectura con IA, necesitas configurar:");
    println!("  • URL de la API (ej: https://api.anthropic.com)");
    println!("  • API Key (tu token de autenticación)");
    println!("  • Modelo a usar (ej: claude-sonnet-4-5-20250929)");
    println!();

    // Verificar si existen variables de entorno para usar como defaults
    let default_url = env::var("ANTHROPIC_BASE_URL").ok();
    let default_key = env::var("ANTHROPIC_AUTH_TOKEN").ok();
    let default_model = env::var("ANTHROPIC_MODEL").ok();

    let api_url: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("URL de la API")
        .default(default_url.unwrap_or_else(|| "https://api.anthropic.com".to_string()))
        .interact_text()
        .into_diagnostic()?;

    let api_key: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("API Key")
        .default(default_key.unwrap_or_else(|| String::new()))
        .interact_text()
        .into_diagnostic()?;

    let model: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Modelo de IA")
        .default(default_model.unwrap_or_else(|| "claude-sonnet-4-5-20250929".to_string()))
        .interact_text()
        .into_diagnostic()?;

    println!("✅ Configuración de IA guardada.\n");

    Ok(AIConfig {
        api_url,
        api_key,
        model,
    })
}

/// Permite al usuario elegir qué reglas de las sugeridas por la IA desea aplicar.
pub fn ask_user_to_confirm_rules(
    suggestions: AISuggestionResponse,
) -> Result<(Vec<SuggestedRule>, usize)> {
    println!("\n🤖 El Arquitecto Virtual ha analizado tu proyecto.");
    println!(
        "\n🤖 El Arquitecto Virtual sugiere el patrón: {}",
        suggestions.pattern
    );

    let max_lines: usize = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Límite máximo de líneas por función sugerido")
        .default(suggestions.suggested_max_lines)
        .interact_text()
        .into_diagnostic()?;

    println!("Deseas aplicar las siguientes reglas de importación?\n");

    // Preparamos las etiquetas para el menú (Regla + Razón)
    let items: Vec<String> = suggestions
        .rules
        .iter()
        .map(|r| format!("{} -> {} \n   └─ Razón: {}", r.from, r.to, r.reason))
        .collect();

    // Por defecto, todas las reglas están marcadas (true)
    let defaults = vec![true; items.len()];

    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Usa [Espacio] para marcar/desmarcar y [Enter] para confirmar")
        .items(&items)
        .defaults(&defaults)
        .interact()
        .into_diagnostic()?;

    // Filtramos solo las reglas seleccionadas por el usuario
    let mut selected_rules = Vec::new();
    for index in selections {
        selected_rules.push(suggestions.rules[index].clone());
    }

    Ok((selected_rules, max_lines))
}

pub fn get_interactive_path() -> Result<PathBuf> {
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
