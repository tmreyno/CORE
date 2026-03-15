// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Forensic triage collection commands.
//!
//! Collects system artifacts, security files, and credential-related files
//! from a live (or mounted) system. Includes a credential/secret scanner
//! that detects API keys, tokens, private keys, and passwords in text files.
//!
//! Platform support:
//! - **Windows**: Registry hives, Event logs, Prefetch, SRUM, browser profiles,
//!   SSH keys, certificates, credential manager, WiFi profiles, PowerShell history.
//! - **macOS**: Keychains, Unified logs, TCC database, SSH keys, browser profiles,
//!   cloud credentials, shell history.
//! - **Linux**: shadow/passwd, SSH keys, browser profiles, cloud credentials,
//!   systemd journal, auth.log, shell history, crontab, GPG keys.

#![allow(unused_imports)]

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::LazyLock;
use tauri::Emitter;
use tracing::{info, warn};

/// Cancel flag for triage operations.
static TRIAGE_CANCEL_FLAG: LazyLock<AtomicBool> = LazyLock::new(|| AtomicBool::new(false));

// =============================================================================
// Types
// =============================================================================

/// A triage artifact category with its collection targets.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TriageCategory {
    pub id: String,
    pub name: String,
    pub description: String,
    pub artifact_count: usize,
}

/// A triage collection profile (preset).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TriageProfile {
    pub id: String,
    pub name: String,
    pub description: String,
    pub categories: Vec<String>,
}

/// Options for starting a triage collection.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TriageOptions {
    /// Output directory where collected artifacts are staged.
    pub output_dir: String,
    /// Category IDs to collect (e.g., ["registry", "credentials", "browser"]).
    pub categories: Vec<String>,
    /// Whether to scan collected text files for secrets/credentials.
    pub scan_for_secrets: bool,
    /// Optional root path to collect from (default: system root).
    pub target_root: Option<String>,
}

/// Progress during triage collection.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TriageProgress {
    pub phase: String,
    pub current_file: String,
    pub files_collected: u64,
    pub files_total: u64,
    pub bytes_collected: u64,
    pub percent: f64,
    pub current_category: String,
}

/// A detected secret/credential in a file.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecretFinding {
    pub file_path: String,
    pub line_number: usize,
    pub secret_type: String,
    pub description: String,
    /// Redacted preview of the match (first/last chars shown, middle masked).
    pub preview: String,
    pub confidence: String,
}

/// Result of a triage collection.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TriageResult {
    pub output_dir: String,
    pub files_collected: u64,
    pub bytes_collected: u64,
    pub files_skipped: u64,
    pub files_failed: u64,
    pub duration_secs: f64,
    pub categories_collected: Vec<String>,
    pub secret_findings: Vec<SecretFinding>,
    pub cancelled: bool,
}

// =============================================================================
// Artifact definitions
// =============================================================================

/// Internal representation of a collectible artifact.
struct ArtifactDef {
    category: &'static str,
    /// Name is used for logging/diagnostics; not read in normal code paths.
    #[allow(dead_code)]
    name: &'static str,
    /// Paths to collect (supports `~` for user home, `*` for glob-like user enumeration).
    paths: Vec<String>,
    recursive: bool,
}

fn get_platform_artifacts(target_root: &Path) -> Vec<ArtifactDef> {
    let root = target_root.to_string_lossy();

    #[cfg(target_os = "windows")]
    {
        get_windows_artifacts(&root)
    }

    #[cfg(target_os = "macos")]
    {
        get_macos_artifacts(&root)
    }

    #[cfg(target_os = "linux")]
    {
        get_linux_artifacts(&root)
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        vec![]
    }
}

