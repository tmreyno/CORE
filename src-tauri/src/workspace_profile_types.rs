// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Type definitions for workspace profiles.
//!
//! All data structures used by the workspace profile system:
//! `ProfileType`, `WorkspaceProfile`, `LayoutConfig`, `CenterLayout`,
//! `ToolConfig`, `FilterPreset`, `ViewSettings`, `QuickAction`,
//! `ProfileManager`, and `ProfileSummary`.

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
