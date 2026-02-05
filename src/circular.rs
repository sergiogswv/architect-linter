use miette::{IntoDiagnostic, Result};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use swc_common::SourceMap;
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsConfig, EsConfig};

/// Representa una dependencia cíclica detectada
#[derive(Debug, Clone)]
pub struct CircularDependency {
    /// El ciclo completo de dependencias
    pub cycle: Vec<String>,
    /// Descripción legible del problema
    pub description: String,
}

/// Analizador de dependencias cíclicas
pub struct CircularDependencyAnalyzer {
    /// Grafo de dependencias: node -> [nodes que importa]
    graph: HashMap<String, Vec<String>>,
    /// Directorio raíz del proyecto
    project_root: PathBuf,
}

impl CircularDependencyAnalyzer {
    /// Crea un nuevo analizador de dependencias cíclicas
    pub fn new(project_root: &Path) -> Self {
        Self {
            graph: HashMap::new(),
            project_root: project_root.to_path_buf(),
        }
    }

    /// Analiza todos los archivos y construye el grafo de dependencias
    pub fn build_graph(&mut self, files: &[PathBuf], cm: &SourceMap) -> Result<()> {
        for file_path in files {
            // Extraer imports del archivo
            let imports = self.extract_imports(file_path, cm)?;

            // Normalizar la ruta del archivo actual
            let normalized_current = self.normalize_file_path(file_path);
            let current_key = normalized_current.clone();

            // Insertar en el grafo
            self.graph.entry(current_key.clone()).or_insert_with(Vec::new);

            // Procesar cada import
            for import_path in imports {
                if let Some(resolved) = self.resolve_import_path(file_path, &import_path) {
                    let normalized_import = self.normalize_file_path(&resolved);

                    // Solo agregar dependencias internas del proyecto
                    if self.is_internal_dependency(&normalized_import) {
                        self.graph
                            .entry(current_key.clone())
                            .or_insert_with(Vec::new)
                            .push(normalized_import);
                    }
                }
            }
        }

        Ok(())
    }

    /// Detecta todos los ciclos en el grafo de dependencias
    pub fn detect_cycles(&self) -> Vec<CircularDependency> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        for node in self.graph.keys() {
            if !visited.contains(node) {
                self.dfs_detect_cycles(
                    node,
                    &mut visited,
                    &mut rec_stack,
                    &mut path,
                    &mut cycles,
                );
            }
        }

