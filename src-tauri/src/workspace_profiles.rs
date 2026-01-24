// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Workspace profiles for different investigation scenarios.
//!
//! Provides:
//! - Named workspace profiles (Investigation, Analysis, Review, etc.)
//! - Profile-specific layouts and tool configurations
//! - Saved filter presets per profile
//! - Quick profile switching with state preservation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Workspace profile types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ProfileType {
    /// General investigation profile
    Investigation,
    /// Deep analysis profile with advanced tools
    Analysis,
    /// Case review and documentation profile
    Review,
    /// Mobile forensics optimized
    Mobile,
    /// Computer forensics optimized
    Computer,
    /// Network forensics optimized
    Network,
    /// Incident response profile
    IncidentResponse,
    /// Custom user-defined profile
    Custom,
}

impl ProfileType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProfileType::Investigation => "Investigation",
            ProfileType::Analysis => "Analysis",
            ProfileType::Review => "Review",
            ProfileType::Mobile => "Mobile",
            ProfileType::Computer => "Computer",
            ProfileType::Network => "Network",
            ProfileType::IncidentResponse => "Incident Response",
            ProfileType::Custom => "Custom",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ProfileType::Investigation => "General purpose investigation workspace",
            ProfileType::Analysis => "Advanced analysis with all tools enabled",
            ProfileType::Review => "Case review and documentation focused",
            ProfileType::Mobile => "Optimized for mobile device forensics",
            ProfileType::Computer => "Optimized for computer forensics",
            ProfileType::Network => "Optimized for network forensics",
            ProfileType::IncidentResponse => "Rapid incident response workflow",
            ProfileType::Custom => "Custom user-defined workspace",
        }
    }
}

/// Complete workspace profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceProfile {
    /// Profile ID
    pub id: String,
    /// Profile name
    pub name: String,
    /// Profile type
    pub profile_type: ProfileType,
    /// Profile description
    pub description: String,
    /// Creation timestamp
    pub created_at: String,
    /// Last used timestamp
    pub last_used: String,
    /// Usage count
    pub usage_count: usize,
    /// Layout configuration
    pub layout: LayoutConfig,
    /// Tool configurations
    pub tools: ToolConfig,
    /// Filter presets
    pub filters: Vec<FilterPreset>,
    /// View settings
    pub view_settings: ViewSettings,
    /// Quick actions
    pub quick_actions: Vec<QuickAction>,
    /// Keyboard shortcuts
    pub shortcuts: HashMap<String, String>,
    /// Custom metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Layout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConfig {
    /// Left panel width (pixels)
    pub left_panel_width: u32,
    /// Right panel width (pixels)
    pub right_panel_width: u32,
    /// Bottom panel height (pixels)
    pub bottom_panel_height: u32,
    /// Left panel collapsed
    pub left_panel_collapsed: bool,
    /// Right panel collapsed
    pub right_panel_collapsed: bool,
    /// Bottom panel collapsed
    pub bottom_panel_collapsed: bool,
    /// Active left panel tab
    pub left_panel_tab: String,
    /// Active right panel tab
    pub right_panel_tab: String,
    /// Active bottom panel tab
    pub bottom_panel_tab: String,
    /// Center pane layout (single, split-vertical, split-horizontal)
    pub center_layout: CenterLayout,
}

/// Center pane layout mode
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CenterLayout {
    Single,
    SplitVertical,
    SplitHorizontal,
    Grid,
}

/// Tool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    /// Enabled tools list
    pub enabled_tools: Vec<String>,
    /// Tool-specific settings
    pub tool_settings: HashMap<String, serde_json::Value>,
    /// Default hash algorithms
    pub default_hash_algorithms: Vec<String>,
    /// Auto-hash on open
    pub auto_hash: bool,
    /// Auto-verify checksums
    pub auto_verify: bool,
    /// Default export format
    pub default_export_format: String,
    /// Show hex viewer by default
    pub show_hex_viewer: bool,
    /// Show metadata panel
    pub show_metadata: bool,
}

/// Filter preset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterPreset {
    /// Preset ID
    pub id: String,
    /// Preset name
    pub name: String,
    /// Description
    pub description: String,
    /// File type filters
    pub file_types: Vec<String>,
    /// Extension filters
    pub extensions: Vec<String>,
    /// Size filters (min, max in bytes)
    pub size_range: Option<(u64, u64)>,
    /// Date filters
    pub date_range: Option<(String, String)>,
    /// Search terms
    pub search_terms: Vec<String>,
    /// Include hidden files
    pub include_hidden: bool,
    /// Include system files
    pub include_system: bool,
}

