// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! System monitoring and resource usage commands.

use std::sync::{OnceLock, Mutex as StdMutex};
use tauri::Emitter;
use tracing::info;

// System Stats Command
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemStats {
    pub cpu_usage: f32,
    pub memory_used: u64,
    pub memory_total: u64,
    pub memory_percent: f32,
    // App-specific stats
    pub app_cpu_usage: f32,
    pub app_memory: u64,
    pub app_threads: usize,
    pub cpu_cores: usize,
}

static SYSTEM: OnceLock<StdMutex<sysinfo::System>> = OnceLock::new();

fn get_system() -> &'static StdMutex<sysinfo::System> {
    SYSTEM.get_or_init(|| {
        // Use minimal initialization - refresh_all is expensive
        // We'll refresh specific items lazily
        let sys = sysinfo::System::new();
        StdMutex::new(sys)
    })
}

/// Initialize system stats collector in background (call from setup)
pub fn init_system_stats_background() {
    std::thread::spawn(|| {
        let start = std::time::Instant::now();
        let Ok(mut sys) = get_system().lock() else { return };
        // Do the expensive refresh in background
        sys.refresh_cpu_usage();
        sys.refresh_memory();
        // Only refresh our own process, not all processes (much faster)
        let pid = sysinfo::Pid::from_u32(std::process::id());
        sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), true);
        info!(elapsed_ms = start.elapsed().as_millis(), "System stats init");
    });
}

pub fn collect_system_stats() -> SystemStats {
    let Ok(mut sys) = get_system().lock() else {
        // Return default stats if lock is poisoned
        tracing::warn!("System stats lock poisoned, returning defaults");
        return SystemStats {
            cpu_usage: 0.0,
            memory_used: 0,
            memory_total: 0,
            memory_percent: 0.0,
            app_cpu_usage: 0.0,
            app_memory: 0,
            app_threads: 0,
            cpu_cores: 0,
        };
    };
    sys.refresh_cpu_usage();
    sys.refresh_memory();
    // Only refresh our own process - refreshing ALL processes is extremely slow (2+ seconds)
    let pid = sysinfo::Pid::from_u32(std::process::id());
    sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), true);
    
    let cpu_usage = sys.global_cpu_usage();
    let memory_used = sys.used_memory();
    let memory_total = sys.total_memory();
    let memory_percent = if memory_total > 0 {
        (memory_used as f32 / memory_total as f32) * 100.0
    } else {
        0.0
    };
    
    // Get app-specific stats
    let (app_cpu_usage, app_memory, app_threads) = if let Some(process) = sys.process(pid) {
        // process.tasks() is not supported on macOS, use rayon thread count as worker threads
        let threads = process.tasks()
            .map(|t| t.len())
            .unwrap_or_else(rayon::current_num_threads);
        (process.cpu_usage(), process.memory(), threads)
    } else {
        (0.0, 0, rayon::current_num_threads())
    };
    
    let cpu_cores = sys.cpus().len();
    
    SystemStats {
        cpu_usage,
        memory_used,
        memory_total,
        memory_percent,
        app_cpu_usage,
        app_memory,
        app_threads,
        cpu_cores,
    }
}

#[tauri::command]
pub fn get_system_stats() -> SystemStats {
    collect_system_stats()
}

/// Start background system stats monitoring - emits "system-stats" events every 2 seconds
pub fn start_system_stats_monitor(app_handle: tauri::AppHandle) {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(2));
            let stats = collect_system_stats();
            let _ = app_handle.emit("system-stats", stats);
        }
    });
}

/// Result of preview cache cleanup
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupResult {
    pub files_removed: u64,
    pub bytes_freed: u64,
    pub errors: Vec<String>,
}

/// Clean up temporary files created by preview extraction and thumbnail generation.
/// Removes contents of `core-ffx-preview/` and `core-ffx-thumbnails/` in the system temp directory.
#[tauri::command]
pub async fn cleanup_preview_cache() -> Result<CleanupResult, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let temp = std::env::temp_dir();
        let dirs = ["core-ffx-preview", "core-ffx-thumbnails"];
        let mut files_removed: u64 = 0;
        let mut bytes_freed: u64 = 0;
        let mut errors = Vec::new();

        for dir_name in &dirs {
            let dir_path = temp.join(dir_name);
            if !dir_path.exists() {
                continue;
            }
            match std::fs::read_dir(&dir_path) {
                Ok(entries) => {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        match std::fs::metadata(&path) {
                            Ok(meta) => {
                                bytes_freed += meta.len();
                                if let Err(e) = std::fs::remove_file(&path) {
                                    errors.push(format!("Failed to remove {}: {}", path.display(), e));
                                } else {
                                    files_removed += 1;
                                }
                            }
                            Err(e) => {
                                errors.push(format!("Failed to read metadata for {}: {}", path.display(), e));
                            }
                        }
                    }
                }
                Err(e) => {
                    errors.push(format!("Failed to read directory {}: {}", dir_path.display(), e));
                }
            }
        }

        info!(files_removed, bytes_freed, "Preview cache cleanup complete");
        Ok(CleanupResult { files_removed, bytes_freed, errors })
    })
    .await
    .map_err(|e| format!("Cleanup task failed: {}", e))?
}

/// Write text content to a file on disk.
/// Used for exporting activity logs, reports, and other text-based data.
#[tauri::command]
pub async fn write_text_file(path: String, content: String) -> Result<(), String> {
    use std::path::Path;

    let file_path = Path::new(&path);

    // Ensure parent directory exists
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {e}"))?;
    }

    std::fs::write(file_path, content.as_bytes())
        .map_err(|e| format!("Failed to write file: {e}"))?;

    info!(path = %path, bytes = content.len(), "Text file written");
    Ok(())
}

/// Get the path to the audit log directory.
/// Returns the platform-specific path where daily-rotating audit logs are stored.
#[tauri::command]
pub fn get_audit_log_path() -> Result<String, String> {
    let dir = crate::logging::audit_log_dir()?;
    Ok(dir.to_string_lossy().into_owned())
}

/// Read recent audit log entries.
///
/// Returns up to `max_lines` recent log entries (newest first).
/// Each entry is a JSON-formatted string from the audit log files.
#[tauri::command]
pub async fn read_audit_log(max_lines: Option<usize>) -> Result<Vec<String>, String> {
    let limit = max_lines.unwrap_or(500);
    tauri::async_runtime::spawn_blocking(move || {
        crate::logging::read_audit_logs(limit)
    })
    .await
    .map_err(|e| format!("Audit log read task failed: {e}"))?
}
