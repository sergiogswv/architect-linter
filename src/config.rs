#![allow(unused_assignments)]

use crate::ai::SuggestedRule;
use miette::{Diagnostic, IntoDiagnostic, Result, SourceSpan};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Framework {
    NestJS,
    React,
    Angular,
    Express,
    Unknown,
}

impl Framework {
    pub fn as_str(&self) -> &str {
        match self {
            Framework::NestJS => "NestJS",
            Framework::React => "React",
            Framework::Angular => "Angular",
            Framework::Express => "Express",
            Framework::Unknown => "Unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ArchPattern {
    Hexagonal,
    Clean,
    MVC,
    Ninguno,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForbiddenRule {
    pub from: String,
    pub to: String,
}

/// Configuración de IA para análisis arquitectónico
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    pub api_url: String,
    pub api_key: String,
    pub model: String,
}

impl Default for AIConfig {
    fn default() -> Self {
        Self {
            api_url: "https://api.anthropic.com".to_string(),
            api_key: String::new(),
            model: "claude-sonnet-4-5-20250929".to_string(),
        }
    }
}

/// Estructura para mapear el architect.json tal cual está en el disco
#[derive(Debug, Serialize, Deserialize)]
struct ConfigFile {
    pub max_lines_per_function: usize,
    pub architecture_pattern: ArchPattern,
    pub forbidden_imports: Vec<ForbiddenRule>,
    #[serde(default = "default_ignored_paths")]
    pub ignored_paths: Vec<String>,
}

/// Valores por defecto para ignored_paths
fn default_ignored_paths() -> Vec<String> {
    vec![
        "node_modules/".to_string(),
        "dist/".to_string(),
        "build/".to_string(),
        ".git/".to_string(),
        "coverage/".to_string(),
        ".next/".to_string(),
        "out/".to_string(),
        ".nuxt/".to_string(),
        ".output/".to_string(),
        ".vite/".to_string(),         // Vite cache
        ".turbo/".to_string(),        // Turborepo cache
        ".parcel-cache/".to_string(), // Parcel cache
        ".cache/".to_string(),        // Generic cache
        "target/".to_string(),        // Rust
        "__pycache__/".to_string(),   // Python
        ".vscode/".to_string(),       // VSCode
        ".idea/".to_string(),         // IntelliJ
    ]
}

/// Estructura para el archivo de configuración de IA (separado y privado)
#[derive(Debug, Serialize, Deserialize)]
struct AIConfigFile {
    pub api_url: String,
    pub api_key: String,
    pub model: String,
}

pub struct LinterContext {
    pub max_lines: usize,
    #[allow(dead_code)]
    pub framework: Framework,
    #[allow(dead_code)]
    pub pattern: ArchPattern,
    pub forbidden_imports: Vec<ForbiddenRule>,
    pub ignored_paths: Vec<String>,
    #[allow(dead_code)]
    pub ai_config: Option<AIConfig>,
}

/// CARGA SILENCIOSA: Lee architect.json y .architect.ai.json y los convierte en contexto
pub fn load_config(root: &Path) -> Result<LinterContext> {
    let config_path = root.join("architect.json");

    // Leer el archivo de reglas
    let content = fs::read_to_string(&config_path)
        .map_err(|e| ConfigError::new(
            format!("No se pudo leer architect.json: {}", e),
            format!("Asegúrate de que el archivo existe en: {}", config_path.display())
        ))?;

    // Validar que es JSON válido
    let json_value: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| ConfigError::new(
            format!("JSON inválido: {}", e),
            "Verifica que el archivo architect.json tenga sintaxis JSON válida. Usa un validador JSON online si es necesario.".to_string()
        ))?;

    // Validar el esquema antes de deserializar
    validate_schema(&json_value)?;

    // Ahora sí deserializar con mejor manejo de errores
    let config: ConfigFile = serde_json::from_value(json_value)
        .map_err(|e| ConfigError::new(
            format!("Error en la estructura: {}", e),
            "Revisa que todos los campos tengan el tipo correcto.".to_string()
        ))?;

    // Validar los valores
    validate_config_values(&config)?;

    // Cargar configuración de IA (si existe, es opcional)
    let ai_config_path = root.join(".architect.ai.json");
    let ai_config = if ai_config_path.exists() {
        let ai_content = fs::read_to_string(&ai_config_path).into_diagnostic()?;
        let ai_file: AIConfigFile = serde_json::from_str(&ai_content).into_diagnostic()?;
        Some(AIConfig {
            api_url: ai_file.api_url,
            api_key: ai_file.api_key,
            model: ai_file.model,
        })
    } else {
        None
    };

    // Re-detectamos el framework para el contexto actual
    let framework = crate::detector::detect_framework(root);

    Ok(LinterContext {
        max_lines: config.max_lines_per_function,
        framework,
        pattern: config.architecture_pattern,
        forbidden_imports: config.forbidden_imports,
        ignored_paths: config.ignored_paths,
        ai_config,
    })
}

/// Valida que el JSON tenga todos los campos requeridos
fn validate_schema(json: &serde_json::Value) -> Result<()> {
    let obj = json.as_object().ok_or_else(|| ConfigError::new(
        "El archivo debe ser un objeto JSON".to_string(),
        "El archivo architect.json debe empezar y terminar con llaves { }".to_string()
    ))?;

    // Validar campo: max_lines_per_function
    if !obj.contains_key("max_lines_per_function") {
        return Err(ConfigError::new(
            "Falta el campo requerido: max_lines_per_function".to_string(),
            "Agrega este campo con un número, ejemplo: \"max_lines_per_function\": 40".to_string()
        ).into());
    }

    if !obj["max_lines_per_function"].is_number() {
        return Err(ConfigError::new(
            "El campo 'max_lines_per_function' debe ser un número".to_string(),
            "Ejemplo correcto: \"max_lines_per_function\": 40".to_string()
        ).into());
    }

    // Validar campo: architecture_pattern
    if !obj.contains_key("architecture_pattern") {
        return Err(ConfigError::new(
            "Falta el campo requerido: architecture_pattern".to_string(),
            "Agrega este campo. Valores válidos: \"Hexagonal\", \"Clean\", \"MVC\", \"Ninguno\"".to_string()
        ).into());
    }

    if !obj["architecture_pattern"].is_string() {
        return Err(ConfigError::new(
            "El campo 'architecture_pattern' debe ser un string".to_string(),
            "Valores válidos: \"Hexagonal\", \"Clean\", \"MVC\", \"Ninguno\"".to_string()
        ).into());
    }

    // Validar que el patrón sea uno de los valores aceptados
    let pattern_str = obj["architecture_pattern"].as_str().unwrap();
    let valid_patterns = ["Hexagonal", "Clean", "MVC", "Ninguno"];
    if !valid_patterns.contains(&pattern_str) {
        return Err(ConfigError::new(
            format!("Patrón arquitectónico inválido: '{}'", pattern_str),
            format!("Valores válidos: {}", valid_patterns.join(", "))
        ).into());
    }

    // Validar campo: forbidden_imports
    if !obj.contains_key("forbidden_imports") {
        return Err(ConfigError::new(
            "Falta el campo requerido: forbidden_imports".to_string(),
            "Agrega este campo como un array, ejemplo: \"forbidden_imports\": []".to_string()
        ).into());
    }

    if !obj["forbidden_imports"].is_array() {
        return Err(ConfigError::new(
            "El campo 'forbidden_imports' debe ser un array".to_string(),
            "Ejemplo: \"forbidden_imports\": [{\"from\": \"src/components/**\", \"to\": \"src/services/**\"}]".to_string()
        ).into());
    }

    // Validar cada regla en forbidden_imports
    let rules = obj["forbidden_imports"].as_array().unwrap();
    for (index, rule) in rules.iter().enumerate() {
        if !rule.is_object() {
            return Err(ConfigError::new(
                format!("La regla #{} en forbidden_imports no es un objeto", index + 1),
                "Cada regla debe tener la forma: {\"from\": \"patrón1\", \"to\": \"patrón2\"}".to_string()
            ).into());
        }

        let rule_obj = rule.as_object().unwrap();

        if !rule_obj.contains_key("from") {
            return Err(ConfigError::new(
                format!("La regla #{} no tiene el campo 'from'", index + 1),
                "Ejemplo: {\"from\": \"src/components/**\", \"to\": \"src/services/**\"}".to_string()
            ).into());
        }

        if !rule_obj.contains_key("to") {
            return Err(ConfigError::new(
                format!("La regla #{} no tiene el campo 'to'", index + 1),
                "Ejemplo: {\"from\": \"src/components/**\", \"to\": \"src/services/**\"}".to_string()
            ).into());
        }

        if !rule_obj["from"].is_string() {
            return Err(ConfigError::new(
                format!("El campo 'from' en la regla #{} debe ser un string", index + 1),
                "Ejemplo: \"from\": \"src/components/**\"".to_string()
            ).into());
        }

        if !rule_obj["to"].is_string() {
            return Err(ConfigError::new(
                format!("El campo 'to' en la regla #{} debe ser un string", index + 1),
                "Ejemplo: \"to\": \"src/services/**\"".to_string()
            ).into());
        }
    }

    Ok(())
}

/// Valida los valores de la configuración (rangos, lógica, etc.)
fn validate_config_values(config: &ConfigFile) -> Result<()> {
    // Validar que max_lines_per_function esté en un rango razonable
    if config.max_lines_per_function == 0 {
        return Err(ConfigError::new(
            "max_lines_per_function no puede ser 0".to_string(),
            "Usa un valor entre 10 y 500. Recomendado: 20-60 según tu framework.".to_string()
        ).into());
    }

    if config.max_lines_per_function > 1000 {
        return Err(ConfigError::new(
            format!("max_lines_per_function es muy alto: {}", config.max_lines_per_function),
            "Un valor tan alto desactiva efectivamente esta validación. Máximo recomendado: 500".to_string()
        ).into());
    }

    // Validar que forbidden_imports tenga reglas únicas (no duplicadas)
    for (i, rule1) in config.forbidden_imports.iter().enumerate() {
        for (j, rule2) in config.forbidden_imports.iter().enumerate() {
            if i != j && rule1.from == rule2.from && rule1.to == rule2.to {
                return Err(ConfigError::new(
                    format!("Regla duplicada: from '{}' to '{}'", rule1.from, rule1.to),
                    "Elimina una de las reglas duplicadas en forbidden_imports.".to_string()
                ).into());
            }
        }
    }

    // Advertencia si no hay reglas (aunque técnicamente válido)
    if config.forbidden_imports.is_empty() {
        eprintln!("⚠️  Advertencia: No hay reglas en forbidden_imports. El linter solo validará la longitud de funciones.");
    }

    Ok(())
}

/// Orquestador de configuración: Carga silenciosa o Wizard con IA
pub fn setup_or_load_config(root: &Path) -> Result<std::sync::Arc<LinterContext>> {
    use std::sync::Arc;

    let config_path = root.join("architect.json");

    if config_path.exists() {
        // MODO AUTOMÁTICO: carga silenciosa
        let ctx = load_config(root)?;
        return Ok(Arc::new(ctx));
    }

    // MODO CONFIGURACIÓN (IA Discovery)
    println!("📝 No encontré 'architect.json'. Iniciando descubrimiento asistido por IA...\n");

    // 0. Pedir configuración de IA si no existe
    let ai_config = crate::ui::ask_ai_config()?;

    // 1. Discovery (Input local)
    let project_info = crate::discovery::get_architecture_snapshot(root);

    // 2. IA (Procesamiento inteligente)
    let suggestions = crate::ai::sugerir_arquitectura_inicial(project_info, ai_config.clone())
        .map_err(|e| miette::miette!("Error consultando la IA: {}", e))?;

    // 3. UI (Wizard de confirmación)
    let (selected_rules, max_lines) = crate::ui::ask_user_to_confirm_rules(suggestions)?;

    // 4. Config (Persistencia)
    let final_ctx = save_config_from_wizard(root, selected_rules, max_lines, Some(ai_config))?;

    println!("✅ Configuración guardada exitosamente.\n");
    Ok(Arc::new(final_ctx))
}

/// PERSISTENCIA: Guarda las reglas de la IA y devuelve el contexto nuevo
pub fn save_config_from_wizard(
    root: &Path,
    rules: Vec<SuggestedRule>,
    max_lines: usize,
    ai_config: Option<AIConfig>,
) -> Result<LinterContext> {
    // 1. Guardar architect.json (reglas - compartible en el repo)
    let config_path = root.join("architect.json");

    // Convertimos de SuggestedRule (IA) a ForbiddenRule (Linter)
    let forbidden_imports: Vec<ForbiddenRule> = rules
        .into_iter()
        .map(|r| ForbiddenRule {
            from: r.from,
            to: r.to,
        })
        .collect();

    let framework = crate::detector::detect_framework(root);

    // Obtener ignored_paths según el framework
    let ignored_paths = get_framework_ignored_paths(&framework);

    // Valores por defecto para el primer architect.json
    let config = ConfigFile {
        max_lines_per_function: max_lines,
        architecture_pattern: ArchPattern::MVC, // O el que detecte la IA
        forbidden_imports: forbidden_imports.clone(),
        ignored_paths: ignored_paths.clone(),
    };

    let json = serde_json::to_string_pretty(&config).into_diagnostic()?;
    fs::write(&config_path, json).into_diagnostic()?;

    // 2. Guardar .architect.ai.json (config de IA - PRIVADO, en .gitignore)
    if let Some(ai_cfg) = ai_config.clone() {
        let ai_config_path = root.join(".architect.ai.json");
        let ai_config_file = AIConfigFile {
            api_url: ai_cfg.api_url,
            api_key: ai_cfg.api_key,
            model: ai_cfg.model,
        };
        let ai_json = serde_json::to_string_pretty(&ai_config_file).into_diagnostic()?;
        fs::write(&ai_config_path, ai_json).into_diagnostic()?;

        println!("🔐 Configuración de IA guardada en: .architect.ai.json");
        println!("⚠️  Este archivo contiene API keys y NO debe ser compartido en el repositorio.");

        // Actualizar .gitignore automáticamente
        update_gitignore(root)?;

        println!();
    }

    // Instalar husky y pre-commit hook después de guardar la configuración
    setup_husky_pre_commit(root)?;

    Ok(LinterContext {
        max_lines: config.max_lines_per_function,
        framework,
        pattern: config.architecture_pattern,
        forbidden_imports,
        ignored_paths,
        ai_config,
    })
}

/// Obtiene los patrones de exclusión según el framework detectado
pub fn get_framework_ignored_paths(framework: &Framework) -> Vec<String> {
    let mut paths = vec![
        "node_modules/".to_string(),
        ".git/".to_string(),
        "coverage/".to_string(),
        ".vscode/".to_string(),
        ".idea/".to_string(),
    ];

    match framework {
        Framework::React => {
            paths.extend(vec![
                "build/".to_string(),
                "dist/".to_string(),
                ".next/".to_string(),         // Next.js
                "out/".to_string(),
                ".vite/".to_string(),         // Vite
                ".turbo/".to_string(),        // Turborepo
                ".parcel-cache/".to_string(), // Parcel
            ]);
        }
        Framework::NestJS => {
            paths.extend(vec![
                "dist/".to_string(),
                "build/".to_string(),
            ]);
        }
        Framework::Angular => {
            paths.extend(vec![
                "dist/".to_string(),
                ".angular/".to_string(),
            ]);
        }
        Framework::Express => {
            paths.extend(vec![
                "dist/".to_string(),
                "build/".to_string(),
            ]);
        }
        Framework::Unknown => {
            paths.extend(vec![
                "dist/".to_string(),
                "build/".to_string(),
                "out/".to_string(),
                ".vite/".to_string(),
                ".cache/".to_string(),
            ]);
        }
    }

    paths
}

/// Configura husky y el hook pre-commit en el proyecto destino
fn setup_husky_pre_commit(root: &Path) -> Result<()> {
    use std::process::Command;

    let package_json_path = root.join("package.json");

    // Verificar si el proyecto tiene package.json
    if !package_json_path.exists() {
        println!("⚠️  No se encontró package.json, omitiendo configuración de husky.");
        return Ok(());
    }

    println!("🔧 Configurando husky y pre-commit hook...");

    // Ejecutar npx husky-init
    let husky_init_output = Command::new("npx")
        .args(["husky-init"])
        .current_dir(root)
        .output();

    match husky_init_output {
        Ok(output) => {
            if output.status.success() {
                println!("✅ husky-init ejecutado correctamente.");

                // Crear el hook pre-commit
                let pre_commit_path = root.join(".husky").join("pre-commit");
                let pre_commit_content = r#"#!/bin/sh
. "$(dirname "$0")/_/husky.sh"

echo "🏗️  Ejecutando Architect Linter..."

# Ejecutar architect-linter en el directorio actual (.)
architect-linter .

# Si el linter encuentra errores, el commit se cancelará
if [ $? -ne 0 ]; then
  echo ""
  echo "❌ El commit fue cancelado debido a violaciones de arquitectura"
  echo "💡 Corrige los errores reportados arriba y vuelve a intentar el commit"
  exit 1
fi

echo "✅ Validación de arquitectura exitosa"
exit 0
"#.to_string();

                // Escribir el hook
                fs::write(&pre_commit_path, pre_commit_content).into_diagnostic()?;

                // Dar permisos de ejecución al hook (Unix-like systems)
                #[cfg(unix)]
                {
                    let _ = Command::new("chmod")
                        .args(["+x", pre_commit_path.to_str().unwrap()])
                        .status();
                }

                // Crear también versión para Windows si es necesario
                #[cfg(windows)]
                {
                    let pre_commit_bat = root.join(".husky").join("pre-commit.bat");
                    let pre_commit_bat_content = r#"@echo off
echo 🏗️  Ejecutando Architect Linter...

REM Ejecutar architect-linter en el directorio actual (.)
architect-linter .

REM Si el linter encuentra errores, el commit se cancelará
if errorlevel 1 (
    echo.
    echo ❌ El commit fue cancelado debido a violaciones de arquitectura
    echo 💡 Corrige los errores reportados arriba y vuelve a intentar el commit
    exit /b 1
)

echo ✅ Validación de arquitectura exitosa
exit /b 0
"#.to_string();
                    let _ = fs::write(&pre_commit_bat, pre_commit_bat_content);
                }

                println!("✅ Hook pre-commit configurado exitosamente.");
                println!("💡 Ahora architect-linter se ejecutará automáticamente en cada commit.");
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                println!("⚠️  Error al ejecutar husky-init: {}", stderr);
                println!("💡 Puedes configurar husky manualmente con: npx husky-init");
            }
        }
        Err(e) => {
            println!("⚠️  No se pudo ejecutar npx husky-init: {}", e);
            println!("💡 Asegúrate de tener Node.js y npm instalados.");
            println!("💡 Para configurar husky manualmente, ejecuta: npx husky-init");
        }
    }

    Ok(())
}

/// Actualiza el .gitignore del proyecto para incluir .architect.ai.json
fn update_gitignore(root: &Path) -> Result<()> {
    let gitignore_path = root.join(".gitignore");
    let entry_to_add = "# Architect Linter - AI Configuration (contains API keys)\n.architect.ai.json";

    // Verificar si el archivo existe
    if gitignore_path.exists() {
        // Leer el contenido actual
        let content = fs::read_to_string(&gitignore_path).into_diagnostic()?;

        // Verificar si ya contiene la entrada
        if content.contains(".architect.ai.json") {
            println!("✅ .architect.ai.json ya está en el .gitignore");
            return Ok(());
        }

        // Agregar la entrada al final
        let mut updated_content = content;
        if !updated_content.ends_with('\n') {
            updated_content.push('\n');
        }
        updated_content.push_str(entry_to_add);
        updated_content.push('\n');

        fs::write(&gitignore_path, updated_content).into_diagnostic()?;
        println!("✅ .architect.ai.json agregado al .gitignore");
    } else {
        // Crear el .gitignore con la entrada
        let mut gitignore_content = String::from("# Architect Linter - AI Configuration (contains API keys)\n");
        gitignore_content.push_str(".architect.ai.json\n");

        fs::write(&gitignore_path, gitignore_content).into_diagnostic()?;
        println!("✅ .gitignore creado con .architect.ai.json");
    }

    Ok(())
}

#[derive(Error, Debug, Diagnostic)]
#[error("Violación de Arquitectura")]
#[diagnostic(code(arch::violation), severity(error))]
pub struct ArchError {
    #[source_code]
    pub src: String,
    #[label("{message}")]
    pub span: SourceSpan,
    pub message: String,
}

#[derive(Error, Debug, Diagnostic)]
#[error("{details}")]
#[diagnostic(code(config::invalid), severity(error))]
pub struct ConfigError {
    #[help]
    pub help: String,
    pub details: String,
}

impl ConfigError {
    pub fn new(details: String, help: String) -> Self {
        Self { details, help }
    }
}
