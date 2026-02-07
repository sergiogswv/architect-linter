use crate::detector;
use crate::parsers;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

#[derive(Serialize)]
pub struct ProjectContext {
    pub framework: String,
    pub dependencies: Vec<String>,
    pub folder_structure: Vec<String>,
    pub key_files: Vec<String>,
}

/// Recolecta todos los archivos soportados que el linter debe analizar.
/// Incluye: TypeScript (.ts, .tsx), JavaScript (.js, .jsx), Python (.py), Go (.go), PHP (.php), Java (.java)
/// Respeta los patrones de exclusión definidos en ignored_paths.
pub fn collect_files(root: &Path, ignored_paths: &[String]) -> Vec<PathBuf> {
    let supported_exts = parsers::supported_extensions();

    WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| is_not_ignored_with_patterns(e, root, ignored_paths))
        .filter_map(|e| e.ok())
        .filter(|e| {
            let path_str = e.path().to_string_lossy();

            // Ignorar archivos de definición TypeScript
            if path_str.ends_with(".d.ts") {
                return false;
            }

            // Verificar si la extensión está en la lista de soportadas
            e.path().extension().map_or(false, |ext| {
                supported_exts.iter().any(|&supported| ext == supported)
            })
        })
        .map(|e| e.path().to_path_buf())
        .collect()
}

/// Genera un resumen completo del proyecto para que la IA tome decisiones arquitectónicas.
pub fn get_architecture_snapshot(root: &Path) -> ProjectContext {
    let mut folders = Vec::new();
    let mut key_files = Vec::new();

    let framework_enum = detector::detect_framework(root);
    let framework = framework_enum.as_str().to_string();
    let dependencies = get_dependency_list(root);

    let src_path = root.join("src");
    let scan_path = if src_path.exists() {
        src_path
    } else {
        root.to_path_buf()
    };

    let walker = WalkDir::new(scan_path)
        .into_iter()
        .filter_entry(|e| is_not_ignored(e));

    for entry in walker.filter_map(|e| e.ok()) {
        let path = entry.path();
        let relative_path = path.strip_prefix(root).unwrap_or(path);
        let path_str = relative_path.to_string_lossy().into_owned();

        if path.is_dir() {
            if path_str != "." && !path_str.is_empty() {
                folders.push(path_str);
            }
        } else if is_architectural_file(path) {
            key_files.push(path_str);
        }
    }

    ProjectContext {
        framework,
        dependencies,
        folder_structure: folders,
        key_files,
    }
}

fn get_dependency_list(root: &Path) -> Vec<String> {
    let pkg_path = root.join("package.json");
    let mut deps_list = Vec::new();

    if let Ok(content) = fs::read_to_string(pkg_path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(deps) = json.get("dependencies").and_then(|d| d.as_object()) {
                deps_list.extend(deps.keys().cloned()); // Simplificado .cloned()
            }
            if let Some(dev_deps) = json.get("devDependencies").and_then(|d| d.as_object()) {
                deps_list.extend(dev_deps.keys().cloned());
            }
        }
    }
    deps_list
}

/// Función de compatibilidad para discovery sin patrones personalizados
fn is_not_ignored(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| {
            s != "node_modules" && s != "dist" && s != ".git" && s != "target" && s != ".suggested"
        })
        .unwrap_or(false)
}

/// Verifica si una entrada debe ser ignorada según los patrones configurados
fn is_not_ignored_with_patterns(entry: &DirEntry, root: &Path, ignored_paths: &[String]) -> bool {
    // Obtener la ruta relativa al root del proyecto
    let entry_path = entry.path();
    let relative_path = entry_path
        .strip_prefix(root)
        .unwrap_or(entry_path)
        .to_string_lossy()
        .replace('\\', "/"); // Normalizar separadores para Windows

    // Verificar si la ruta coincide con algún patrón de exclusión
    for pattern in ignored_paths {
        let normalized_pattern = pattern.replace('\\', "/");

        // Coincidencia exacta o si la ruta comienza con el patrón
        if relative_path == normalized_pattern.trim_end_matches('/')
            || relative_path.starts_with(&normalized_pattern)
            || relative_path.starts_with(&format!("{}/", normalized_pattern.trim_end_matches('/')))
        {
            return false;
        }

        // También verificar el nombre del directorio/archivo directamente
        if let Some(file_name) = entry.file_name().to_str() {
            if file_name == normalized_pattern.trim_end_matches('/')
                || format!("{}/", file_name) == normalized_pattern
            {
                return false;
            }
        }
    }

    true
}

fn is_architectural_file(path: &Path) -> bool {
    let s = path.to_string_lossy().to_lowercase();

    // TypeScript/JavaScript patterns
    let ts_js_patterns = [
        ".controller.ts", ".controller.js",
        ".service.ts", ".service.js",
        ".entity.ts", ".entity.js",
        ".repository.ts", ".repository.js",
        ".dto.ts", ".dto.js",
        ".module.ts", ".module.js",
        ".handler.ts", ".handler.js",
    ];

    // Python patterns
    let py_patterns = [
        "_controller.py", "_service.py",
        "_repository.py", "_model.py",
        "_view.py", "_handler.py",
    ];

    // Go patterns
    let go_patterns = [
        "_controller.go", "_service.go",
        "_repository.go", "_handler.go",
        "_model.go",
    ];

    // PHP patterns
    let php_patterns = [
        "controller.php", "service.php",
        "repository.php", "model.php",
        "handler.php", "entity.php",
    ];

    // Java patterns
    let java_patterns = [
        "controller.java", "service.java",
        "repository.java", "model.java",
        "handler.java", "entity.java",
        "dao.java",
    ];

    ts_js_patterns.iter().any(|&pattern| s.ends_with(pattern))
        || py_patterns.iter().any(|&pattern| s.ends_with(pattern))
        || go_patterns.iter().any(|&pattern| s.ends_with(pattern))
        || php_patterns.iter().any(|&pattern| s.ends_with(pattern))
        || java_patterns.iter().any(|&pattern| s.ends_with(pattern))
}