        cycles
    }

    /// DFS para detectar ciclos en el grafo
    fn dfs_detect_cycles(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
        cycles: &mut Vec<CircularDependency>,
    ) {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());

        if let Some(neighbors) = self.graph.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    self.dfs_detect_cycles(neighbor, visited, rec_stack, path, cycles);
                } else if rec_stack.contains(neighbor) {
                    // Encontramos un ciclo
                    let cycle_start = path.iter().position(|x| x == neighbor).unwrap_or(0);
                    let mut cycle = path[cycle_start..].to_vec();
                    cycle.push(neighbor.clone());

                    cycles.push(CircularDependency {
                        cycle: cycle.clone(),
                        description: self.format_cycle_description(&cycle),
                    });
                }
            }
        }

        path.pop();
        rec_stack.remove(node);
    }

    /// Extrae todos los imports de un archivo
    fn extract_imports(&self, file_path: &Path, cm: &SourceMap) -> Result<Vec<String>> {
        let mut imports = Vec::new();

        // Parsear según la extensión
        let extension = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let syntax = match extension {
            "ts" | "tsx" => Syntax::Typescript(TsConfig {
                decorators: true,
                tsx: extension == "tsx",
                ..Default::default()
            }),
            "js" | "jsx" => Syntax::Es(EsConfig {
                decorators: true,
                jsx: extension == "jsx",
                ..Default::default()
            }),
            _ => Syntax::Typescript(TsConfig::default()),
        };

        let fm = cm.load_file(file_path).into_diagnostic()?;
        let lexer = Lexer::new(syntax, Default::default(), StringInput::from(&*fm), None);
        let mut parser = Parser::new_from(lexer);

        let module = parser
            .parse_module()
            .map_err(|e| miette::miette!("Error parsing {}: {:?}", file_path.display(), e))?;

        // Extraer imports estáticos
        for item in &module.body {
            if let swc_ecma_ast::ModuleItem::ModuleDecl(swc_ecma_ast::ModuleDecl::Import(
                import,
            )) = item
            {
                imports.push(import.src.value.to_string());
            }
        }

        Ok(imports)
    }

    /// Resuelve un path de import a una ruta de archivo real
    fn resolve_import_path(&self, current_file: &Path, import_path: &str) -> Option<PathBuf> {
        // Ignorar imports externos (node_modules, @/aliases si no se resuelven, etc.)
        if import_path.starts_with('@')
            || import_path.starts_with("node_modules")
            || (!import_path.starts_with('.') && !import_path.starts_with('/'))
        {
            // Podríamos agregar lógica para resolver alias de TypeScript aquí
            // Por ahora, solo procesamos imports relativos
            return None;
        }

        // Resolver path relativo
        let current_dir = current_file.parent()?;
        let resolved = current_dir.join(import_path);

        // Intentar diferentes extensiones
        let extensions = ["ts", "tsx", "js", "jsx"];
        for ext in &extensions {
            let with_ext = resolved.with_extension(ext);
            if with_ext.exists() {
                return Some(with_ext);
            }
        }

        // Intentar index.ts/js en directorios
        let index_ts = resolved.join("index.ts");
        let index_js = resolved.join("index.js");

        if index_ts.exists() {
            return Some(index_ts);
        }
        if index_js.exists() {
            return Some(index_js);
        }

        // Si el archivo existe tal cual (sin cambiar extensión)
        if resolved.exists() {
            Some(resolved)
        } else {
            None
        }
    }

    /// Normaliza una ruta de archivo a una representación canónica
    fn normalize_file_path(&self, path: &Path) -> String {
        // Obtener ruta relativa al directorio raíz del proyecto
        if let Ok(relative) = path.strip_prefix(&self.project_root) {
            relative
                .to_string_lossy()
                .replace('\\', "/")
                .to_lowercase()
        } else {
            path.to_string_lossy().replace('\\', "/").to_lowercase()
        }
    }

    /// Verifica si una dependencia es interna del proyecto
    fn is_internal_dependency(&self, path: &str) -> bool {
        // Es interna si no contiene node_modules
        !path.contains("node_modules")
    }

    /// Formatea una descripción legible del ciclo
    fn format_cycle_description(&self, cycle: &[String]) -> String {
        if cycle.is_empty() {
            return "Ciclo vacío".to_string();
        }

        let mut desc = String::from("Dependencia cíclica detectada:\n");
        for (i, node) in cycle.iter().enumerate() {
            if i < cycle.len() - 1 {
                desc.push_str(&format!("  {} → {}\n", node, cycle[i + 1]));
            }
        }
        desc.push_str(&format!(
            "\n  ⚠️  Esto rompe la jerarquía de capas y crea acoplamiento circular."
        ));

        desc
    }
}

/// Función pública para analizar dependencias cíclicas en un proyecto
pub fn analyze_circular_dependencies(
    files: &[PathBuf],
    project_root: &Path,
    cm: &SourceMap,
) -> Result<Vec<CircularDependency>> {
    let mut analyzer = CircularDependencyAnalyzer::new(project_root);
    analyzer.build_graph(files, cm)?;
    Ok(analyzer.detect_cycles())
}

/// Imprime un reporte de dependencias cíclicas
pub fn print_circular_dependency_report(cycles: &[CircularDependency]) {
    if cycles.is_empty() {
        println!("✅ No se detectaron dependencias cíclicas.");
        return;
    }

    println!("\n🔴 DEPENDENCIAS CÍCLICAS DETECTADAS\n");
    println!("Se encontraron {} ciclo(s) de dependencias:\n", cycles.len());

    for (i, cycle) in cycles.iter().enumerate() {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Ciclo #{}", i + 1);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        // Mostrar el ciclo completo usando el campo cycle
        println!("📂 Rutas del ciclo:");
        for (j, path) in cycle.cycle.iter().enumerate() {
            if j < cycle.cycle.len() - 1 {
                println!("  {} →", path);
            } else {
                println!("  {} ↑ (cierra el ciclo)", path);
            }
        }
        println!();

        println!("{}", cycle.description);
        println!();
    }

    println!("💡 Soluciones sugeridas:");
    println!("  1. Aplicar Inyección de Dependencias para romper el ciclo");
    println!("  2. Extraer la lógica compartida a un tercer módulo");
    println!("  3. Usar eventos/observadores en lugar de llamadas directas");
    println!("  4. Aplicar el principio de inversión de dependencias (DIP)");
}
