// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Native menu bar configuration for CORE-FFX.
//!
//! Provides platform-native menus for macOS (app menu bar) and Windows (window menu bar)
//! with standard items: File, Edit, View, Window, Help.

use tauri::{
    menu::{AboutMetadata, MenuBuilder, MenuEvent, MenuItem, SubmenuBuilder},
    AppHandle, Emitter, Manager, Runtime, WebviewUrl, WebviewWindowBuilder,
};
use tracing::{error, info};

/// Build the application menu bar.
///
/// ## macOS
/// - CORE-FFX (app menu with About, Hide, Quit)
/// - File (New Window, Close Window)
/// - Edit (Undo, Redo, Cut, Copy, Paste, Select All)
/// - View (Fullscreen)
/// - Window (Minimize, Maximize)
/// - Help
///
/// ## Windows / Linux
/// - File (New Window, Close Window, Exit)
/// - Edit (Undo, Redo, Cut, Copy, Paste, Select All)
/// - View (Fullscreen)
/// - Window (Minimize, Maximize)
/// - Help (About)
pub fn build_menu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<tauri::menu::Menu<R>> {
    // =========================================================================
    // macOS App submenu (only shown on macOS)
    // =========================================================================
    #[cfg(target_os = "macos")]
    let app_menu = {
        SubmenuBuilder::new(app, "CORE-FFX")
            .about(Some(AboutMetadata {
                name: Some("CORE-FFX".into()),
                version: Some(env!("CARGO_PKG_VERSION").into()),
                copyright: Some("© 2024-2026 CORE-FFX Project Contributors".into()),
                license: Some("MIT License".into()),
                ..Default::default()
            }))
            .separator()
            .services()
            .separator()
            .hide()
            .hide_others()
            .show_all()
            .separator()
            .quit()
            .build()?
    };

    // =========================================================================
    // File submenu
    // =========================================================================
    let file_menu = {
        #[allow(unused_mut)]
        let mut builder = SubmenuBuilder::new(app, "File")
            .item(&MenuItem::with_id(
                app,
                "new-project",
                "New Project…",
                true,
                Some("CmdOrCtrl+Shift+N"),
            )?)
            .text("new-window", "New Window")
            .separator()
            .item(&MenuItem::with_id(
                app,
                "open-project",
                "Open Project…",
                true,
                Some("CmdOrCtrl+O"),
            )?)
            .item(&MenuItem::with_id(
                app,
                "open-directory",
                "Open Evidence Directory…",
                true,
                Some("CmdOrCtrl+Shift+O"),
            )?)
            .separator()
            .item(&MenuItem::with_id(
                app,
                "save-project",
                "Save Project",
                true,
                Some("CmdOrCtrl+S"),
            )?)
            .item(&MenuItem::with_id(
                app,
                "save-project-as",
                "Save Project As…",
                true,
                Some("CmdOrCtrl+Shift+S"),
            )?)
            .separator()
            .item(&MenuItem::with_id(
                app,
                "export",
                "Export…",
                true,
                Some("CmdOrCtrl+E"),
            )?)
            .separator()
            .text("scan-evidence", "Scan Evidence")
            .item(&MenuItem::with_id(
                app,
                "close-active-tab",
                "Close Tab",
                false,
                Some("CmdOrCtrl+W"),
            )?)
            .text("close-all-tabs", "Close All Tabs")
            .separator()
            .text("toggle-autosave", "Toggle Auto-Save")
            .separator()
            .close_window();

        // On non-macOS, add Quit/Exit to File menu
        #[cfg(not(target_os = "macos"))]
        {
            builder = builder.separator().quit();
        }

        builder.build()?
    };

    // =========================================================================
    // Edit submenu
    // =========================================================================
    let edit_menu = SubmenuBuilder::new(app, "Edit")
        .undo()
        .redo()
        .separator()
        .cut()
        .copy()
        .paste()
        .separator()
        .select_all()
        .separator()
        .item(&MenuItem::with_id(
            app,
            "select-all-evidence",
            "Select All Evidence",
            false,
            None::<&str>,
        )?)
        .build()?;

    // =========================================================================
    // View submenu
    // =========================================================================
    let view_menu = SubmenuBuilder::new(app, "View")
        .item(&MenuItem::with_id(
            app,
            "toggle-sidebar",
            "Toggle Sidebar",
            true,
            Some("CmdOrCtrl+B"),
        )?)
        .text("toggle-right-panel", "Toggle Right Panel")
        .text("toggle-quick-actions", "Toggle Quick Actions")
        .separator()
        .item(&MenuItem::with_id(
            app,
            "show-dashboard",
            "Dashboard",
            false,
            None::<&str>,
        )?)
        .item(&MenuItem::with_id(
            app,
            "show-evidence",
            "Evidence Panel",
            false,
            None::<&str>,
        )?)
        .item(&MenuItem::with_id(
            app,
            "show-casedocs",
            "Case Documents",
            false,
            None::<&str>,
        )?)
        .item(&MenuItem::with_id(
            app,
            "show-processed",
            "Processed Databases",
            false,
            None::<&str>,
        )?)
        .item(&MenuItem::with_id(
            app,
            "show-activity",
            "Activity Timeline",
            false,
            None::<&str>,
        )?)
        .item(&MenuItem::with_id(
            app,
            "show-bookmarks",
            "Bookmarks",
            false,
            None::<&str>,
        )?)
        .separator()
        .item(&MenuItem::with_id(
            app,
            "view-info",
            "Info View",
            false,
            Some("CmdOrCtrl+1"),
        )?)
        .item(&MenuItem::with_id(
            app,
            "view-hex",
            "Hex View",
            false,
            Some("CmdOrCtrl+2"),
        )?)
        .item(&MenuItem::with_id(
            app,
            "view-text",
            "Text View",
            false,
            Some("CmdOrCtrl+3"),
        )?)
        .separator()
        .text("cycle-theme", "Cycle Theme")
        .separator()
        .fullscreen()
        .build()?;

    // =========================================================================
    // Window submenu
    // =========================================================================
    let window_menu = SubmenuBuilder::new(app, "Window")
        .minimize()
        .maximize()
        .separator()
        .close_window()
        .build()?;

    // =========================================================================
    // Tools submenu
    // =========================================================================
    let tools_menu = SubmenuBuilder::new(app, "Tools")
        .item(&MenuItem::with_id(
            app,
            "generate-report",
            "Generate Report…",
            false,
            Some("CmdOrCtrl+P"),
        )?)
        .item(&MenuItem::with_id(
            app,
            "evidence-collection",
            "Evidence Collection…",
            false,
            None::<&str>,
        )?)
        .item(&MenuItem::with_id(
            app,
            "evidence-collection-list",
            "Evidence Collection List…",
            false,
            None::<&str>,
        )?)
        .separator()
        .item(&MenuItem::with_id(
            app,
            "search-evidence",
            "Search Evidence…",
            false,
            Some("CmdOrCtrl+F"),
        )?)
        .item(&MenuItem::with_id(
            app,
            "hash-all",
            "Hash All Evidence",
            false,
            None::<&str>,
        )?)
        .item(&MenuItem::with_id(
            app,
            "hash-selected",
            "Hash Selected Files",
            false,
            None::<&str>,
        )?)
        .item(&MenuItem::with_id(
            app,
            "hash-active",
            "Hash Active File",
            false,
            Some("CmdOrCtrl+H"),
        )?)
        .separator()
        .item(&MenuItem::with_id(
            app,
            "deduplication",
            "File Deduplication…",
            false,
            None::<&str>,
        )?)
        .item(&MenuItem::with_id(
            app,
            "load-all-info",
            "Load All File Info",
            false,
            None::<&str>,
        )?)
        .item(&MenuItem::with_id(
            app,
            "clean-cache",
            "Clean Preview Cache",
            true,
            None::<&str>,
        )?)
        .separator()
        .item(&MenuItem::with_id(
            app,
            "settings",
            "Settings…",
            true,
            Some("CmdOrCtrl+,"),
        )?)
        .build()?;

    // =========================================================================
    // Help submenu
    // =========================================================================
    let help_menu = {
        #[allow(unused_mut)]
        let mut builder = SubmenuBuilder::new(app, "Help")
            .text("user-guide", "User Guide")
            .separator()
            .text("welcome-screen", "Welcome Screen")
            .text("start-tour", "Start Guided Tour")
            .separator()
            .text("keyboard-shortcuts", "Keyboard Shortcuts…")
            .item(&MenuItem::with_id(
                app,
                "command-palette",
                "Command Palette…",
                true,
                Some("CmdOrCtrl+K"),
            )?)
            .separator()
            .text("check-updates", "Check for Updates…");

        // On non-macOS, add About to Help menu
        #[cfg(not(target_os = "macos"))]
        {
            builder = builder.separator().about(Some(AboutMetadata {
                name: Some("CORE-FFX".into()),
                version: Some(env!("CARGO_PKG_VERSION").into()),
                copyright: Some("© 2024-2026 CORE-FFX Project Contributors".into()),
                license: Some("MIT License".into()),
                ..Default::default()
            }));
        }

        builder.build()?
    };

    // =========================================================================
    // Assemble the full menu bar
    // =========================================================================
    #[cfg(target_os = "macos")]
    let menu_builder = MenuBuilder::new(app).item(&app_menu);
    #[cfg(not(target_os = "macos"))]
    let menu_builder = MenuBuilder::new(app);

    menu_builder
        .item(&file_menu)
        .item(&edit_menu)
        .item(&view_menu)
        .item(&tools_menu)
        .item(&window_menu)
        .item(&help_menu)
        .build()
}

