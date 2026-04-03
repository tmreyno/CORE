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

use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use seven_zip::StreamOptions;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use tauri::Emitter;
use tracing::{info, warn};

/// Cancel flag for triage operations.
static TRIAGE_CANCEL_FLAG: LazyLock<AtomicBool> = LazyLock::new(|| AtomicBool::new(false));

/// Pre-compiled secret detection patterns (compiled once, reused across calls).
static SECRET_PATTERNS: LazyLock<Vec<(Regex, &'static str, &'static str, &'static str)>> =
    LazyLock::new(build_secret_patterns);

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
    /// Individual artifact names within this category (e.g., "SAM hive", "SSH keys").
    pub artifacts: Vec<String>,
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
    /// Maximum file size in bytes to collect (default: 100 MB). Files exceeding
    /// this are skipped. Prevents hangs on very large system log files (e.g.,
    /// macOS `.tracev3` unified logging files that can be hundreds of MB).
    pub max_file_size: Option<u64>,
    /// Optional container format for packaging collected artifacts.
    /// Supported values: `"7z"`. When set, artifacts are packaged into a
    /// container after collection and the staging directory is cleaned up.
    pub container_format: Option<String>,
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

/// Per-category collection statistics.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryResult {
    pub files_collected: u64,
    pub bytes_collected: u64,
    pub files_skipped: u64,
    pub files_failed: u64,
    /// Representative file names collected in this category (for UI display).
    pub sample_files: Vec<String>,
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
    /// Per-category breakdown of collection results.
    pub category_details: HashMap<String, CategoryResult>,
    pub secret_findings: Vec<SecretFinding>,
    pub cancelled: bool,
    /// Path to the packaged container file (7z), if container_format was set.
    pub container_path: Option<String>,
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
        // ── System Identification ────────────────────────────────────
        ArtifactDef {
            category: "systeminfo",
            name: "Computer name & domain",
            paths: vec![
                format!("{sys32}/config/SYSTEM"),
            ],
            recursive: false,
        },
        ArtifactDef {
            category: "systeminfo",
            name: "OS version & product ID",
            paths: vec![
                format!("{sys32}/config/SOFTWARE"),
            ],
            recursive: false,
        },
        ArtifactDef {
            category: "systeminfo",
            name: "SMBIOS / DMI data",
            paths: vec![
                format!("{root}/Windows/System32/wbem/Repository"),
            ],
            recursive: true,
        },
        ArtifactDef {
            category: "systeminfo",
            name: "Time zone info",
            paths: vec![
                format!("{root}/Windows/Globalization/Time Zone/timezoneMapping.xml"),
            ],
            recursive: false,
        },
        ArtifactDef {
            category: "systeminfo",
            name: "Activation & license",
            paths: vec![
                format!("{root}/Windows/ServiceProfiles/NetworkService/AppData/Local/Microsoft/Windows/tokens.dat"),
            ],
            recursive: false,
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
        // ── System Identification ────────────────────────────────────
        ArtifactDef {
            category: "systeminfo",
            name: "Hardware UUID & serial number",
            paths: vec![
                format!("{root}/Library/Preferences/SystemConfiguration/com.apple.Boot.plist"),
            ],
            recursive: false,
        },
        ArtifactDef {
            category: "systeminfo",
            name: "System version",
            paths: vec![
                format!("{root}/System/Library/CoreServices/SystemVersion.plist"),
            ],
            recursive: false,
        },
        ArtifactDef {
            category: "systeminfo",
            name: "Computer name & hostname",
            paths: vec![
                format!("{root}/Library/Preferences/SystemConfiguration/preferences.plist"),
                format!("{root}/etc/hostname"),
            ],
            recursive: false,
        },
        ArtifactDef {
            category: "systeminfo",
            name: "Network interfaces",
            paths: vec![
                format!("{root}/Library/Preferences/SystemConfiguration/NetworkInterfaces.plist"),
            ],
            recursive: false,
        },
        ArtifactDef {
            category: "systeminfo",
            name: "Global preferences",
            paths: vec![
                format!("{root}/Library/Preferences/.GlobalPreferences.plist"),
            ],
            recursive: false,
        },
        ArtifactDef {
            category: "systeminfo",
            name: "Time zone",
            paths: vec![
                format!("{root}/var/db/timezone/localtime"),
                format!("{root}/etc/localtime"),
            ],
            recursive: false,
        },
        ArtifactDef {
            category: "systeminfo",
            name: "Install history",
            paths: vec![
                format!("{root}/Library/Receipts/InstallHistory.plist"),
            ],
            recursive: false,
        },
        ArtifactDef {
            category: "systeminfo",
            name: "Disk & volume info",
            paths: vec![
                format!("{root}/var/db/DiskManagement.plist"),
                format!("{root}/etc/fstab"),
            ],
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
            paths: vec![format!("{root}/home/*/.ssh"), format!("{root}/root/.ssh")],
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
            paths: vec![format!("{root}/home/*/.aws"), format!("{root}/root/.aws")],
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
            paths: vec![
                format!("{root}/etc/hosts"),
                format!("{root}/etc/resolv.conf"),
            ],
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
        // ── System Identification ────────────────────────────────────
        ArtifactDef {
            category: "systeminfo",
            name: "Machine ID",
            paths: vec![
                format!("{root}/etc/machine-id"),
                format!("{root}/var/lib/dbus/machine-id"),
            ],
            recursive: false,
        },
        ArtifactDef {
            category: "systeminfo",
            name: "Hostname",
            paths: vec![
                format!("{root}/etc/hostname"),
                format!("{root}/etc/HOSTNAME"),
            ],
            recursive: false,
        },
        ArtifactDef {
            category: "systeminfo",
            name: "OS release info",
            paths: vec![
                format!("{root}/etc/os-release"),
                format!("{root}/etc/lsb-release"),
                format!("{root}/etc/redhat-release"),
                format!("{root}/etc/debian_version"),
            ],
            recursive: false,
        },
        ArtifactDef {
            category: "systeminfo",
            name: "DMI/SMBIOS data",
            paths: vec![
                format!("{root}/sys/class/dmi/id/product_uuid"),
                format!("{root}/sys/class/dmi/id/product_serial"),
                format!("{root}/sys/class/dmi/id/product_name"),
                format!("{root}/sys/class/dmi/id/sys_vendor"),
                format!("{root}/sys/class/dmi/id/board_serial"),
                format!("{root}/sys/class/dmi/id/board_name"),
                format!("{root}/sys/class/dmi/id/bios_version"),
                format!("{root}/sys/class/dmi/id/bios_vendor"),
            ],
            recursive: false,
        },
        ArtifactDef {
            category: "systeminfo",
            name: "Time zone",
            paths: vec![
                format!("{root}/etc/timezone"),
                format!("{root}/etc/localtime"),
            ],
            recursive: false,
        },
        ArtifactDef {
            category: "systeminfo",
            name: "Disk & mount info",
            paths: vec![format!("{root}/etc/fstab"), format!("{root}/etc/mtab")],
            recursive: false,
        },
        ArtifactDef {
            category: "systeminfo",
            name: "Network interface config",
            paths: vec![
                format!("{root}/etc/network/interfaces"),
                format!("{root}/etc/sysconfig/network-scripts"),
                format!("{root}/etc/netplan"),
            ],
            recursive: true,
        },
    ]
}

// =============================================================================
// Category metadata
// =============================================================================

fn get_category_meta() -> Vec<(&'static str, &'static str, &'static str)> {
    // Ordered by forensic frequency — categories that most commonly yield results
    // during a full triage are listed first (system/browser/user activity are almost
    // always present on any live system; registry is Windows-only).
    vec![
        ("system", "System Artifacts", "OS-level artifacts including configuration files, scheduled tasks, and security databases"),
        ("browser", "Browser Data", "Browser profiles including bookmarks, history, cookies, saved passwords databases, and cached data"),
        ("useractivity", "User Activity", "User-level artifacts including shell history, recent documents, registry hives, and application usage"),
        ("credentials", "Credentials & Keys", "SSH keys, cloud provider credentials, keychains, certificates, DPAPI keys, and other authentication material"),
        ("eventlogs", "Event Logs", "System event logs, security audit trails, and forensic log sources"),
        ("systeminfo", "System Identification", "Hardware UUID, serial number, hostname, OS version, time zone, disk info, and network interfaces — key identifiers for evidence collection and chain of custody"),
        ("network", "Network Configuration", "Network-related configuration including hosts files, WiFi profiles, firewall rules, and DNS settings"),
        ("registry", "Registry Hives", "Windows registry hives (SAM, SECURITY, SYSTEM, SOFTWARE) containing system configuration, user accounts, and security policies"),
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
                "system".to_string(),
                "browser".to_string(),
                "useractivity".to_string(),
                "credentials".to_string(),
                "eventlogs".to_string(),
                "systeminfo".to_string(),
                "network".to_string(),
                "registry".to_string(),
            ],
        },
        TriageProfile {
            id: "identification".to_string(),
            name: "System Identification".to_string(),
            description: "Hardware UUID, serial number, hostname, and OS version — essential identifiers for evidence collection forms and chain of custody documentation".to_string(),
            categories: vec!["systeminfo".to_string()],
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

    // Collect artifact names per category
    let mut category_artifacts: HashMap<String, Vec<String>> = HashMap::new();
    for art in &artifacts {
        category_artifacts
            .entry(art.category.to_string())
            .or_default()
            .push(art.name.to_string());
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
            artifacts: category_artifacts.get(*id).cloned().unwrap_or_default(),
        })
        .collect();

    let profiles = get_profiles();

    Ok((profiles, categories))
}

