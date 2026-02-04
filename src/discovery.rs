use crate::detector;
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

/// Recolecta todos los archivos .ts, .tsx, .js, .jsx que el linter debe analizar.
pub fn collect_files(root: &Path) -> Vec<PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| is_not_ignored(e))
        .filter_map(|e| e.ok())
        .filter(|e| {
            let path_str = e.path().to_string_lossy();
            e.path().extension().map_or(false, |ext| {
                ext == "ts" || ext == "tsx" || ext == "js" || ext == "jsx"
            }) && !path_str.ends_with(".d.ts") // Ignorar definiciones TypeScript
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

fn is_not_ignored(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| {
            s != "node_modules" && s != "dist" && s != ".git" && s != "target" && s != ".suggested"
        })
        .unwrap_or(false)
}

fn is_architectural_file(path: &Path) -> bool {
    let s = path.to_string_lossy().to_lowercase();
    s.ends_with(".controller.ts")
        || s.ends_with(".controller.js")
        || s.ends_with(".service.ts")
        || s.ends_with(".service.js")
        || s.ends_with(".entity.ts")
        || s.ends_with(".entity.js")
        || s.ends_with(".repository.ts")
        || s.ends_with(".repository.js")
        || s.ends_with(".dto.ts")
        || s.ends_with(".dto.js")
        || s.ends_with(".module.ts")
        || s.ends_with(".module.js")
        || s.ends_with(".handler.ts")
        || s.ends_with(".handler.js")
}