/// Handle menu events from the native menu bar.
///
/// Custom menu items (identified by string IDs) are handled here.
/// Predefined items (Close, Quit, Copy, etc.) are handled automatically by Tauri.
pub fn handle_menu_event<R: Runtime>(app: &AppHandle<R>, event: MenuEvent) {
    let id = event.id();
    info!(menu_id = %id.0, "Menu event received");

    // Custom items: emit to the focused window so the frontend handles them.
    // Predefined items (quit, close, copy, paste, etc.) are handled by Tauri automatically.
    let action: Option<&str> = if id == "new-window" {
        if let Err(e) = create_new_window(app) {
            error!("Failed to create new window: {}", e);
        }
        None
    } else if id == "open-project" {
        Some("open-project")
    } else if id == "open-directory" {
        Some("open-directory")
    } else if id == "save-project" {
        Some("save-project")
    } else if id == "save-project-as" {
        Some("save-project-as")
    } else if id == "toggle-sidebar" {
        Some("toggle-sidebar")
    } else if id == "toggle-right-panel" {
        Some("toggle-right-panel")
    } else if id == "keyboard-shortcuts" {
        Some("keyboard-shortcuts")
    } else if id == "command-palette" {
        Some("command-palette")
    } else if id == "new-project" {
        Some("new-project")
    } else if id == "export" {
        Some("export")
    } else if id == "generate-report" {
        Some("generate-report")
    } else if id == "scan-evidence" {
        Some("scan-evidence")
    } else if id == "toggle-quick-actions" {
        Some("toggle-quick-actions")
    } else if id == "show-evidence" {
        Some("show-evidence")
    } else if id == "show-casedocs" {
        Some("show-casedocs")
    } else if id == "show-processed" {
        Some("show-processed")
    } else if id == "evidence-collection" {
        Some("evidence-collection")
    } else if id == "search-evidence" {
        Some("search-evidence")
    } else if id == "settings" {
        Some("settings")
    } else if id == "close-all-tabs" {
        Some("close-all-tabs")
    } else if id == "close-active-tab" {
        Some("close-active-tab")
    } else if id == "toggle-autosave" {
        Some("toggle-autosave")
    } else if id == "hash-all" {
        Some("hash-all")
    } else if id == "hash-selected" {
        Some("hash-selected")
    } else if id == "hash-active" {
        Some("hash-active")
    } else if id == "evidence-collection-list" {
        Some("evidence-collection-list")
    } else if id == "user-guide" {
        Some("user-guide")
    } else if id == "welcome-screen" {
        Some("welcome-screen")
    } else if id == "start-tour" {
        Some("start-tour")
    } else if id == "show-dashboard" {
        Some("show-dashboard")
    } else if id == "show-activity" {
        Some("show-activity")
    } else if id == "show-bookmarks" {
        Some("show-bookmarks")
    } else if id == "view-info" {
        Some("view-info")
    } else if id == "view-hex" {
        Some("view-hex")
    } else if id == "view-text" {
        Some("view-text")
    } else if id == "cycle-theme" {
        Some("cycle-theme")
    } else if id == "select-all-evidence" {
        Some("select-all-evidence")
    } else if id == "deduplication" {
        Some("deduplication")
    } else if id == "load-all-info" {
        Some("load-all-info")
    } else if id == "clean-cache" {
        Some("clean-cache")
    } else if id == "check-updates" {
        Some("check-updates")
    } else {
        None
    };

    if let Some(action_name) = action {
        emit_to_focused_window(app, action_name);
    }
}

