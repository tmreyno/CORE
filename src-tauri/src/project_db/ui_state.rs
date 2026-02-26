// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Tab state, UI key-value state, and report operations.

use std::collections::HashMap;

use super::database::ProjectDatabase;
use super::types::*;
use rusqlite::{params, Result as SqlResult};

impl ProjectDatabase {
    // ========================================================================
    // Tab Operations (UI State)
    // ========================================================================

    /// Save all tabs (replace all)
    pub fn save_tabs(&self, tabs: &[DbProjectTab]) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM tabs", [])?;
        for tab in tabs {
            conn.execute(
                "INSERT INTO tabs (id, tab_type, file_path, name, subtitle, tab_order, extra)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    tab.id,
                    tab.tab_type,
                    tab.file_path,
                    tab.name,
                    tab.subtitle,
                    tab.tab_order,
                    tab.extra,
                ],
            )?;
        }
        Ok(())
    }

    /// Get all tabs ordered
    pub fn get_tabs(&self) -> SqlResult<Vec<DbProjectTab>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, tab_type, file_path, name, subtitle, tab_order, extra
             FROM tabs ORDER BY tab_order",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(DbProjectTab {
                id: row.get(0)?,
                tab_type: row.get(1)?,
                file_path: row.get(2)?,
                name: row.get(3)?,
                subtitle: row.get(4)?,
                tab_order: row.get(5)?,
                extra: row.get(6)?,
            })
        })?;

        rows.collect()
    }

    // ========================================================================
    // UI State (key-value)
    // ========================================================================

    /// Set a UI state value
    pub fn set_ui_state(&self, key: &str, value: &str) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO ui_state (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    /// Get a UI state value
    pub fn get_ui_state(&self, key: &str) -> SqlResult<Option<String>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare("SELECT value FROM ui_state WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    /// Get all UI state as a map
    pub fn get_all_ui_state(&self) -> SqlResult<HashMap<String, String>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare("SELECT key, value FROM ui_state")?;
        let mut map = HashMap::new();
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let key: String = row.get(0)?;
            let value: String = row.get(1)?;
            map.insert(key, value);
        }
        Ok(map)
    }
}
