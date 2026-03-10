// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Default workspace profile definitions.
//!
//! Contains the built-in profile builders: Investigation, Analysis, Review,
//! Mobile, and Computer forensics profiles with their tool, layout, and
//! filter preset configurations.

use std::collections::HashMap;

use super::workspace_profile_types::*;

impl Default for ProfileManager {
    fn default() -> Self {
        Self::new()
    }
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
                    icon: "fingerprint".to_string(),
                    command: "hash_selected".to_string(),
                    shortcut: Some("Ctrl+H".to_string()),
                },
                QuickAction {
                    id: "bookmark".to_string(),
                    name: "Add Bookmark".to_string(),
                    icon: "bookmark".to_string(),
                    command: "bookmark_add".to_string(),
                    shortcut: Some("Ctrl+B".to_string()),
                },
                QuickAction {
                    id: "evidence".to_string(),
                    name: "Evidence Collection".to_string(),
                    icon: "evidence".to_string(),
                    command: "evidence_collection".to_string(),
                    shortcut: None,
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
            Self::create_filter_preset("plists", "Property Lists", vec!["PLIST", "BPLIST"]),
            Self::create_filter_preset("app_data", "App Data", vec!["XML", "JSON", "PLIST"]),
        ];

        profile.tools.enabled_tools.push("plist_viewer".to_string());
        profile
            .tools
            .enabled_tools
            .push("sqlite_viewer".to_string());

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
            Self::create_filter_preset("system", "System Files", vec!["REG", "LOG", "EVT", "EVTX"]),
            Self::create_filter_preset(
                "scripts",
                "Script Files",
                vec!["PS1", "BAT", "CMD", "SH", "PY"],
            ),
        ];

        profile
            .tools
            .enabled_tools
            .push("registry_viewer".to_string());
        profile
            .tools
            .enabled_tools
            .push("event_log_viewer".to_string());

        profile
    }

    /// Create filter preset helper
    pub(crate) fn create_filter_preset(
        id: &str,
        name: &str,
        extensions: Vec<&str>,
    ) -> FilterPreset {
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
}
