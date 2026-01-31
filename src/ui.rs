use crate::ai::{AISuggestionResponse, SuggestedRule};
use dialoguer::{theme::ColorfulTheme, Input, MultiSelect, Select};
use miette::{IntoDiagnostic, Result};
use std::env;
use std::path::PathBuf;

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