/// View settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewSettings {
    /// Theme (light, dark, auto)
    pub theme: String,
    /// Font size
    pub font_size: u32,
    /// Show hidden files
    pub show_hidden_files: bool,
    /// Show file extensions
    pub show_file_extensions: bool,
    /// Tree indent size
    pub tree_indent: u32,
    /// Icon size
    pub icon_size: u32,
    /// Detail view mode
    pub detail_view_mode: String,
    /// Thumbnail size
    pub thumbnail_size: u32,
}

/// Quick action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickAction {
    /// Action ID
    pub id: String,
    /// Action name
    pub name: String,
    /// Action icon
    pub icon: String,
    /// Command to execute
    pub command: String,
    /// Keyboard shortcut
    pub shortcut: Option<String>,
}

/// Profile manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileManager {
    /// All profiles
    pub profiles: Vec<WorkspaceProfile>,
    /// Active profile ID
    pub active_profile_id: Option<String>,
    /// Default profile ID
    pub default_profile_id: Option<String>,
}

impl ProfileManager {
    /// Create new profile manager with default profiles
    pub fn new() -> Self {
        let mut manager = ProfileManager {
            profiles: Vec::new(),
            active_profile_id: None,
            default_profile_id: None,
        };

        // Create default profiles
        manager.profiles.push(Self::create_investigation_profile());
        manager.profiles.push(Self::create_analysis_profile());
        manager.profiles.push(Self::create_review_profile());
        manager.profiles.push(Self::create_mobile_profile());
        manager.profiles.push(Self::create_computer_profile());

        // Set default
        if let Some(first) = manager.profiles.first() {
            manager.default_profile_id = Some(first.id.clone());
            manager.active_profile_id = Some(first.id.clone());
        }

        manager
    }

    /// Create investigation profile
    fn create_investigation_profile() -> WorkspaceProfile {
        WorkspaceProfile {
            id: "investigation".to_string(),
            name: "Investigation".to_string(),
            profile_type: ProfileType::Investigation,
            description: "General purpose investigation workspace".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            last_used: chrono::Utc::now().to_rfc3339(),
            usage_count: 0,
            layout: LayoutConfig {
                left_panel_width: 300,
                right_panel_width: 350,
                bottom_panel_height: 200,
                left_panel_collapsed: false,
                right_panel_collapsed: false,
                bottom_panel_collapsed: true,
                left_panel_tab: "evidence".to_string(),
                right_panel_tab: "details".to_string(),
                bottom_panel_tab: "activity".to_string(),
                center_layout: CenterLayout::Single,
            },
            tools: ToolConfig {
                enabled_tools: vec![
                    "hash".to_string(),
                    "viewer".to_string(),
                    "bookmarks".to_string(),
                    "notes".to_string(),
                    "search".to_string(),
                ],
                tool_settings: HashMap::new(),
                default_hash_algorithms: vec!["SHA-256".to_string(), "MD5".to_string()],
                auto_hash: false,
                auto_verify: true,
                default_export_format: "json".to_string(),
                show_hex_viewer: true,
                show_metadata: true,
            },
            filters: vec![
                Self::create_filter_preset(
                    "documents",
                    "Document Files",
                    vec!["DOC", "DOCX", "PDF", "TXT"],
                ),
                Self::create_filter_preset(
                    "images",
                    "Image Files",
                    vec!["JPG", "JPEG", "PNG", "GIF", "BMP"],
                ),
            ],
            view_settings: ViewSettings {
                theme: "auto".to_string(),
                font_size: 14,
                show_hidden_files: true,
                show_file_extensions: true,
                tree_indent: 16,
                icon_size: 20,
                detail_view_mode: "list".to_string(),
                thumbnail_size: 128,
            },
            quick_actions: vec![
                QuickAction {
                    id: "hash_file".to_string(),
                    name: "Hash File".to_string(),
                    icon: "hash".to_string(),
                    command: "hash_compute".to_string(),
                    shortcut: Some("Ctrl+H".to_string()),
                },
                QuickAction {
                    id: "bookmark".to_string(),
                    name: "Add Bookmark".to_string(),
                    icon: "bookmark".to_string(),
                    command: "bookmark_add".to_string(),
                    shortcut: Some("Ctrl+B".to_string()),
                },
            ],
            shortcuts: HashMap::from([
                ("save".to_string(), "Ctrl+S".to_string()),
                ("search".to_string(), "Ctrl+F".to_string()),
                ("hash".to_string(), "Ctrl+H".to_string()),
            ]),
            metadata: HashMap::new(),
        }
    }

