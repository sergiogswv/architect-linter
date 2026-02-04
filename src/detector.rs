use crate::config::Framework;
use std::fs;
use std::path::Path;

/// Analiza el package.json para determinar el framework del proyecto.
pub fn detect_framework(root: &Path) -> Framework {
    let pkg_path = root.join("package.json");

    // Si no hay package.json, no podemos saber qué es con certeza
    if !pkg_path.exists() {
        return Framework::Unknown;
    }

    // Leemos el contenido del package.json
    if let Ok(content) = fs::read_to_string(pkg_path) {
        // Buscamos firmas específicas en las dependencias
        if content.contains("@nestjs/core") {
            return Framework::NestJS;
        }
        if content.contains("\"react\"") {
            return Framework::React;
        }
        if content.contains("@angular/core") {
            return Framework::Angular;
        }
        if content.contains("\"express\"") {
            return Framework::Express;
        }
    }

    Framework::Unknown
}

/// Sugiere un límite de líneas de código (LOC) basado en el framework detectado.
#[allow(dead_code)]
pub fn get_loc_suggestion(framework: &Framework) -> usize {
    match framework {
        Framework::NestJS => 40,  // Métodos de clase
        Framework::React => 25,   // Componentes funcionales pequeños
        Framework::Angular => 50, // Componentes con lógica más extensa
        Framework::Express => 60, // Middlewares y handlers
        Framework::Unknown => 50, // Estándar general
    }
}