#[cfg(target_os = "windows")]
fn get_windows_artifacts(root: &str) -> Vec<ArtifactDef> {
    let sys32 = format!("{root}/Windows/System32");
    let config = format!("{sys32}/config");

    vec![
        // ── Registry ────────────────────────────────────────────────
        ArtifactDef {
            category: "registry",
            name: "SAM hive",
            paths: vec![format!("{config}/SAM"), format!("{config}/SAM.LOG1"), format!("{config}/SAM.LOG2")],
            recursive: false,
        },
        ArtifactDef {
            category: "registry",
            name: "SECURITY hive",
            paths: vec![format!("{config}/SECURITY"), format!("{config}/SECURITY.LOG1"), format!("{config}/SECURITY.LOG2")],
            recursive: false,
        },
        ArtifactDef {
            category: "registry",
            name: "SYSTEM hive",
            paths: vec![format!("{config}/SYSTEM"), format!("{config}/SYSTEM.LOG1"), format!("{config}/SYSTEM.LOG2")],
            recursive: false,
        },
        ArtifactDef {
            category: "registry",
            name: "SOFTWARE hive",
            paths: vec![format!("{config}/SOFTWARE"), format!("{config}/SOFTWARE.LOG1"), format!("{config}/SOFTWARE.LOG2")],
            recursive: false,
        },
        ArtifactDef {
            category: "registry",
            name: "DEFAULT hive",
            paths: vec![format!("{config}/DEFAULT")],
            recursive: false,
        },
        // ── Event Logs ──────────────────────────────────────────────
        ArtifactDef {
            category: "eventlogs",
            name: "Windows Event Logs (.evtx)",
            paths: vec![format!("{sys32}/winevt/Logs")],
            recursive: true,
        },
        // ── System Artifacts ────────────────────────────────────────
        ArtifactDef {
            category: "system",
            name: "Prefetch",
            paths: vec![format!("{root}/Windows/Prefetch")],
            recursive: false,
        },
        ArtifactDef {
            category: "system",
            name: "SRUM database",
            paths: vec![format!("{sys32}/sru/SRUDB.dat")],
            recursive: false,
        },
        ArtifactDef {
            category: "system",
            name: "Amcache",
            paths: vec![format!("{root}/Windows/appcompat/Programs/Amcache.hve")],
            recursive: false,
        },
        ArtifactDef {
            category: "system",
            name: "Scheduled Tasks",
            paths: vec![format!("{sys32}/Tasks")],
            recursive: true,
        },
        // ── Credentials & Keys ──────────────────────────────────────
        ArtifactDef {
            category: "credentials",
            name: "Windows Credential Manager",
            paths: vec![
                // Enumerate all user profiles: %SYSTEMDRIVE%/Users/*/
                format!("{root}/Users/*/AppData/Local/Microsoft/Credentials"),
                format!("{root}/Users/*/AppData/Roaming/Microsoft/Credentials"),
            ],
            recursive: true,
        },
        ArtifactDef {
            category: "credentials",
            name: "DPAPI Master Keys",
            paths: vec![
                format!("{root}/Users/*/AppData/Roaming/Microsoft/Protect"),
            ],
            recursive: true,
        },
        ArtifactDef {
            category: "credentials",
            name: "SSH keys",
            paths: vec![format!("{root}/Users/*/.ssh")],
            recursive: false,
        },
        ArtifactDef {
            category: "credentials",
            name: "WiFi profiles",
            paths: vec![format!("{root}/ProgramData/Microsoft/Wlansvc/Profiles/Interfaces")],
            recursive: true,
        },
        ArtifactDef {
            category: "credentials",
            name: "Certificate stores",
            paths: vec![
                format!("{root}/Users/*/AppData/Roaming/Microsoft/SystemCertificates"),
            ],
            recursive: true,
        },
        ArtifactDef {
            category: "credentials",
            name: "AWS credentials",
            paths: vec![format!("{root}/Users/*/.aws")],
            recursive: false,
        },
        ArtifactDef {
            category: "credentials",
            name: "Azure credentials",
            paths: vec![format!("{root}/Users/*/.azure")],
            recursive: false,
        },
        ArtifactDef {
            category: "credentials",
            name: "GCP credentials",
            paths: vec![format!("{root}/Users/*/.config/gcloud")],
            recursive: true,
        },
        // ── Browser ─────────────────────────────────────────────────
        ArtifactDef {
            category: "browser",
            name: "Chrome profiles",
            paths: vec![format!("{root}/Users/*/AppData/Local/Google/Chrome/User Data")],
            recursive: true,
        },
        ArtifactDef {
            category: "browser",
            name: "Firefox profiles",
            paths: vec![format!("{root}/Users/*/AppData/Roaming/Mozilla/Firefox/Profiles")],
            recursive: true,
        },
        ArtifactDef {
            category: "browser",
            name: "Edge profiles",
            paths: vec![format!("{root}/Users/*/AppData/Local/Microsoft/Edge/User Data")],
            recursive: true,
        },
        // ── User Activity ───────────────────────────────────────────
        ArtifactDef {
            category: "useractivity",
            name: "NTUSER.DAT",
            paths: vec![format!("{root}/Users/*/NTUSER.DAT"), format!("{root}/Users/*/NTUSER.DAT.LOG1")],
            recursive: false,
        },
        ArtifactDef {
            category: "useractivity",
            name: "Recent files",
            paths: vec![format!("{root}/Users/*/AppData/Roaming/Microsoft/Windows/Recent")],
            recursive: true,
        },
        ArtifactDef {
            category: "useractivity",
            name: "PowerShell history",
            paths: vec![format!("{root}/Users/*/AppData/Roaming/Microsoft/Windows/PowerShell/PSReadLine/ConsoleHost_history.txt")],
            recursive: false,
        },
        ArtifactDef {
            category: "useractivity",
            name: "Jump Lists",
            paths: vec![
                format!("{root}/Users/*/AppData/Roaming/Microsoft/Windows/Recent/AutomaticDestinations"),
                format!("{root}/Users/*/AppData/Roaming/Microsoft/Windows/Recent/CustomDestinations"),
            ],
            recursive: false,
        },
        // ── Network ─────────────────────────────────────────────────
        ArtifactDef {
            category: "network",
            name: "Hosts file",
            paths: vec![format!("{sys32}/drivers/etc/hosts")],
            recursive: false,
        },
        ArtifactDef {
            category: "network",
            name: "Firewall rules",
            paths: vec![format!("{sys32}/LogFiles/Firewall")],
            recursive: true,
        },
    ]
}