    /// Create analysis profile
    fn create_analysis_profile() -> WorkspaceProfile {
        let mut profile = Self::create_investigation_profile();
        profile.id = "analysis".to_string();
        profile.name = "Analysis".to_string();
        profile.profile_type = ProfileType::Analysis;
        profile.description = "Advanced analysis with all tools enabled".to_string();

        // Analysis-specific layout: all panels visible
        profile.layout.left_panel_collapsed = false;
        profile.layout.right_panel_collapsed = false;
        profile.layout.bottom_panel_collapsed = false;
        profile.layout.center_layout = CenterLayout::SplitVertical;

        // Enable all tools
        profile.tools.enabled_tools = vec![
            "hash".to_string(),
            "viewer".to_string(),
            "hex".to_string(),
            "entropy".to_string(),
            "strings".to_string(),
            "bookmarks".to_string(),
            "notes".to_string(),
            "search".to_string(),
            "carve".to_string(),
            "timeline".to_string(),
        ];
        profile.tools.default_hash_algorithms = vec![
            "SHA-256".to_string(),
            "SHA-1".to_string(),
            "MD5".to_string(),
            "BLAKE3".to_string(),
        ];
        profile.tools.auto_hash = true;
        profile.tools.show_hex_viewer = true;

        profile
    }

    /// Create review profile
    fn create_review_profile() -> WorkspaceProfile {
        let mut profile = Self::create_investigation_profile();
        profile.id = "review".to_string();
        profile.name = "Review".to_string();
        profile.profile_type = ProfileType::Review;
        profile.description = "Case review and documentation focused".to_string();

        // Review-specific layout: focus on notes and bookmarks
        profile.layout.right_panel_width = 450;
        profile.layout.right_panel_tab = "notes".to_string();

        profile.tools.enabled_tools = vec![
            "viewer".to_string(),
            "bookmarks".to_string(),
            "notes".to_string(),
            "search".to_string(),
            "report".to_string(),
        ];
        profile.tools.show_metadata = true;

        // Add review-specific filters
        profile.filters.push(FilterPreset {
            id: "bookmarked".to_string(),
            name: "Bookmarked Files".to_string(),
            description: "Show only bookmarked files".to_string(),
            file_types: Vec::new(),
            extensions: Vec::new(),
            size_range: None,
            date_range: None,
            search_terms: Vec::new(),
            include_hidden: true,
            include_system: true,
        });

        profile
    }

    /// Create mobile forensics profile
    fn create_mobile_profile() -> WorkspaceProfile {
        let mut profile = Self::create_investigation_profile();
        profile.id = "mobile".to_string();
        profile.name = "Mobile Forensics".to_string();
        profile.profile_type = ProfileType::Mobile;
        profile.description = "Optimized for mobile device forensics".to_string();

        // Mobile-specific filters
        profile.filters = vec![
            Self::create_filter_preset(
                "databases",
                "Database Files",
                vec!["DB", "SQLITE", "SQLITEDB"],
            ),
            Self::create_filter_preset(
                "plists",
                "Property Lists",
                vec!["PLIST", "BPLIST"],
            ),
            Self::create_filter_preset(
                "app_data",
                "App Data",
                vec!["XML", "JSON", "PLIST"],
            ),
        ];

        profile.tools.enabled_tools.push("plist_viewer".to_string());
        profile.tools.enabled_tools.push("sqlite_viewer".to_string());

        profile
    }

    /// Create computer forensics profile
    fn create_computer_profile() -> WorkspaceProfile {
        let mut profile = Self::create_investigation_profile();
        profile.id = "computer".to_string();
        profile.name = "Computer Forensics".to_string();
        profile.profile_type = ProfileType::Computer;
        profile.description = "Optimized for computer forensics".to_string();

        // Computer-specific filters
        profile.filters = vec![
            Self::create_filter_preset(
                "executables",
                "Executable Files",
                vec!["EXE", "DLL", "SYS", "APP"],
            ),
            Self::create_filter_preset(
                "system",
                "System Files",
                vec!["REG", "LOG", "EVT", "EVTX"],
            ),
            Self::create_filter_preset(
                "scripts",
                "Script Files",
                vec!["PS1", "BAT", "CMD", "SH", "PY"],
            ),
        ];

        profile.tools.enabled_tools.push("registry_viewer".to_string());
        profile.tools.enabled_tools.push("event_log_viewer".to_string());

        profile
    }

    /// Create filter preset helper
    fn create_filter_preset(id: &str, name: &str, extensions: Vec<&str>) -> FilterPreset {
        FilterPreset {
            id: id.to_string(),
            name: name.to_string(),
            description: format!("Filter for {}", name.to_lowercase()),
            file_types: Vec::new(),
            extensions: extensions.iter().map(|s| s.to_string()).collect(),
            size_range: None,
            date_range: None,
            search_terms: Vec::new(),
            include_hidden: false,
            include_system: false,
        }
    }

    /// Get profile by ID
    pub fn get_profile(&self, id: &str) -> Option<&WorkspaceProfile> {
        self.profiles.iter().find(|p| p.id == id)
    }

