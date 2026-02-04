/// Módulo CLI - Funciones relacionadas con la interfaz de línea de comandos

use std::env;

const VERSION: &str = env!("CARGO_PKG_VERSION");

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
    println!();
    println!("EJEMPLOS:");
    println!("  architect-linter                    # Modo interactivo");
    println!("  architect-linter .                  # Analizar directorio actual");
    println!("  architect-linter /ruta/a/proyecto   # Analizar proyecto específico");
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
/// Retorna Some(args) si hay que continuar con el análisis
pub fn process_args() -> Option<Vec<String>> {
    let args: Vec<String> = env::args().collect();

    // Manejo de flags especiales
    if args.len() > 1 {
        match args[1].as_str() {
            "--version" | "-v" => {
                print_version();
                return None;
            }
            "--help" | "-h" => {
                print_help();
                return None;
            }
            _ => {}
        }
    }

    Some(args)
}