#[cfg(target_os = "macos")]
fn get_macos_artifacts(root: &str) -> Vec<ArtifactDef> {
    vec![
        // ── Credentials & Keys ──────────────────────────────────────
        ArtifactDef {
            category: "credentials",
            name: "System Keychain",
            paths: vec![format!("{root}/Library/Keychains")],
            recursive: false,
        },
        ArtifactDef {
            category: "credentials",
            name: "User Keychains",
            paths: vec![format!("{root}/Users/*/Library/Keychains")],
            recursive: false,
        },
        ArtifactDef {
            category: "credentials",
            name: "SSH keys",
            paths: vec![format!("{root}/Users/*/.ssh")],
            recursive: false,
        },
        ArtifactDef {
            category: "credentials",
            name: "AWS credentials",
            paths: vec![format!("{root}/Users/*/.aws")],
            recursive: false,
        },
        ArtifactDef {
            category: "credentials",
            name: "Azure credentials",
            paths: vec![format!("{root}/Users/*/.azure")],
            recursive: false,
        },
        ArtifactDef {
            category: "credentials",
            name: "GCP credentials",
            paths: vec![format!("{root}/Users/*/.config/gcloud")],
            recursive: true,
        },
        ArtifactDef {
            category: "credentials",
            name: "GPG keys",
            paths: vec![format!("{root}/Users/*/.gnupg")],
            recursive: true,
        },
        // ── System ──────────────────────────────────────────────────
        ArtifactDef {
            category: "system",
            name: "TCC database (privacy permissions)",
            paths: vec![
                format!("{root}/Library/Application Support/com.apple.TCC/TCC.db"),
                format!("{root}/Users/*/Library/Application Support/com.apple.TCC/TCC.db"),
            ],
            recursive: false,
        },
        ArtifactDef {
            category: "system",
            name: "Quarantine events",
            paths: vec![format!("{root}/Users/*/Library/Preferences/com.apple.LaunchServices.QuarantineEventsV2")],
            recursive: false,
        },
        ArtifactDef {
            category: "system",
            name: "Install history",
            paths: vec![format!("{root}/var/log/install.log")],
            recursive: false,
        },
        ArtifactDef {
            category: "system",
            name: "System logs",
            paths: vec![format!("{root}/var/log/system.log"), format!("{root}/var/log/wifi.log")],
            recursive: false,
        },
        ArtifactDef {
            category: "system",
            name: "Launch agents/daemons",
            paths: vec![
                format!("{root}/Library/LaunchAgents"),
                format!("{root}/Library/LaunchDaemons"),
                format!("{root}/Users/*/Library/LaunchAgents"),
            ],
            recursive: false,
        },
        ArtifactDef {
            category: "system",
            name: "Crontab files",
            paths: vec![format!("{root}/var/at/tabs"), format!("{root}/usr/lib/cron/tabs")],
            recursive: false,
        },
        // ── Event Logs ──────────────────────────────────────────────
        ArtifactDef {
            category: "eventlogs",
            name: "Unified logs (logarchive)",
            paths: vec![format!("{root}/var/db/diagnostics")],
            recursive: true,
        },
        ArtifactDef {
            category: "eventlogs",
            name: "Audit logs",
            paths: vec![format!("{root}/var/audit")],
            recursive: false,
        },
        ArtifactDef {
            category: "eventlogs",
            name: "FSEvents",
            paths: vec![format!("{root}/.fseventsd")],
            recursive: true,
        },
        // ── Browser ─────────────────────────────────────────────────
        ArtifactDef {
            category: "browser",
            name: "Chrome profiles",
            paths: vec![format!("{root}/Users/*/Library/Application Support/Google/Chrome")],
            recursive: true,
        },
        ArtifactDef {
            category: "browser",
            name: "Firefox profiles",
            paths: vec![format!("{root}/Users/*/Library/Application Support/Firefox/Profiles")],
            recursive: true,
        },
        ArtifactDef {
            category: "browser",
            name: "Safari",
            paths: vec![
                format!("{root}/Users/*/Library/Safari"),
                format!("{root}/Users/*/Library/Cookies"),
            ],
            recursive: true,
        },
        // ── User Activity ───────────────────────────────────────────
        ArtifactDef {
            category: "useractivity",
            name: "Shell history",
            paths: vec![
                format!("{root}/Users/*/.bash_history"),
                format!("{root}/Users/*/.zsh_history"),
                format!("{root}/Users/*/.bash_sessions"),
            ],
            recursive: false,
        },
        ArtifactDef {
            category: "useractivity",
            name: "Recent items",
            paths: vec![format!("{root}/Users/*/Library/Application Support/com.apple.sharedfilelist")],
            recursive: true,
        },
        ArtifactDef {
            category: "useractivity",
            name: "Spotlight shortcuts",
            paths: vec![format!("{root}/Users/*/Library/Application Support/com.apple.spotlight.Shortcuts")],
            recursive: false,
        },
        // ── Network ─────────────────────────────────────────────────
        ArtifactDef {
            category: "network",
            name: "Known networks",
            paths: vec![
                format!("{root}/Library/Preferences/SystemConfiguration/com.apple.airport.preferences.plist"),
                format!("{root}/Library/Preferences/com.apple.wifi.known-networks.plist"),
            ],
            recursive: false,
        },
        ArtifactDef {
            category: "network",
            name: "Hosts file",
            paths: vec![format!("{root}/etc/hosts")],
            recursive: false,
        },
        ArtifactDef {
            category: "network",
            name: "Firewall configuration",
            paths: vec![format!("{root}/Library/Preferences/com.apple.alf.plist")],
            recursive: false,
        },
    ]
}

