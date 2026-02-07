use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
use std::time::{Duration, Instant};
use miette::{IntoDiagnostic, Result};

/// Representa un evento de cambio de archivo procesado
#[derive(Debug, Clone)]
pub struct FileChangeEvent {
    /// Archivos que han cambiado
    pub changed_files: Vec<PathBuf>,
}

/// Gestor de observación de archivos con debouncing
pub struct FileWatcher {
    watcher: RecommendedWatcher,
    rx: Receiver<notify::Result<Event>>,
    debounce_duration: Duration,
    ignored_paths: Vec<String>,
    project_root: PathBuf,
}

impl FileWatcher {
    /// Crea un nuevo observador de archivos
    pub fn new(debounce_ms: u64, project_root: &Path, ignored_paths: Vec<String>) -> Result<Self> {
        let (tx, rx) = channel();

        let watcher = RecommendedWatcher::new(
            move |res| {
                let _ = tx.send(res);
            },
            Config::default(),
        )
        .into_diagnostic()?;

        Ok(Self {
            watcher,
            rx,
            debounce_duration: Duration::from_millis(debounce_ms),
            ignored_paths,
            project_root: project_root.to_path_buf(),
        })
    }

    /// Comienza a observar un directorio
    pub fn watch(&mut self, path: &Path) -> Result<()> {
        self.watcher
            .watch(path, RecursiveMode::Recursive)
            .into_diagnostic()?;
        Ok(())
    }

    /// Espera por cambios en archivos con debouncing
    /// Devuelve los archivos que cambiaron después del período de debounce
    pub fn wait_for_changes(&self) -> Result<FileChangeEvent> {
        let mut changed_files = Vec::new();
        let mut last_event_time = Instant::now();

        loop {
            // Esperar por eventos con timeout
            match self.rx.recv_timeout(Duration::from_millis(100)) {
                Ok(Ok(event)) => {
                    // Filtrar solo eventos de modificación y creación
                    if self.is_relevant_event(&event) {
                        // Extraer archivos del evento
                        for path in &event.paths {
                            if self.is_relevant_file(path) && !self.should_ignore_path(path) {
                                if !changed_files.contains(path) {
                                    changed_files.push(path.clone());
                                }
                            }
                        }
                        last_event_time = Instant::now();
                    }
                }
                Ok(Err(e)) => {
                    eprintln!("⚠️  Error en file watcher: {:?}", e);
                }
                Err(_) => {
                    // Timeout: verificar si debemos retornar los cambios acumulados
                    if !changed_files.is_empty()
                        && last_event_time.elapsed() >= self.debounce_duration
                    {
                        return Ok(FileChangeEvent {
                            changed_files,
                        });
                    }
                }
            }
        }
    }

    /// Verifica si un evento es relevante para el análisis
    fn is_relevant_event(&self, event: &Event) -> bool {
        matches!(
            event.kind,
            EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
        )
    }

    /// Verifica si un archivo es relevante para el análisis
    fn is_relevant_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            matches!(
                ext.to_str(),
                Some("ts") | Some("tsx") | Some("js") | Some("jsx")
            )
        } else {
            false
        }
    }

    /// Verifica si un path debe ser ignorado según los patrones configurados
    fn should_ignore_path(&self, path: &Path) -> bool {
        // Obtener la ruta relativa al root del proyecto
        let relative_path = path
            .strip_prefix(&self.project_root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");

        // Verificar si la ruta coincide con algún patrón de exclusión
        for pattern in &self.ignored_paths {
            let normalized_pattern = pattern.replace('\\', "/");

            // Coincidencia si la ruta contiene el patrón
            if relative_path.contains(&normalized_pattern.trim_end_matches('/'))
                || relative_path.starts_with(&normalized_pattern)
                || relative_path.starts_with(&format!("{}/", normalized_pattern.trim_end_matches('/')))
            {
                return true;
            }
        }

        false
    }
}

/// Inicia el modo watch para observación continua
pub fn start_watch_mode<F>(
    project_root: &Path,
    ignored_paths: Vec<String>,
    mut on_change: F,
) -> Result<()>
where
    F: FnMut(&[PathBuf]) -> Result<()>,
{
    println!("👁️  Modo Watch activado");
    println!("📂 Observando: {}", project_root.display());

    if !ignored_paths.is_empty() {
        println!("🚫 Ignorando: {}", ignored_paths.join(", "));
    }

    println!("⏱️  Debounce: 300ms");
    println!("💡 Presiona Ctrl+C para detener\n");

    let mut watcher = FileWatcher::new(300, project_root, ignored_paths)?;
    watcher.watch(project_root)?;

    loop {
        match watcher.wait_for_changes() {
            Ok(event) => {
                println!("\n🔄 Cambios detectados en {} archivo(s):", event.changed_files.len());
                for file in &event.changed_files {
                    println!("   📝 {}", file.display());
                }
                println!();

                // Ejecutar callback de análisis
                if let Err(e) = on_change(&event.changed_files) {
                    eprintln!("❌ Error durante re-análisis: {}", e);
                } else {
                    println!("✅ Re-análisis completado\n");
                    println!("👁️  Esperando cambios...");
                }
            }
            Err(e) => {
                eprintln!("❌ Error en watch mode: {}", e);
                break;
            }
        }
    }

    Ok(())
}