    /// Get active profile
    pub fn get_active_profile(&self) -> Option<&WorkspaceProfile> {
        self.active_profile_id
            .as_ref()
            .and_then(|id| self.get_profile(id))
    }

    /// Set active profile
    pub fn set_active_profile(&mut self, id: &str) -> Result<(), String> {
        if self.profiles.iter().any(|p| p.id == id) {
            self.active_profile_id = Some(id.to_string());
            
            // Update usage stats
            if let Some(profile) = self.profiles.iter_mut().find(|p| p.id == id) {
                profile.usage_count += 1;
                profile.last_used = chrono::Utc::now().to_rfc3339();
            }
            
            Ok(())
        } else {
            Err(format!("Profile not found: {}", id))
        }
    }

    /// Add custom profile
    pub fn add_profile(&mut self, profile: WorkspaceProfile) {
        self.profiles.push(profile);
    }

    /// Update profile
    pub fn update_profile(&mut self, profile: WorkspaceProfile) -> Result<(), String> {
        if let Some(existing) = self.profiles.iter_mut().find(|p| p.id == profile.id) {
            *existing = profile;
            Ok(())
        } else {
            Err("Profile not found".to_string())
        }
    }

    /// Delete profile
    pub fn delete_profile(&mut self, id: &str) -> Result<(), String> {
        let index = self
            .profiles
            .iter()
            .position(|p| p.id == id)
            .ok_or("Profile not found")?;

        // Don't delete if it's the active or default profile
        if self.active_profile_id.as_deref() == Some(id) {
            return Err("Cannot delete active profile".to_string());
        }
        if self.default_profile_id.as_deref() == Some(id) {
            return Err("Cannot delete default profile".to_string());
        }

        self.profiles.remove(index);
        Ok(())
    }

    /// Clone profile to create new custom profile
    pub fn clone_profile(&mut self, source_id: &str, new_name: &str) -> Result<String, String> {
        let source = self
            .get_profile(source_id)
            .ok_or("Source profile not found")?
            .clone();

        let new_id = format!("custom_{}", uuid::Uuid::new_v4());
        let mut new_profile = source;
        new_profile.id = new_id.clone();
        new_profile.name = new_name.to_string();
        new_profile.profile_type = ProfileType::Custom;
        new_profile.created_at = chrono::Utc::now().to_rfc3339();
        new_profile.last_used = chrono::Utc::now().to_rfc3339();
        new_profile.usage_count = 0;

        self.profiles.push(new_profile);
        Ok(new_id)
    }

    /// Export profile to JSON
    pub fn export_profile(&self, id: &str) -> Result<String, String> {
        let profile = self.get_profile(id).ok_or("Profile not found")?;
        serde_json::to_string_pretty(profile).map_err(|e| e.to_string())
    }

    /// Import profile from JSON
    pub fn import_profile(&mut self, json: &str) -> Result<String, String> {
        let profile: WorkspaceProfile = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let id = profile.id.clone();
        self.profiles.push(profile);
        Ok(id)
    }

    /// List all profiles
    pub fn list_profiles(&self) -> Vec<ProfileSummary> {
        self.profiles
            .iter()
            .map(|p| ProfileSummary {
                id: p.id.clone(),
                name: p.name.clone(),
                profile_type: p.profile_type,
                description: p.description.clone(),
                last_used: p.last_used.clone(),
                usage_count: p.usage_count,
                is_active: self.active_profile_id.as_deref() == Some(&p.id),
                is_default: self.default_profile_id.as_deref() == Some(&p.id),
            })
            .collect()
    }
}

/// Profile summary for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSummary {
    pub id: String,
    pub name: String,
    pub profile_type: ProfileType,
    pub description: String,
    pub last_used: String,
    pub usage_count: usize,
    pub is_active: bool,
    pub is_default: bool,
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_manager_creation() {
        let manager = ProfileManager::new();
        assert_eq!(manager.profiles.len(), 5); // Default profiles
        assert!(manager.active_profile_id.is_some());
        assert!(manager.default_profile_id.is_some());
    }

    #[test]
    fn test_set_active_profile() {
        let mut manager = ProfileManager::new();
        let profile_id = manager.profiles[0].id.clone();
        assert!(manager.set_active_profile(&profile_id).is_ok());
        assert_eq!(manager.active_profile_id, Some(profile_id));
    }

    #[test]
    fn test_clone_profile() {
        let mut manager = ProfileManager::new();
        let source_id = manager.profiles[0].id.clone();
        let result = manager.clone_profile(&source_id, "My Custom Profile");
        assert!(result.is_ok());
        assert_eq!(manager.profiles.len(), 6);
    }

    #[test]
    fn test_profile_type_strings() {
        assert_eq!(ProfileType::Investigation.as_str(), "Investigation");
        assert_eq!(ProfileType::Mobile.as_str(), "Mobile");
    }
}