#[cfg(target_os = "linux")]
fn get_linux_artifacts(root: &str) -> Vec<ArtifactDef> {
    vec![
        // ── System Security ─────────────────────────────────────────
        ArtifactDef {
            category: "system",
            name: "Password and group files",
            paths: vec![
                format!("{root}/etc/passwd"),
                format!("{root}/etc/shadow"),
                format!("{root}/etc/group"),
                format!("{root}/etc/gshadow"),
                format!("{root}/etc/sudoers"),
            ],
            recursive: false,
        },
        ArtifactDef {
            category: "system",
            name: "SSH server config",
            paths: vec![format!("{root}/etc/ssh")],
            recursive: false,
        },
        ArtifactDef {
            category: "system",
            name: "PAM configuration",
            paths: vec![format!("{root}/etc/pam.d")],
            recursive: false,
        },
        ArtifactDef {
            category: "system",
            name: "Crontab files",
            paths: vec![
                format!("{root}/etc/crontab"),
                format!("{root}/etc/cron.d"),
                format!("{root}/var/spool/cron"),
            ],
            recursive: true,
        },
        ArtifactDef {
            category: "system",
            name: "Systemd services",
            paths: vec![
                format!("{root}/etc/systemd/system"),
                format!("{root}/usr/lib/systemd/system"),
            ],
            recursive: true,
        },
        // ── Credentials & Keys ──────────────────────────────────────
        ArtifactDef {
            category: "credentials",
            name: "SSH keys",
            paths: vec![
                format!("{root}/home/*/.ssh"),
                format!("{root}/root/.ssh"),
            ],
            recursive: false,
        },
        ArtifactDef {
            category: "credentials",
            name: "GPG keys",
            paths: vec![
                format!("{root}/home/*/.gnupg"),
                format!("{root}/root/.gnupg"),
            ],
            recursive: true,
        },
        ArtifactDef {
            category: "credentials",
            name: "AWS credentials",
            paths: vec![
                format!("{root}/home/*/.aws"),
                format!("{root}/root/.aws"),
            ],
            recursive: false,
        },
        ArtifactDef {
            category: "credentials",
            name: "Azure credentials",
            paths: vec![
                format!("{root}/home/*/.azure"),
                format!("{root}/root/.azure"),
            ],
            recursive: false,
        },
        ArtifactDef {
            category: "credentials",
            name: "GCP credentials",
            paths: vec![
                format!("{root}/home/*/.config/gcloud"),
                format!("{root}/root/.config/gcloud"),
            ],
            recursive: true,
        },
        ArtifactDef {
            category: "credentials",
            name: "Docker credentials",
            paths: vec![
                format!("{root}/home/*/.docker/config.json"),
                format!("{root}/root/.docker/config.json"),
            ],
            recursive: false,
        },
        ArtifactDef {
            category: "credentials",
            name: "Kubernetes config",
            paths: vec![
                format!("{root}/home/*/.kube/config"),
                format!("{root}/root/.kube/config"),
            ],
            recursive: false,
        },
        // ── Event Logs ──────────────────────────────────────────────
        ArtifactDef {
            category: "eventlogs",
            name: "Auth/security logs",
            paths: vec![
                format!("{root}/var/log/auth.log"),
                format!("{root}/var/log/auth.log.1"),
                format!("{root}/var/log/secure"),
                format!("{root}/var/log/secure-*"),
            ],
            recursive: false,
        },
        ArtifactDef {
            category: "eventlogs",
            name: "System logs",
            paths: vec![
                format!("{root}/var/log/syslog"),
                format!("{root}/var/log/messages"),
                format!("{root}/var/log/kern.log"),
                format!("{root}/var/log/daemon.log"),
            ],
            recursive: false,
        },
        ArtifactDef {
            category: "eventlogs",
            name: "Systemd journal",
            paths: vec![format!("{root}/var/log/journal")],
            recursive: true,
        },
        ArtifactDef {
            category: "eventlogs",
            name: "Login records",
            paths: vec![
                format!("{root}/var/log/wtmp"),
                format!("{root}/var/log/btmp"),
                format!("{root}/var/log/lastlog"),
                format!("{root}/var/run/utmp"),
            ],
            recursive: false,
        },
        // ── Browser ─────────────────────────────────────────────────
        ArtifactDef {
            category: "browser",
            name: "Chrome profiles",
            paths: vec![format!("{root}/home/*/.config/google-chrome")],
            recursive: true,
        },
        ArtifactDef {
            category: "browser",
            name: "Firefox profiles",
            paths: vec![format!("{root}/home/*/.mozilla/firefox")],
            recursive: true,
        },
        // ── User Activity ───────────────────────────────────────────
        ArtifactDef {
            category: "useractivity",
            name: "Shell history",
            paths: vec![
                format!("{root}/home/*/.bash_history"),
                format!("{root}/home/*/.zsh_history"),
                format!("{root}/root/.bash_history"),
                format!("{root}/root/.zsh_history"),
            ],
            recursive: false,
        },
        // ── Network ─────────────────────────────────────────────────
        ArtifactDef {
            category: "network",
            name: "Hosts file",
            paths: vec![format!("{root}/etc/hosts"), format!("{root}/etc/resolv.conf")],
            recursive: false,
        },
        ArtifactDef {
            category: "network",
            name: "Network manager",
            paths: vec![format!("{root}/etc/NetworkManager/system-connections")],
            recursive: false,
        },
        ArtifactDef {
            category: "network",
            name: "IPTables rules",
            paths: vec![
                format!("{root}/etc/iptables"),
                format!("{root}/etc/sysconfig/iptables"),
            ],
            recursive: false,
        },
    ]
}

