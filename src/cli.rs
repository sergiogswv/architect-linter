/// Módulo CLI - Funciones relacionadas con la interfaz de línea de comandos

use std::env;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Argumentos procesados de la línea de comandos
#[derive(Debug, Clone)]
pub struct CliArgs {
    /// Ruta del proyecto a analizar (None = modo interactivo)
    pub project_path: Option<String>,
    /// Activar modo watch
    pub watch_mode: bool,
    /// Activar modo fix (auto-reparación con IA)
    pub fix_mode: bool,
}

/// Muestra la ayuda del CLI
pub fn print_help() {
    println!("architect-linter {}", VERSION);
    println!();
    println!("Linter de arquitectura de software para proyectos TypeScript/JavaScript");
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
    println!("  -w, --watch      Modo watch: observa cambios y re-analiza automáticamente");
    println!("  -f, --fix        Modo fix: sugiere y aplica correcciones automáticas con IA");
    println!();
    println!("EJEMPLOS:");
    println!("  architect-linter                    # Modo interactivo");
    println!("  architect-linter .                  # Analizar directorio actual");
    println!("  architect-linter /ruta/a/proyecto   # Analizar proyecto específico");
    println!("  architect-linter --watch .          # Modo watch en directorio actual");
    println!("  architect-linter --fix .            # Analizar y auto-corregir con IA");
    println!();
    println!("DOCUMENTACIÓN:");
    println!("  https://github.com/sergio/architect-linter");
}

/// Muestra la versión del linter
pub fn print_version() {
    println!("architect-linter {}", VERSION);
}

/// Procesa los argumentos de línea de comandos
/// Retorna None si se procesó un flag especial (--help, --version)
/// Retorna Some(CliArgs) si hay que continuar con el análisis
pub fn process_args() -> Option<CliArgs> {
    let args: Vec<String> = env::args().collect();

    let mut watch_mode = false;
    let mut fix_mode = false;
    let mut project_path: Option<String> = None;

    // Procesar argumentos
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--version" | "-v" => {
                print_version();
                return None;
            }
            "--help" | "-h" => {
                print_help();
                return None;
            }
            "--watch" | "-w" => {
                watch_mode = true;
            }
            "--fix" | "-f" => {
                fix_mode = true;
            }
            _ => {
                // Si no es un flag, asumimos que es la ruta del proyecto
                if !args[i].starts_with('-') {
                    project_path = Some(args[i].clone());
                }
            }
        }
        i += 1;
    }

    Some(CliArgs {
        project_path,
        watch_mode,
        fix_mode,
    })
}