/// Emit an event to the focused webview window.
///
/// Falls back to the first window if none is focused (e.g., when menu is active).
fn emit_to_focused_window<R: Runtime>(app: &AppHandle<R>, action: &str) {
    let windows = app.webview_windows();

    // Try to find the focused window
    let focused = windows.values().find(|w| w.is_focused().unwrap_or(false));

    if let Some(window) = focused.or_else(|| windows.values().next()) {
        let _ = window.emit("menu-action", action);
    }
}

/// Create a new application window.
///
/// Each window gets a unique label derived from a timestamp counter.
/// The window opens the same frontend URL but with independent state.
fn create_new_window<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let label = format!(
        "main-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    );

    info!(label = %label, "Creating new window");

    WebviewWindowBuilder::new(app, &label, WebviewUrl::default())
        .title("CORE-FFX")
        .inner_size(1100.0, 720.0)
        .min_inner_size(960.0, 640.0)
        .build()?;

    Ok(())
}

/// Tauri command: Create a new window from the frontend.
#[tauri::command]
pub async fn new_window(app: AppHandle) -> Result<String, String> {
    let label = format!(
        "main-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    );

    WebviewWindowBuilder::new(&app, &label, WebviewUrl::default())
        .title("CORE-FFX")
        .inner_size(1100.0, 720.0)
        .min_inner_size(960.0, 640.0)
        .build()
        .map_err(|e| e.to_string())?;

    Ok(label)
}

/// Tauri command: Get all open window labels.
#[tauri::command]
pub async fn get_window_labels(app: AppHandle) -> Result<Vec<String>, String> {
    let windows = app.webview_windows();
    let labels: Vec<String> = windows.keys().cloned().collect();
    Ok(labels)
}

/// Menu item IDs that require an active project to be enabled.
const PROJECT_DEPENDENT_IDS: &[&str] = &[
    "save-project",
    "save-project-as",
    "export",
    "scan-evidence",
    "close-all-tabs",
    "close-active-tab",
    "toggle-autosave",
    "show-dashboard",
    "show-evidence",
    "show-casedocs",
    "show-processed",
    "show-activity",
    "show-bookmarks",
    "view-info",
    "view-hex",
    "view-text",
    "generate-report",
    "evidence-collection",
    "evidence-collection-list",
    "search-evidence",
    "hash-all",
    "hash-selected",
    "hash-active",
    "deduplication",
    "load-all-info",
    "select-all-evidence",
];

/// Tauri command: Enable or disable project-dependent menu items.
///
/// Called by the frontend when a project is loaded/created or closed.
#[tauri::command]
pub async fn set_project_menu_state(app: AppHandle, has_project: bool) -> Result<(), String> {
    use tauri::menu::MenuItemKind;

    let menu = app.menu().ok_or("No application menu found")?;

    // Iterate all top-level submenus and search for items by ID
    for submenu_kind in menu.items().map_err(|e| e.to_string())? {
        if let MenuItemKind::Submenu(submenu) = submenu_kind {
            for id in PROJECT_DEPENDENT_IDS {
                if let Some(kind) = submenu.get(*id) {
                    if let Some(item) = kind.as_menuitem() {
                        item.set_enabled(has_project).map_err(|e| e.to_string())?;
                    }
                }
            }
        }
    }

    info!(
        has_project = has_project,
        "Updated project-dependent menu state"
    );
    Ok(())
}