// =============================================================================
// Category metadata
// =============================================================================

fn get_category_meta() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("registry", "Registry Hives", "Windows registry hives (SAM, SECURITY, SYSTEM, SOFTWARE) containing system configuration, user accounts, and security policies"),
        ("eventlogs", "Event Logs", "System event logs, security audit trails, and forensic log sources"),
        ("system", "System Artifacts", "OS-level artifacts including configuration files, scheduled tasks, and security databases"),
        ("credentials", "Credentials & Keys", "SSH keys, cloud provider credentials, keychains, certificates, DPAPI keys, and other authentication material"),
        ("browser", "Browser Data", "Browser profiles including bookmarks, history, cookies, saved passwords databases, and cached data"),
        ("useractivity", "User Activity", "User-level artifacts including shell history, recent documents, registry hives, and application usage"),
        ("network", "Network Configuration", "Network-related configuration including hosts files, WiFi profiles, firewall rules, and DNS settings"),
    ]
}

// =============================================================================
// Profile definitions
// =============================================================================

fn get_profiles() -> Vec<TriageProfile> {
    vec![
        TriageProfile {
            id: "security".to_string(),
            name: "Security Artifacts".to_string(),
            description: "Registry hives, event logs, system security databases, and authentication material".to_string(),
            categories: vec!["registry".to_string(), "eventlogs".to_string(), "system".to_string()],
        },
        TriageProfile {
            id: "credentials".to_string(),
            name: "Credentials & Keys".to_string(),
            description: "SSH keys, cloud credentials, keychains, certificates, browser password databases, and API tokens".to_string(),
            categories: vec!["credentials".to_string(), "browser".to_string()],
        },
        TriageProfile {
            id: "useractivity".to_string(),
            name: "User Activity".to_string(),
            description: "Shell history, recent documents, browser history, and application usage patterns".to_string(),
            categories: vec!["useractivity".to_string(), "browser".to_string()],
        },
        TriageProfile {
            id: "network".to_string(),
            name: "Network".to_string(),
            description: "WiFi profiles, hosts file, DNS configuration, and firewall rules".to_string(),
            categories: vec!["network".to_string()],
        },
        TriageProfile {
            id: "full".to_string(),
            name: "Full Triage".to_string(),
            description: "Comprehensive collection of all available artifact categories".to_string(),
            categories: vec![
                "registry".to_string(),
                "eventlogs".to_string(),
                "system".to_string(),
                "credentials".to_string(),
                "browser".to_string(),
                "useractivity".to_string(),
                "network".to_string(),
            ],
        },
    ]
}

// =============================================================================
// Tauri Commands
// =============================================================================

/// Get available triage profiles and categories for the current platform.
#[tauri::command]
pub async fn triage_get_profiles() -> Result<(Vec<TriageProfile>, Vec<TriageCategory>), String> {
    let platform_root = get_default_root();
    let artifacts = get_platform_artifacts(Path::new(&platform_root));

    // Count artifacts per category
    let mut category_counts: HashMap<String, usize> = HashMap::new();
    for art in &artifacts {
        *category_counts.entry(art.category.to_string()).or_default() += 1;
    }

    let meta = get_category_meta();
    let categories: Vec<TriageCategory> = meta
        .iter()
        .filter(|(id, _, _)| category_counts.contains_key(*id))
        .map(|(id, name, desc)| TriageCategory {
            id: id.to_string(),
            name: name.to_string(),
            description: desc.to_string(),
            artifact_count: *category_counts.get(*id).unwrap_or(&0),
        })
        .collect();

    let profiles = get_profiles();

    Ok((profiles, categories))
}