/// Execute a triage collection.
///
/// Uses `spawn_blocking` to avoid blocking the tokio runtime, which would
/// prevent progress events from being delivered to the frontend.
#[tauri::command]
#[allow(unused_variables)]
pub async fn triage_collect(
    options: TriageOptions,
    window: tauri::Window,
) -> Result<TriageResult, String> {
    TRIAGE_CANCEL_FLAG.store(false, Ordering::Relaxed);

    tokio::task::spawn_blocking(move || {
        let default_root = get_default_root();
        let target_root = options.target_root.as_deref().unwrap_or(&default_root);
        let target_path = Path::new(target_root);
        let output_path = Path::new(&options.output_dir);

        // Create output directory
        std::fs::create_dir_all(output_path)
            .map_err(|e| format!("Failed to create output directory: {e}"))?;

        info!(
            "Starting triage collection: categories={:?}, output={}, target={}",
            options.categories, options.output_dir, target_root
        );

        let started = std::time::Instant::now();

        // Get artifacts filtered by selected categories
        let all_artifacts = get_platform_artifacts(target_path);
        let selected: Vec<&ArtifactDef> = all_artifacts
            .iter()
            .filter(|a| options.categories.contains(&a.category.to_string()))
            .collect();

        // First pass: enumerate all files to collect (for progress)
        let mut all_files: Vec<(PathBuf, String, String)> = Vec::new(); // (source, relative_dest, category)
        for art in &selected {
            let cat = art.category.to_string();
            let resolved = resolve_artifact_paths(&art.paths, target_path);
            for src in resolved {
                if src.is_file() {
                    let rel = make_relative(&src, target_path, art.category);
                    all_files.push((src, rel, cat.clone()));
                } else if src.is_dir() && art.recursive {
                    if let Ok(entries) = collect_dir_recursive(&src) {
                        for entry in entries {
                            let rel = make_relative(&entry, target_path, art.category);
                            all_files.push((entry, rel, cat.clone()));
                        }
                    }
                } else if src.is_dir() {
                    // Non-recursive: collect direct children
                    if let Ok(rd) = std::fs::read_dir(&src) {
                        for entry in rd.flatten() {
                            let p = entry.path();
                            if p.is_file() {
                                let rel = make_relative(&p, target_path, art.category);
                                all_files.push((p, rel, cat.clone()));
                            }
                        }
                    }
                }
            }
        }

        let total = all_files.len() as u64;
        let mut category_details: HashMap<String, CategoryResult> = HashMap::new();

        // Initialize category details for all selected categories
        for cat in &options.categories {
            category_details.insert(
                cat.clone(),
                CategoryResult {
                    files_collected: 0,
                    bytes_collected: 0,
                    files_skipped: 0,
                    files_failed: 0,
                    sample_files: Vec::new(),
                },
            );
        }

        info!(
            "Triage: {} files to collect across {} categories",
            total,
            options.categories.len()
        );

        // Pre-create all destination directories (sequential — avoids race conditions)
        {
            let mut dirs_seen = std::collections::HashSet::new();
            for (_src, rel_dest, _category) in &all_files {
                let dest = output_path.join(rel_dest);
                if let Some(parent) = dest.parent() {
                    if dirs_seen.insert(parent.to_path_buf()) {
                        let _ = std::fs::create_dir_all(parent);
                    }
                }
            }
        }

        // Shared atomic counters for parallel progress
        let a_collected = AtomicU64::new(0);
        let a_bytes = AtomicU64::new(0);
        let a_skipped = AtomicU64::new(0);
        let a_failed = AtomicU64::new(0);
        let a_cancelled = AtomicBool::new(false);
        let shared_category_details = Arc::new(Mutex::new(category_details));
        let last_emit = Arc::new(Mutex::new(std::time::Instant::now()));

        // Default max file size: 100 MB. Prevents hangs on very large system log
        // files (e.g., macOS .tracev3 unified logging files).
        let max_file_size = options.max_file_size.unwrap_or(100 * 1024 * 1024);
        // Per-file copy timeout: 30 seconds. If a copy doesn't finish within this
        // window the thread is abandoned and we move on. This handles kernel-level
        // blocked reads (e.g., macOS logd holding .tracev3 files).
        const PER_FILE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

        // Parallel file copy with rayon
        all_files.par_iter().for_each(|(src, rel_dest, category)| {
            if TRIAGE_CANCEL_FLAG.load(Ordering::Relaxed) {
                a_cancelled.store(true, Ordering::Relaxed);
                return;
            }

            let dest = output_path.join(rel_dest);

            const LARGE_FILE_THRESHOLD: u64 = 4 * 1024 * 1024; // 4 MB
            const CHUNK_SIZE: usize = 256 * 1024; // 256 KB

            let file_size = src.metadata().map(|m| m.len()).unwrap_or(0);

            // Skip files exceeding the size limit (e.g., huge .tracev3 files)
            if max_file_size > 0 && file_size > max_file_size {
                warn!(
                    "Skipping oversized file ({} bytes): {}",
                    file_size,
                    src.display()
                );
                a_skipped.fetch_add(1, Ordering::Relaxed);
                let mut details = lock_mutex_recover(shared_category_details.as_ref());
                if let Some(cat_result) = details.get_mut(category.as_str()) {
                    cat_result.files_skipped += 1;
                }
                return;
            }

            // Spawn the copy in a dedicated OS thread so a kernel-level blocked
            // read() (e.g., macOS logd holding .tracev3 files) doesn't permanently
            // freeze the rayon worker. We poll for completion with a timeout.
            let src_path = src.clone();
            let dest_path = dest.clone();
            let copy_limit = file_size; // snapshot — don't chase growing files
            let copy_handle = std::thread::spawn(move || -> Result<u64, std::io::Error> {
                if file_size <= LARGE_FILE_THRESHOLD {
                    std::fs::copy(&src_path, &dest_path)
                } else {
                    // Chunked copy capped at the snapshot file size
                    use std::io::{Read, Write};
                    let mut reader = std::fs::File::open(&src_path)?;
                    let mut writer = std::fs::File::create(&dest_path)?;
                    let mut buf = vec![0u8; CHUNK_SIZE];
                    let mut copied: u64 = 0;
                    loop {
                        let remaining = copy_limit.saturating_sub(copied) as usize;
                        if remaining == 0 {
                            break;
                        }
                        let to_read = CHUNK_SIZE.min(remaining);
                        let n = reader.read(&mut buf[..to_read])?;
                        if n == 0 {
                            break;
                        }
                        writer.write_all(&buf[..n])?;
                        copied += n as u64;
                    }
                    Ok(copied)
                }
            });

            // Poll for thread completion with timeout + cancel check.
            // If the thread is stuck in a blocked syscall, we abandon it.
            let poll_start = std::time::Instant::now();
            let copy_result: Result<u64, std::io::Error> = loop {
                if copy_handle.is_finished() {
                    break copy_handle
                        .join()
                        .unwrap_or_else(|_| Err(std::io::Error::other("copy thread panicked")));
                }
                if TRIAGE_CANCEL_FLAG.load(Ordering::Relaxed) {
                    a_cancelled.store(true, Ordering::Relaxed);
                    let _ = std::fs::remove_file(&dest);
                    break Err(std::io::Error::new(
                        std::io::ErrorKind::Interrupted,
                        "cancelled",
                    ));
                }
                if poll_start.elapsed() >= PER_FILE_TIMEOUT {
                    warn!(
                        "Triage: copy stuck after {}s, abandoning: {}",
                        PER_FILE_TIMEOUT.as_secs(),
                        src.display()
                    );
                    // Thread is stuck in a kernel syscall — we can't kill it
                    // but we can move on. Clean up any partial output.
                    let _ = std::fs::remove_file(&dest);
                    break Err(std::io::Error::new(
                        std::io::ErrorKind::TimedOut,
                        "copy stuck in blocked I/O",
                    ));
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            };

            match copy_result {
                Ok(size) => {
                    a_collected.fetch_add(1, Ordering::Relaxed);
                    a_bytes.fetch_add(size, Ordering::Relaxed);
                    let mut details = lock_mutex_recover(shared_category_details.as_ref());
                    if let Some(cat_result) = details.get_mut(category.as_str()) {
                        cat_result.files_collected += 1;
                        cat_result.bytes_collected += size;
                        if cat_result.sample_files.len() < 10 {
                            if let Some(name) = src.file_name() {
                                cat_result
                                    .sample_files
                                    .push(name.to_string_lossy().to_string());
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to copy {}: {e}", src.display());
                    a_skipped.fetch_add(1, Ordering::Relaxed);
                    let mut details = lock_mutex_recover(shared_category_details.as_ref());
                    if let Some(cat_result) = details.get_mut(category.as_str()) {
                        cat_result.files_skipped += 1;
                    }
                }
            }

            // Time-based progress emission (~200ms interval)
            if let Ok(mut last) = last_emit.try_lock() {
                if last.elapsed() >= std::time::Duration::from_millis(200) {
                    *last = std::time::Instant::now();
                    let done = a_collected.load(Ordering::Relaxed)
                        + a_skipped.load(Ordering::Relaxed)
                        + a_failed.load(Ordering::Relaxed);
                    let pct = if total > 0 {
                        (done as f64 / total as f64) * 90.0
                    } else {
                        0.0
                    };
                    let _ = window.emit(
                        "triage-progress",
                        TriageProgress {
                            phase: "collecting".to_string(),
                            current_file: src
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_default(),
                            files_collected: a_collected.load(Ordering::Relaxed),
                            files_total: total,
                            bytes_collected: a_bytes.load(Ordering::Relaxed),
                            percent: pct,
                            current_category: category.clone(),
                        },
                    );
                }
            }
        });

        // Collect final counters
        let collected = a_collected.load(Ordering::Relaxed);
        let bytes = a_bytes.load(Ordering::Relaxed);
        let skipped = a_skipped.load(Ordering::Relaxed);
        let failed = a_failed.load(Ordering::Relaxed);
        category_details = into_inner_recover(
            Arc::try_unwrap(shared_category_details)
                .unwrap_or_else(|arc| Mutex::new(lock_mutex_recover(arc.as_ref()).clone())),
        );

        if a_cancelled.load(Ordering::Relaxed) {
            info!("Triage collection cancelled during parallel copy");
            return Ok(TriageResult {
                output_dir: options.output_dir,
                files_collected: collected,
                bytes_collected: bytes,
                files_skipped: skipped,
                files_failed: failed,
                duration_secs: started.elapsed().as_secs_f64(),
                categories_collected: options.categories,
                category_details,
                secret_findings: vec![],
                cancelled: true,
                container_path: None,
            });
        }

        // Write manifest CSV to output directory
        if collected > 0 {
            if let Err(e) = write_triage_manifest(output_path) {
                warn!("Failed to write triage manifest: {e}");
            }
        }

        // Secret scanning phase
        let mut findings = Vec::new();
        if options.scan_for_secrets && collected > 0 {
            let _ = window.emit(
                "triage-progress",
                TriageProgress {
                    phase: "scanning".to_string(),
                    current_file: String::new(),
                    files_collected: collected,
                    files_total: total,
                    bytes_collected: bytes,
                    percent: 90.0,
                    current_category: "secrets".to_string(),
                },
            );

            findings = scan_for_secrets(output_path, &window, collected);
            info!("Secret scan found {} potential findings", findings.len());
        }

        let duration = started.elapsed().as_secs_f64();

        // Container packaging phase (7z)
        let container_path = if let Some(ref fmt) = options.container_format {
            if fmt == "7z" && collected > 0 {
                let _ = window.emit(
                    "triage-progress",
                    TriageProgress {
                        phase: "packaging".to_string(),
                        current_file: String::new(),
                        files_collected: collected,
                        files_total: total,
                        bytes_collected: bytes,
                        percent: 95.0,
                        current_category: String::new(),
                    },
                );

                // Build output archive path alongside the staging directory
                let staging_name = output_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "triage".to_string());
                let archive_name = format!("{}.7z", staging_name);
                let archive_path = output_path
                    .parent()
                    .unwrap_or(output_path)
                    .join(&archive_name);

                match package_to_7z(output_path, &archive_path) {
                    Ok(()) => {
                        info!("Triage packaged to: {}", archive_path.display());
                        // Clean up staging directory after successful packaging
                        if let Err(e) = std::fs::remove_dir_all(output_path) {
                            warn!("Failed to clean staging dir: {e}");
                        }
                        Some(archive_path.to_string_lossy().to_string())
                    }
                    Err(e) => {
                        warn!("Failed to package triage to 7z: {e}");
                        // Leave staging dir intact on failure
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

        // Final progress
        let _ = window.emit(
            "triage-progress",
            TriageProgress {
                phase: "complete".to_string(),
                current_file: String::new(),
                files_collected: collected,
                files_total: total,
                bytes_collected: bytes,
                percent: 100.0,
                current_category: String::new(),
            },
        );

        info!(
        "Triage complete: {} files ({} bytes) in {:.1}s, {} skipped, {} failed, {} secrets found",
        collected,
        bytes,
        started.elapsed().as_secs_f64(),
        skipped,
        failed,
        findings.len()
    );

        Ok(TriageResult {
            output_dir: options.output_dir,
            files_collected: collected,
            bytes_collected: bytes,
            files_skipped: skipped,
            files_failed: failed,
            duration_secs: started.elapsed().as_secs_f64(),
            categories_collected: options.categories,
            category_details,
            secret_findings: findings,
            cancelled: false,
            container_path,
        })
    }) // end spawn_blocking
    .await
    .map_err(|e| format!("Triage task panicked: {e}"))?
}

/// Cancel an in-progress triage collection.
#[tauri::command]
pub async fn triage_cancel() -> Result<(), String> {
    TRIAGE_CANCEL_FLAG.store(true, Ordering::Relaxed);
    info!("Triage cancel requested");
    Ok(())
}

// =============================================================================
// Helpers
// =============================================================================

fn lock_mutex_recover<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            warn!("Triage mutex poisoned, recovering inner state");
            poisoned.into_inner()
        }
    }
}

fn into_inner_recover<T>(mutex: Mutex<T>) -> T {
    match mutex.into_inner() {
        Ok(value) => value,
        Err(poisoned) => {
            warn!("Triage mutex poisoned during collection, recovering inner state");
            poisoned.into_inner()
        }
    }
}

/// Package the staging directory contents into a 7z archive.
fn package_to_7z(staging_dir: &Path, archive_path: &Path) -> Result<(), String> {
    let sz =
        seven_zip::SevenZip::new().map_err(|e| format!("Failed to initialize 7z library: {e}"))?;
    let input = staging_dir.to_string_lossy().to_string();
    let stream_opts = StreamOptions {
        solid: false,
        ..StreamOptions::default()
    };
    sz.create_archive_streaming(
        archive_path,
        &[&input],
        seven_zip::CompressionLevel::Store,
        Some(&stream_opts),
        None,
    )
    .map_err(|e| format!("7z archive creation failed: {e}"))
}

/// Write a `triage_manifest.csv` to the output directory listing all collected files.
///
/// Walks the output directory recursively, writing one row per file with:
/// relative_path, category (first path component), size in bytes, and last modified time.
fn write_triage_manifest(output_dir: &Path) -> Result<(), String> {
    use std::io::Write;

    let manifest_path = output_dir.join("triage_manifest.csv");
    let mut file = std::fs::File::create(&manifest_path)
        .map_err(|e| format!("Cannot create manifest: {e}"))?;

    writeln!(file, "relative_path,category,size_bytes,modified")
        .map_err(|e| format!("Write header failed: {e}"))?;

    fn walk_dir(dir: &Path, base: &Path, out: &mut std::fs::File) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk_dir(&path, base, out);
            } else if path.is_file() {
                // Skip the manifest file itself
                if path
                    .file_name()
                    .map(|n| n == "triage_manifest.csv")
                    .unwrap_or(false)
                {
                    continue;
                }
                let rel = path.strip_prefix(base).unwrap_or(&path);
                let rel_str = rel.to_string_lossy().replace(',', ";"); // escape commas
                let category = rel
                    .components()
                    .next()
                    .map(|c| c.as_os_str().to_string_lossy().to_string())
                    .unwrap_or_default();
                let meta = std::fs::metadata(&path);
                let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
                let modified = meta
                    .as_ref()
                    .ok()
                    .and_then(|m| m.modified().ok())
                    .and_then(|t| {
                        let dur = t.duration_since(std::time::UNIX_EPOCH).ok()?;
                        let secs = dur.as_secs() as i64;
                        // Format as ISO 8601 (basic — no chrono dependency)
                        Some(format!("{secs}"))
                    })
                    .unwrap_or_default();
                let _ = writeln!(out, "{rel_str},{category},{size},{modified}");
            }
        }
    }

    walk_dir(output_dir, output_dir, &mut file);
    info!("Triage manifest written: {}", manifest_path.display());
    Ok(())
}

fn get_default_root() -> String {
    #[cfg(target_os = "windows")]
    {
        "C:".to_string()
    }

    #[cfg(target_os = "macos")]
    {
        "/".to_string()
    }

    #[cfg(target_os = "linux")]
    {
        "/".to_string()
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        "/".to_string()
    }
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
fn scan_for_secrets(
    output_dir: &Path,
    window: &tauri::Window,
    total_collected: u64,
) -> Vec<SecretFinding> {
    let mut findings = Vec::new();
    let patterns = &*SECRET_PATTERNS;
    let mut scanned: u64 = 0;
    let mut last_emit = std::time::Instant::now();

    // Walk the output directory for scannable text files
    let mut stack = vec![output_dir.to_path_buf()];
    while let Some(dir) = stack.pop() {
        if TRIAGE_CANCEL_FLAG.load(Ordering::Relaxed) {
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
                scanned += 1;
                let rel_path = path
                    .strip_prefix(output_dir)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .to_string();

                for (line_num, line) in content.lines().enumerate() {
                    for (pattern, secret_type, description, confidence) in patterns {
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

                // Time-based progress during secret scanning (~200ms)
                if last_emit.elapsed() >= std::time::Duration::from_millis(200) {
                    last_emit = std::time::Instant::now();
                    let pct = if total_collected > 0 {
                        90.0 + (scanned as f64 / total_collected as f64) * 10.0
                    } else {
                        95.0
                    };
                    let _ = window.emit(
                        "triage-progress",
                        TriageProgress {
                            phase: "scanning".to_string(),
                            current_file: rel_path,
                            files_collected: total_collected,
                            files_total: total_collected,
                            bytes_collected: 0,
                            percent: pct.min(99.0),
                            current_category: "secrets".to_string(),
                        },
                    );
                }
            }
        }
    }

    findings
}

/// Check if a file is likely text/config (worth scanning for secrets).
fn is_scannable_file(path: &Path) -> bool {
    let scannable_extensions = [
        "txt",
        "log",
        "cfg",
        "conf",
        "config",
        "ini",
        "env",
        "yaml",
        "yml",
        "json",
        "xml",
        "toml",
        "properties",
        "pem",
        "key",
        "pub",
        "crt",
        "cer",
        "pfx",
        "p12",
        "plist",
        "sh",
        "bash",
        "zsh",
        "ps1",
        "bat",
        "cmd",
        "py",
        "rb",
        "js",
        "ts",
        "php",
        "sql",
        "csv",
        "md",
        "credentials",
        "gitconfig",
        "npmrc",
        "netrc",
        "pgpass",
    ];

    // Also scan files without extensions (e.g., `credentials`, `config`, `known_hosts`)
    let scannable_names = [
        "credentials",
        "config",
        "known_hosts",
        "authorized_keys",
        "id_rsa",
        "id_ed25519",
        "id_ecdsa",
        "id_dsa",
        "shadow",
        "passwd",
        "hosts",
        "resolv.conf",
        ".env",
        ".envrc",
        ".bashrc",
        ".zshrc",
        ".profile",
        "ConsoleHost_history.txt",
        "history",
    ];

    if let Some(ext) = path.extension() {
        if scannable_extensions.contains(&ext.to_string_lossy().to_lowercase().as_str()) {
            return true;
        }
    }

    if let Some(name) = path.file_name() {
        let name_str = name.to_string_lossy().to_lowercase();
        if scannable_names
            .iter()
            .any(|n| name_str == n.to_lowercase() || name_str.ends_with(n))
        {
            return true;
        }
    }

    false
}

/// Build regex patterns for detecting secrets. Returns (regex, type, description, confidence).
/// Called once via LazyLock — patterns are compiled at first use and reused.
fn build_secret_patterns() -> Vec<(Regex, &'static str, &'static str, &'static str)> {
    let mut patterns = Vec::new();

    // Helper to add pattern, skipping invalid regexes
    let mut add = |pat: &str, stype: &'static str, desc: &'static str, conf: &'static str| {
        if let Ok(re) = Regex::new(pat) {
            patterns.push((re, stype, desc, conf));
        }
    };

    // ── AWS ─────────────────────────────────────────────────────────
    add(
        r"(?i)AKIA[0-9A-Z]{16}",
        "aws_access_key",
        "AWS Access Key ID",
        "high",
    );
    add(
        r"(?i)aws[_\-]?secret[_\-]?access[_\-]?key\s*[=:]\s*\S+",
        "aws_secret_key",
        "AWS Secret Access Key",
        "high",
    );

    // ── Private Keys ────────────────────────────────────────────────
    add(
        r"-----BEGIN\s+(RSA\s+)?PRIVATE\s+KEY-----",
        "private_key",
        "RSA/Generic Private Key (PEM)",
        "high",
    );
    add(
        r"-----BEGIN\s+EC\s+PRIVATE\s+KEY-----",
        "ec_private_key",
        "EC Private Key (PEM)",
        "high",
    );
    add(
        r"-----BEGIN\s+OPENSSH\s+PRIVATE\s+KEY-----",
        "openssh_key",
        "OpenSSH Private Key",
        "high",
    );
    add(
        r"-----BEGIN\s+PGP\s+PRIVATE\s+KEY\s+BLOCK-----",
        "pgp_key",
        "PGP Private Key",
        "high",
    );
    add(
        r"-----BEGIN\s+ENCRYPTED\s+PRIVATE\s+KEY-----",
        "encrypted_key",
        "Encrypted Private Key (PEM)",
        "medium",
    );

    // ── Generic API / Tokens ────────────────────────────────────────
    add(
        r#"(?i)(api[_\-]?key|api[_\-]?secret|api[_\-]?token)\s*[=:]\s*['"]?[A-Za-z0-9\-_]{20,}['"]?"#,
        "api_key",
        "Generic API Key/Token",
        "medium",
    );
    add(
        r#"(?i)(access[_\-]?token|auth[_\-]?token|bearer[_\-]?token)\s*[=:]\s*['"]?[A-Za-z0-9\-_.]{20,}['"]?"#,
        "access_token",
        "Access/Auth/Bearer Token",
        "medium",
    );
    add(
        r"(?i)bearer\s+[A-Za-z0-9\-_.]{20,}",
        "bearer_token",
        "Bearer Token in Header",
        "high",
    );

    // ── Connection Strings ──────────────────────────────────────────
    add(
        r#"(?i)(password|passwd|pwd)\s*[=:]\s*['"]?[^\s'"]{4,}['"]?"#,
        "password",
        "Password in Configuration",
        "medium",
    );
    add(
        r"(?i)mysql://\S+:\S+@",
        "mysql_conn",
        "MySQL Connection String with Credentials",
        "high",
    );
    add(
        r"(?i)postgres(ql)?://\S+:\S+@",
        "postgres_conn",
        "PostgreSQL Connection String with Credentials",
        "high",
    );
    add(
        r"(?i)mongodb(\+srv)?://\S+:\S+@",
        "mongodb_conn",
        "MongoDB Connection String with Credentials",
        "high",
    );

    // ── Cloud Provider ──────────────────────────────────────────────
    add(
        r"(?i)gcp[_\-]?service[_\-]?account|type.*service_account",
        "gcp_service_account",
        "GCP Service Account Key",
        "medium",
    );
    add(
        r#""client_secret"\s*:\s*"[A-Za-z0-9\-_]{20,}""#,
        "oauth_secret",
        "OAuth Client Secret",
        "high",
    );

    // ── GitHub / GitLab ─────────────────────────────────────────────
    add(
        r"gh[pousr]_[A-Za-z0-9_]{36,}",
        "github_token",
        "GitHub Personal Access Token",
        "high",
    );
    add(
        r"glpat-[A-Za-z0-9\-_]{20,}",
        "gitlab_token",
        "GitLab Personal Access Token",
        "high",
    );

    // ── Slack / Discord ─────────────────────────────────────────────
    add(
        r"xox[baprs]-[0-9A-Za-z\-]{10,}",
        "slack_token",
        "Slack Token",
        "high",
    );

    // ── Stripe ──────────────────────────────────────────────────────
    add(
        r"sk_live_[0-9a-zA-Z]{24,}",
        "stripe_secret",
        "Stripe Secret Key",
        "high",
    );
    add(
        r"rk_live_[0-9a-zA-Z]{24,}",
        "stripe_restricted",
        "Stripe Restricted Key",
        "high",
    );

    // ── SendGrid / Twilio ───────────────────────────────────────────
    add(
        r"SG\.[A-Za-z0-9\-_]{22,}\.[A-Za-z0-9\-_]{22,}",
        "sendgrid_key",
        "SendGrid API Key",
        "high",
    );

    // ── SSH / Known Hosts ───────────────────────────────────────────
    add(
        r"(?i)ssh-rsa\s+AAAA[0-9A-Za-z+/]{100,}",
        "ssh_public_key",
        "SSH RSA Public Key",
        "low",
    );

    // ── Encryption Keys / Passphrases ───────────────────────────────
    add(
        r#"(?i)(encryption[_\-]?key|encrypt[_\-]?secret|aes[_\-]?key|master[_\-]?key)\s*[=:]\s*['"]?[A-Fa-f0-9]{32,}['"]?"#,
        "encryption_key",
        "Encryption Key (hex)",
        "high",
    );
    add(
        r#"(?i)(passphrase|pass_phrase)\s*[=:]\s*['"]?[^\s'"]{4,}['"]?"#,
        "passphrase",
        "Passphrase in Configuration",
        "medium",
    );

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

#[cfg(test)]
mod tests {
    use super::{into_inner_recover, lock_mutex_recover};
    use std::sync::{Arc, Mutex};

    #[test]
    fn lock_mutex_recover_reads_poisoned_mutex() {
        let value = Arc::new(Mutex::new(41_u32));
        let worker_value = Arc::clone(&value);

        let _ = std::thread::spawn(move || {
            let _guard = worker_value.lock().unwrap();
            panic!("poison lock");
        })
        .join();

        assert!(value.is_poisoned());
        let guard = lock_mutex_recover(value.as_ref());
        assert_eq!(*guard, 41);
    }

    #[test]
    fn into_inner_recover_returns_poisoned_value() {
        let value = Arc::new(Mutex::new(7_u32));
        let worker_value = Arc::clone(&value);

        let _ = std::thread::spawn(move || {
            let _guard = worker_value.lock().unwrap();
            panic!("poison lock");
        })
        .join();

        assert!(value.is_poisoned());
        let value = Arc::try_unwrap(value).expect("single owner after join");
        assert_eq!(into_inner_recover(value), 7);
    }
}