/// Execute a triage collection.
#[tauri::command]
#[allow(unused_variables)]
pub async fn triage_collect(
    options: TriageOptions,
    window: tauri::Window,
) -> Result<TriageResult, String> {
    TRIAGE_CANCEL_FLAG.store(false, Ordering::SeqCst);

    let default_root = get_default_root();
    let target_root = options.target_root.as_deref().unwrap_or(&default_root);
    let target_path = Path::new(target_root);
    let output_path = Path::new(&options.output_dir);

    // Create output directory
    std::fs::create_dir_all(output_path)
        .map_err(|e| format!("Failed to create output directory: {e}"))?;

    info!(
        "Starting triage collection: categories={:?}, output={}, target={}",
        options.categories,
        options.output_dir,
        target_root
    );

    let started = std::time::Instant::now();

    // Get artifacts filtered by selected categories
    let all_artifacts = get_platform_artifacts(target_path);
    let selected: Vec<&ArtifactDef> = all_artifacts
        .iter()
        .filter(|a| options.categories.contains(&a.category.to_string()))
        .collect();

    // First pass: enumerate all files to collect (for progress)
    let mut all_files: Vec<(PathBuf, String)> = Vec::new(); // (source, relative_dest)
    for art in &selected {
        let resolved = resolve_artifact_paths(&art.paths, target_path);
        for src in resolved {
            if src.is_file() {
                let rel = make_relative(&src, target_path, art.category);
                all_files.push((src, rel));
            } else if src.is_dir() && art.recursive {
                if let Ok(entries) = collect_dir_recursive(&src) {
                    for entry in entries {
                        let rel = make_relative(&entry, target_path, art.category);
                        all_files.push((entry, rel));
                    }
                }
            } else if src.is_dir() {
                // Non-recursive: collect direct children
                if let Ok(rd) = std::fs::read_dir(&src) {
                    for entry in rd.flatten() {
                        let p = entry.path();
                        if p.is_file() {
                            let rel = make_relative(&p, target_path, art.category);
                            all_files.push((p, rel));
                        }
                    }
                }
            }
        }
    }

    let total = all_files.len() as u64;
    let mut collected: u64 = 0;
    let mut bytes: u64 = 0;
    let mut skipped: u64 = 0;
    let mut failed: u64 = 0;

    info!("Triage: {} files to collect across {} categories", total, options.categories.len());

    // Copy files to output
    for (i, (src, rel_dest)) in all_files.iter().enumerate() {
        if TRIAGE_CANCEL_FLAG.load(Ordering::SeqCst) {
            info!("Triage collection cancelled at file {}/{}", i, total);
            return Ok(TriageResult {
                output_dir: options.output_dir,
                files_collected: collected,
                bytes_collected: bytes,
                files_skipped: skipped,
                files_failed: failed,
                duration_secs: started.elapsed().as_secs_f64(),
                categories_collected: options.categories,
                secret_findings: vec![],
                cancelled: true,
            });
        }

        let dest = output_path.join(rel_dest);

        // Emit progress
        if i % 5 == 0 || i == 0 {
            let pct = if total > 0 { (i as f64 / total as f64) * 100.0 } else { 0.0 };
            let category = rel_dest.split('/').next().unwrap_or("unknown");
            let _ = window.emit("triage-progress", TriageProgress {
                phase: "collecting".to_string(),
                current_file: src.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(),
                files_collected: collected,
                files_total: total,
                bytes_collected: bytes,
                percent: pct,
                current_category: category.to_string(),
            });
        }

        // Create parent directories
        if let Some(parent) = dest.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                warn!("Failed to create directory {}: {e}", parent.display());
                failed += 1;
                continue;
            }
        }

        // Copy file (read-only — never modifies source)
        match std::fs::copy(src, &dest) {
            Ok(size) => {
                collected += 1;
                bytes += size;
            }
            Err(e) => {
                // Permission denied or locked files are expected (e.g., SAM hive on live Windows)
                warn!("Failed to copy {}: {e}", src.display());
                skipped += 1;
            }
        }
    }

    // Secret scanning phase
    let mut findings = Vec::new();
    if options.scan_for_secrets && collected > 0 {
        let _ = window.emit("triage-progress", TriageProgress {
            phase: "scanning".to_string(),
            current_file: String::new(),
            files_collected: collected,
            files_total: total,
            bytes_collected: bytes,
            percent: 95.0,
            current_category: "secrets".to_string(),
        });

        findings = scan_for_secrets(output_path);
        info!("Secret scan found {} potential findings", findings.len());
    }

    let duration = started.elapsed().as_secs_f64();

    // Final progress
    let _ = window.emit("triage-progress", TriageProgress {
        phase: "complete".to_string(),
        current_file: String::new(),
        files_collected: collected,
        files_total: total,
        bytes_collected: bytes,
        percent: 100.0,
        current_category: String::new(),
    });

    info!(
        "Triage complete: {} files ({} bytes) in {:.1}s, {} skipped, {} failed, {} secrets found",
        collected, bytes, duration, skipped, failed, findings.len()
    );

    Ok(TriageResult {
        output_dir: options.output_dir,
        files_collected: collected,
        bytes_collected: bytes,
        files_skipped: skipped,
        files_failed: failed,
        duration_secs: duration,
        categories_collected: options.categories,
        secret_findings: findings,
        cancelled: false,
    })
}

/// Cancel an in-progress triage collection.
#[tauri::command]
pub async fn triage_cancel() -> Result<(), String> {
    TRIAGE_CANCEL_FLAG.store(true, Ordering::SeqCst);
    info!("Triage cancel requested");
    Ok(())
}

// =============================================================================
// Helpers
// =============================================================================

fn get_default_root() -> String {
    #[cfg(target_os = "windows")]
    { "C:".to_string() }

    #[cfg(target_os = "macos")]
    { "/".to_string() }

    #[cfg(target_os = "linux")]
    { "/".to_string() }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    { "/".to_string() }
}

/// Resolve artifact paths, expanding `*` wildcards in user directory patterns.
fn resolve_artifact_paths(patterns: &[String], _target_root: &Path) -> Vec<PathBuf> {
    let mut result = Vec::new();

    for pattern in patterns {
        if pattern.contains('*') {
            // Expand glob-style wildcards (e.g., /Users/*/... → /Users/alice/..., /Users/bob/...)
            let parts: Vec<&str> = pattern.splitn(2, '*').collect();
            if parts.len() == 2 {
                let parent_dir = Path::new(parts[0]);
                let suffix = parts[1];

                if parent_dir.is_dir() {
                    if let Ok(entries) = std::fs::read_dir(parent_dir) {
                        for entry in entries.flatten() {
                            let user_path = entry.path();
                            if user_path.is_dir() {
                                let full = format!("{}{suffix}", user_path.display());
                                let p = PathBuf::from(&full);
                                if p.exists() {
                                    result.push(p);
                                }
                            }
                        }
                    }
                }
            }
        } else {
            let p = PathBuf::from(pattern);
            if p.exists() {
                result.push(p);
            }
        }
    }

    result
}

/// Collect all files in a directory recursively.
fn collect_dir_recursive(dir: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut files = Vec::new();
    let mut stack = vec![dir.to_path_buf()];

    while let Some(current) = stack.pop() {
        if let Ok(entries) = std::fs::read_dir(&current) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.is_file() {
                    files.push(path);
                }
            }
        }
    }

    Ok(files)
}

/// Create a relative destination path, prefixed by category.
fn make_relative(file_path: &Path, target_root: &Path, category: &str) -> String {
    let rel = file_path
        .strip_prefix(target_root)
        .unwrap_or(file_path)
        .to_string_lossy()
        .to_string();

    // Use forward slashes and prefix with category
    let normalized = rel.replace('\\', "/");
    format!("{category}/{normalized}")
}

// =============================================================================
// Secret Scanner
// =============================================================================

/// Maximum file size to scan (512 KB — larger files are likely databases, not config files)
const MAX_SCAN_SIZE: u64 = 512 * 1024;

/// Scan collected files for secrets, credentials, tokens, and keys.
fn scan_for_secrets(output_dir: &Path) -> Vec<SecretFinding> {
    let mut findings = Vec::new();
    let patterns = get_secret_patterns();

    // Walk the output directory for scannable text files
    let mut stack = vec![output_dir.to_path_buf()];
    while let Some(dir) = stack.pop() {
        if TRIAGE_CANCEL_FLAG.load(Ordering::SeqCst) {
            break;
        }

        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }

            if !is_scannable_file(&path) {
                continue;
            }

            // Check size
            if let Ok(meta) = std::fs::metadata(&path) {
                if meta.len() > MAX_SCAN_SIZE {
                    continue;
                }
            }

            // Read and scan
            if let Ok(content) = std::fs::read_to_string(&path) {
                let rel_path = path
                    .strip_prefix(output_dir)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .to_string();

                for (line_num, line) in content.lines().enumerate() {
                    for (pattern, secret_type, description, confidence) in &patterns {
                        if pattern.is_match(line) {
                            // Extract the matched portion and redact it
                            if let Some(m) = pattern.find(line) {
                                findings.push(SecretFinding {
                                    file_path: rel_path.clone(),
                                    line_number: line_num + 1,
                                    secret_type: secret_type.to_string(),
                                    description: description.to_string(),
                                    preview: redact_match(m.as_str()),
                                    confidence: confidence.to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    findings
}

/// Check if a file is likely text/config (worth scanning for secrets).
fn is_scannable_file(path: &Path) -> bool {
    let scannable_extensions = [
        "txt", "log", "cfg", "conf", "config", "ini", "env", "yaml", "yml",
        "json", "xml", "toml", "properties", "pem", "key", "pub", "crt",
        "cer", "pfx", "p12", "plist", "sh", "bash", "zsh", "ps1", "bat",
        "cmd", "py", "rb", "js", "ts", "php", "sql", "csv", "md",
        "credentials", "gitconfig", "npmrc", "netrc", "pgpass",
    ];

    // Also scan files without extensions (e.g., `credentials`, `config`, `known_hosts`)
    let scannable_names = [
        "credentials", "config", "known_hosts", "authorized_keys",
        "id_rsa", "id_ed25519", "id_ecdsa", "id_dsa",
        "shadow", "passwd", "hosts", "resolv.conf",
        ".env", ".envrc", ".bashrc", ".zshrc", ".profile",
        "ConsoleHost_history.txt", "history",
    ];

    if let Some(ext) = path.extension() {
        if scannable_extensions.contains(&ext.to_string_lossy().to_lowercase().as_str()) {
            return true;
        }
    }

    if let Some(name) = path.file_name() {
        let name_str = name.to_string_lossy().to_lowercase();
        if scannable_names.iter().any(|n| name_str == n.to_lowercase() || name_str.ends_with(n)) {
            return true;
        }
    }

    false
}

/// Build regex patterns for detecting secrets. Returns (regex, type, description, confidence).
fn get_secret_patterns() -> Vec<(Regex, &'static str, &'static str, &'static str)> {
    let mut patterns = Vec::new();

    // Helper to add pattern, skipping invalid regexes
    let mut add = |pat: &str, stype: &'static str, desc: &'static str, conf: &'static str| {
        if let Ok(re) = Regex::new(pat) {
            patterns.push((re, stype, desc, conf));
        }
    };

    // ── AWS ─────────────────────────────────────────────────────────
    add(r"(?i)AKIA[0-9A-Z]{16}", "aws_access_key", "AWS Access Key ID", "high");
    add(r"(?i)aws[_\-]?secret[_\-]?access[_\-]?key\s*[=:]\s*\S+", "aws_secret_key", "AWS Secret Access Key", "high");

    // ── Private Keys ────────────────────────────────────────────────
    add(r"-----BEGIN\s+(RSA\s+)?PRIVATE\s+KEY-----", "private_key", "RSA/Generic Private Key (PEM)", "high");
    add(r"-----BEGIN\s+EC\s+PRIVATE\s+KEY-----", "ec_private_key", "EC Private Key (PEM)", "high");
    add(r"-----BEGIN\s+OPENSSH\s+PRIVATE\s+KEY-----", "openssh_key", "OpenSSH Private Key", "high");
    add(r"-----BEGIN\s+PGP\s+PRIVATE\s+KEY\s+BLOCK-----", "pgp_key", "PGP Private Key", "high");
    add(r"-----BEGIN\s+ENCRYPTED\s+PRIVATE\s+KEY-----", "encrypted_key", "Encrypted Private Key (PEM)", "medium");

    // ── Generic API / Tokens ────────────────────────────────────────
    add(r#"(?i)(api[_\-]?key|api[_\-]?secret|api[_\-]?token)\s*[=:]\s*['"]?[A-Za-z0-9\-_]{20,}['"]?"#, "api_key", "Generic API Key/Token", "medium");
    add(r#"(?i)(access[_\-]?token|auth[_\-]?token|bearer[_\-]?token)\s*[=:]\s*['"]?[A-Za-z0-9\-_.]{20,}['"]?"#, "access_token", "Access/Auth/Bearer Token", "medium");
    add(r"(?i)bearer\s+[A-Za-z0-9\-_.]{20,}", "bearer_token", "Bearer Token in Header", "high");

    // ── Connection Strings ──────────────────────────────────────────
    add(r#"(?i)(password|passwd|pwd)\s*[=:]\s*['"]?[^\s'"]{4,}['"]?"#, "password", "Password in Configuration", "medium");
    add(r"(?i)mysql://\S+:\S+@", "mysql_conn", "MySQL Connection String with Credentials", "high");
    add(r"(?i)postgres(ql)?://\S+:\S+@", "postgres_conn", "PostgreSQL Connection String with Credentials", "high");
    add(r"(?i)mongodb(\+srv)?://\S+:\S+@", "mongodb_conn", "MongoDB Connection String with Credentials", "high");

    // ── Cloud Provider ──────────────────────────────────────────────
    add(r"(?i)gcp[_\-]?service[_\-]?account|type.*service_account", "gcp_service_account", "GCP Service Account Key", "medium");
    add(r#""client_secret"\s*:\s*"[A-Za-z0-9\-_]{20,}""#, "oauth_secret", "OAuth Client Secret", "high");

    // ── GitHub / GitLab ─────────────────────────────────────────────
    add(r"gh[pousr]_[A-Za-z0-9_]{36,}", "github_token", "GitHub Personal Access Token", "high");
    add(r"glpat-[A-Za-z0-9\-_]{20,}", "gitlab_token", "GitLab Personal Access Token", "high");

    // ── Slack / Discord ─────────────────────────────────────────────
    add(r"xox[baprs]-[0-9A-Za-z\-]{10,}", "slack_token", "Slack Token", "high");

    // ── Stripe ──────────────────────────────────────────────────────
    add(r"sk_live_[0-9a-zA-Z]{24,}", "stripe_secret", "Stripe Secret Key", "high");
    add(r"rk_live_[0-9a-zA-Z]{24,}", "stripe_restricted", "Stripe Restricted Key", "high");

    // ── SendGrid / Twilio ───────────────────────────────────────────
    add(r"SG\.[A-Za-z0-9\-_]{22,}\.[A-Za-z0-9\-_]{22,}", "sendgrid_key", "SendGrid API Key", "high");

    // ── SSH / Known Hosts ───────────────────────────────────────────
    add(r"(?i)ssh-rsa\s+AAAA[0-9A-Za-z+/]{100,}", "ssh_public_key", "SSH RSA Public Key", "low");

    // ── Encryption Keys / Passphrases ───────────────────────────────
    add(r#"(?i)(encryption[_\-]?key|encrypt[_\-]?secret|aes[_\-]?key|master[_\-]?key)\s*[=:]\s*['"]?[A-Fa-f0-9]{32,}['"]?"#, "encryption_key", "Encryption Key (hex)", "high");
    add(r#"(?i)(passphrase|pass_phrase)\s*[=:]\s*['"]?[^\s'"]{4,}['"]?"#, "passphrase", "Passphrase in Configuration", "medium");

    patterns
}

/// Redact the middle portion of a matched secret for safe display.
fn redact_match(matched: &str) -> String {
    let len = matched.len();
    if len <= 8 {
        return "*".repeat(len);
    }
    let show = std::cmp::min(4, len / 4);
    format!(
        "{}{}{}",
        &matched[..show],
        "*".repeat(len - 2 * show),
        &matched[len - show..]
    )
}
