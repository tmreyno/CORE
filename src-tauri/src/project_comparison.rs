// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Project comparison and merge functionality.
//!
//! Provides:
//! - Compare two projects (diff analysis)
//! - Merge projects with conflict detection
//! - Synchronize bookmarks and notes across projects
//! - Export comparison reports

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use crate::project::{FFXProject, ProjectBookmark, ProjectNote};

/// Project comparison result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectComparison {
    /// Project A name
    pub project_a_name: String,
    /// Project B name
    pub project_b_name: String,
    /// Comparison timestamp
    pub compared_at: String,
    /// Summary statistics
    pub summary: ComparisonSummary,
    /// Bookmark differences
    pub bookmark_diff: BookmarkDiff,
    /// Note differences
    pub note_diff: NoteDiff,
    /// Evidence file differences
    pub evidence_diff: EvidenceDiff,
    /// Activity differences
    pub activity_diff: ActivityDiff,
    /// Merge conflicts
    pub conflicts: Vec<MergeConflict>,
    /// Merge recommendations
    pub recommendations: Vec<String>,
}

/// Comparison summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonSummary {
    /// Items only in project A
    pub unique_to_a: usize,
    /// Items only in project B
    pub unique_to_b: usize,
    /// Items in both (identical)
    pub common: usize,
    /// Items in both (modified)
    pub modified: usize,
    /// Total differences
    pub total_differences: usize,
    /// Similarity percentage (0-100)
    pub similarity_percent: f64,
}

/// Bookmark differences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkDiff {
    /// Bookmarks only in A
    pub only_in_a: Vec<ProjectBookmark>,
    /// Bookmarks only in B
    pub only_in_b: Vec<ProjectBookmark>,
    /// Bookmarks in both (identical)
    pub common: Vec<ProjectBookmark>,
    /// Bookmarks with different content
    pub modified: Vec<BookmarkPair>,
}

/// Note differences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteDiff {
    /// Notes only in A
    pub only_in_a: Vec<ProjectNote>,
    /// Notes only in B
    pub only_in_b: Vec<ProjectNote>,
    /// Notes in both (identical)
    pub common: Vec<ProjectNote>,
    /// Notes with different content
    pub modified: Vec<NotePair>,
}

/// Evidence file differences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceDiff {
    /// Files only in A
    pub only_in_a: Vec<String>,
    /// Files only in B
    pub only_in_b: Vec<String>,
    /// Files in both
    pub common: Vec<String>,
}

/// Activity differences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDiff {
    /// Total activities in A
    pub count_a: usize,
    /// Total activities in B
    pub count_b: usize,
    /// Activity types only in A
    pub types_only_in_a: Vec<String>,
    /// Activity types only in B
    pub types_only_in_b: Vec<String>,
    /// Common activity types
    pub common_types: Vec<String>,
}

/// Bookmark pair (modified)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkPair {
    pub from_a: ProjectBookmark,
    pub from_b: ProjectBookmark,
    pub differences: Vec<String>,
}

/// Note pair (modified)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotePair {
    pub from_a: ProjectNote,
    pub from_b: ProjectNote,
    pub differences: Vec<String>,
}

/// Merge conflict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeConflict {
    /// Conflict type
    pub conflict_type: ConflictType,
    /// Conflict description
    pub description: String,
    /// Item A (if applicable)
    pub item_a: Option<serde_json::Value>,
    /// Item B (if applicable)
    pub item_b: Option<serde_json::Value>,
    /// Suggested resolution
    pub resolution: String,
}

/// Conflict type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConflictType {
    /// Bookmark with same name but different content
    BookmarkConflict,
    /// Note with same title but different content
    NoteConflict,
    /// Evidence file with same name but different hash
    EvidenceConflict,
    /// Metadata conflict
    MetadataConflict,
}

/// Merge strategy
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MergeStrategy {
    /// Prefer items from project A
    PreferA,
    /// Prefer items from project B
    PreferB,
    /// Keep both (with suffix)
    KeepBoth,
    /// Skip conflicting items
    Skip,
    /// Manual resolution required
    Manual,
}

/// Merge result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeResult {
    /// Merged project
    pub merged_project: FFXProject,
    /// Conflicts encountered
    pub conflicts: Vec<MergeConflict>,
    /// Items merged
    pub items_merged: usize,
    /// Items skipped
    pub items_skipped: usize,
    /// Merge summary
    pub summary: String,
}

/// Compare two projects
pub fn compare_projects(project_a: &FFXProject, project_b: &FFXProject) -> ProjectComparison {
    let bookmark_diff = compare_bookmarks(&project_a.bookmarks, &project_b.bookmarks);
    let note_diff = compare_notes(&project_a.notes, &project_b.notes);
    let evidence_diff = compare_evidence(project_a, project_b);
    let activity_diff = compare_activity(project_a, project_b);
    
    // Calculate summary
    let total_a = project_a.bookmarks.len() + project_a.notes.len() + project_a.open_directories.len();
    let total_b = project_b.bookmarks.len() + project_b.notes.len() + project_b.open_directories.len();
    let unique_to_a = bookmark_diff.only_in_a.len() + note_diff.only_in_a.len() + evidence_diff.only_in_a.len();
    let unique_to_b = bookmark_diff.only_in_b.len() + note_diff.only_in_b.len() + evidence_diff.only_in_b.len();
    let common = bookmark_diff.common.len() + note_diff.common.len() + evidence_diff.common.len();
    let modified = bookmark_diff.modified.len() + note_diff.modified.len();
    let total_differences = unique_to_a + unique_to_b + modified;
    
    let similarity = if total_a + total_b > 0 {
        ((common * 2) as f64 / (total_a + total_b) as f64) * 100.0
    } else {
        100.0
    };
    
    // Detect conflicts
    let conflicts = detect_conflicts(&bookmark_diff, &note_diff);
    
    // Generate recommendations
    let recommendations = generate_merge_recommendations(&bookmark_diff, &note_diff, &evidence_diff);
    
    ProjectComparison {
        project_a_name: project_a.name.clone(),
        project_b_name: project_b.name.clone(),
        compared_at: chrono::Utc::now().to_rfc3339(),
        summary: ComparisonSummary {
            unique_to_a,
            unique_to_b,
            common,
            modified,
            total_differences,
            similarity_percent: similarity,
        },
        bookmark_diff,
        note_diff,
        evidence_diff,
        activity_diff,
        conflicts,
        recommendations,
    }
}

/// Compare bookmarks
fn compare_bookmarks(bookmarks_a: &[ProjectBookmark], bookmarks_b: &[ProjectBookmark]) -> BookmarkDiff {
    let mut only_in_a = Vec::new();
    let mut only_in_b = Vec::new();
    let mut common = Vec::new();
    let mut modified = Vec::new();
    
    let map_b: HashMap<_, _> = bookmarks_b.iter().map(|b| (&b.name, b)).collect();
    let mut seen_b = HashSet::new();
    
    for bookmark_a in bookmarks_a {
        if let Some(bookmark_b) = map_b.get(&bookmark_a.name) {
            seen_b.insert(&bookmark_a.name);
            
            if bookmarks_equal(bookmark_a, bookmark_b) {
                common.push(bookmark_a.clone());
            } else {
                let differences = find_bookmark_differences(bookmark_a, bookmark_b);
                modified.push(BookmarkPair {
                    from_a: bookmark_a.clone(),
                    from_b: (*bookmark_b).clone(),
                    differences,
                });
            }
        } else {
            only_in_a.push(bookmark_a.clone());
        }
    }
    
    for bookmark_b in bookmarks_b {
        if !seen_b.contains(&bookmark_b.name) {
            only_in_b.push(bookmark_b.clone());
        }
    }
    
    BookmarkDiff {
        only_in_a,
        only_in_b,
        common,
        modified,
    }
}

/// Compare notes
fn compare_notes(notes_a: &[ProjectNote], notes_b: &[ProjectNote]) -> NoteDiff {
    let mut only_in_a = Vec::new();
    let mut only_in_b = Vec::new();
    let mut common = Vec::new();
    let mut modified = Vec::new();
    
    let map_b: HashMap<_, _> = notes_b.iter().map(|n| (&n.title, n)).collect();
    let mut seen_b = HashSet::new();
    
    for note_a in notes_a {
        if let Some(note_b) = map_b.get(&note_a.title) {
            seen_b.insert(&note_a.title);
            
            if notes_equal(note_a, note_b) {
                common.push(note_a.clone());
            } else {
                let differences = find_note_differences(note_a, note_b);
                modified.push(NotePair {
                    from_a: note_a.clone(),
                    from_b: (*note_b).clone(),
                    differences,
                });
            }
        } else {
            only_in_a.push(note_a.clone());
        }
    }
    
    for note_b in notes_b {
        if !seen_b.contains(&note_b.title) {
            only_in_b.push(note_b.clone());
        }
    }
    
    NoteDiff {
        only_in_a,
        only_in_b,
        common,
        modified,
    }
}

/// Compare evidence files
fn compare_evidence(project_a: &FFXProject, project_b: &FFXProject) -> EvidenceDiff {
    let files_a: HashSet<_> = project_a.open_directories.iter().map(|d| d.path.clone()).collect();
    let files_b: HashSet<_> = project_b.open_directories.iter().map(|d| d.path.clone()).collect();
    
    let only_in_a: Vec<_> = files_a.difference(&files_b).cloned().collect();
    let only_in_b: Vec<_> = files_b.difference(&files_a).cloned().collect();
    let common: Vec<_> = files_a.intersection(&files_b).cloned().collect();
    
    EvidenceDiff {
        only_in_a,
        only_in_b,
        common,
    }
}

/// Compare activity logs
fn compare_activity(project_a: &FFXProject, project_b: &FFXProject) -> ActivityDiff {
    let types_a: HashSet<_> = project_a.activity_log.iter().map(|a| a.action.clone()).collect();
    let types_b: HashSet<_> = project_b.activity_log.iter().map(|a| a.action.clone()).collect();
    
    let types_only_in_a: Vec<_> = types_a.difference(&types_b).cloned().collect();
    let types_only_in_b: Vec<_> = types_b.difference(&types_a).cloned().collect();
    let common_types: Vec<_> = types_a.intersection(&types_b).cloned().collect();
    
    ActivityDiff {
        count_a: project_a.activity_log.len(),
        count_b: project_b.activity_log.len(),
        types_only_in_a,
        types_only_in_b,
        common_types,
    }
}

/// Check if bookmarks are equal
fn bookmarks_equal(a: &ProjectBookmark, b: &ProjectBookmark) -> bool {
    a.name == b.name &&
    a.target_path == b.target_path &&
    a.notes == b.notes &&
    a.color == b.color &&
    a.tags == b.tags
}

/// Check if notes are equal
fn notes_equal(a: &ProjectNote, b: &ProjectNote) -> bool {
    a.title == b.title &&
    a.content == b.content &&
    a.priority == b.priority &&
    a.tags == b.tags
}

/// Find differences between bookmarks
fn find_bookmark_differences(a: &ProjectBookmark, b: &ProjectBookmark) -> Vec<String> {
    let mut diffs = Vec::new();
    
    if a.target_path != b.target_path {
        diffs.push("target_path".to_string());
    }
    if a.notes != b.notes {
        diffs.push("notes".to_string());
    }
    if a.color != b.color {
        diffs.push("color".to_string());
    }
    if a.tags != b.tags {
        diffs.push("tags".to_string());
    }
    
    diffs
}

/// Find differences between notes
fn find_note_differences(a: &ProjectNote, b: &ProjectNote) -> Vec<String> {
    let mut diffs = Vec::new();
    
    if a.content != b.content {
        diffs.push("content".to_string());
    }
    if a.priority != b.priority {
        diffs.push("priority".to_string());
    }
    if a.tags != b.tags {
        diffs.push("tags".to_string());
    }
    
    diffs
}

/// Detect merge conflicts
fn detect_conflicts(bookmark_diff: &BookmarkDiff, note_diff: &NoteDiff) -> Vec<MergeConflict> {
    let mut conflicts = Vec::new();
    
    // Bookmark conflicts
    for pair in &bookmark_diff.modified {
        conflicts.push(MergeConflict {
            conflict_type: ConflictType::BookmarkConflict,
            description: format!("Bookmark '{}' has different content", pair.from_a.name),
            item_a: Some(serde_json::to_value(&pair.from_a).unwrap()),
            item_b: Some(serde_json::to_value(&pair.from_b).unwrap()),
            resolution: "Choose which version to keep or merge manually".to_string(),
        });
    }
    
    // Note conflicts
    for pair in &note_diff.modified {
        conflicts.push(MergeConflict {
            conflict_type: ConflictType::NoteConflict,
            description: format!("Note '{}' has different content", pair.from_a.title),
            item_a: Some(serde_json::to_value(&pair.from_a).unwrap()),
            item_b: Some(serde_json::to_value(&pair.from_b).unwrap()),
            resolution: "Choose which version to keep or merge content".to_string(),
        });
    }
    
    conflicts
}

/// Generate merge recommendations
fn generate_merge_recommendations(
    bookmark_diff: &BookmarkDiff,
    note_diff: &NoteDiff,
    evidence_diff: &EvidenceDiff,
) -> Vec<String> {
    let mut recommendations = Vec::new();
    
    if !bookmark_diff.only_in_b.is_empty() {
        recommendations.push(format!(
            "Consider importing {} unique bookmarks from Project B",
            bookmark_diff.only_in_b.len()
        ));
    }
    
    if !note_diff.only_in_b.is_empty() {
        recommendations.push(format!(
            "Consider importing {} unique notes from Project B",
            note_diff.only_in_b.len()
        ));
    }
    
    if !bookmark_diff.modified.is_empty() {
        recommendations.push(format!(
            "Review {} modified bookmarks and choose preferred version",
            bookmark_diff.modified.len()
        ));
    }
    
    if !note_diff.modified.is_empty() {
        recommendations.push(format!(
            "Review {} modified notes and merge content if needed",
            note_diff.modified.len()
        ));
    }
    
    if !evidence_diff.only_in_b.is_empty() {
        recommendations.push(format!(
            "Project B has {} unique evidence files",
            evidence_diff.only_in_b.len()
        ));
    }
    
    recommendations
}

/// Merge two projects
pub fn merge_projects(
    project_a: &FFXProject,
    project_b: &FFXProject,
    strategy: MergeStrategy,
) -> Result<MergeResult, String> {
    let mut merged = project_a.clone();
    let mut conflicts = Vec::new();
    let mut items_merged = 0;
    let mut items_skipped = 0;
    
    // Merge bookmarks - collect decisions first, then apply
    let mut bookmarks_to_add = Vec::new();
    let mut bookmarks_to_update: Vec<(usize, ProjectBookmark)> = Vec::new();
    
    for bookmark_b in &project_b.bookmarks {
        if let Some(pos) = merged.bookmarks.iter().position(|b| b.name == bookmark_b.name) {
            let bookmark_a = &merged.bookmarks[pos];
            if !bookmarks_equal(bookmark_a, bookmark_b) {
                match strategy {
                    MergeStrategy::PreferA => {
                        items_skipped += 1;
                    }
                    MergeStrategy::PreferB => {
                        bookmarks_to_update.push((pos, bookmark_b.clone()));
                        items_merged += 1;
                    }
                    MergeStrategy::KeepBoth => {
                        let mut new_bookmark = bookmark_b.clone();
                        new_bookmark.name = format!("{} (from B)", bookmark_b.name);
                        bookmarks_to_add.push(new_bookmark);
                        items_merged += 1;
                    }
                    MergeStrategy::Skip => {
                        items_skipped += 1;
                    }
                    MergeStrategy::Manual => {
                        conflicts.push(MergeConflict {
                            conflict_type: ConflictType::BookmarkConflict,
                            description: format!("Bookmark '{}' exists in both projects", bookmark_b.name),
                            item_a: Some(serde_json::to_value(bookmark_a).unwrap()),
                            item_b: Some(serde_json::to_value(bookmark_b).unwrap()),
                            resolution: "Manual resolution required".to_string(),
                        });
                        items_skipped += 1;
                    }
                }
            }
        } else {
            bookmarks_to_add.push(bookmark_b.clone());
            items_merged += 1;
        }
    }
    
    // Apply bookmark changes
    for (pos, bookmark) in bookmarks_to_update {
        merged.bookmarks[pos] = bookmark;
    }
    merged.bookmarks.extend(bookmarks_to_add);
    
    // Merge notes - collect decisions first, then apply
    let mut notes_to_add = Vec::new();
    let mut notes_to_update: Vec<(usize, ProjectNote)> = Vec::new();
    
    for note_b in &project_b.notes {
        if let Some(pos) = merged.notes.iter().position(|n| n.title == note_b.title) {
            let note_a = &merged.notes[pos];
            if !notes_equal(note_a, note_b) {
                match strategy {
                    MergeStrategy::PreferA => {
                        items_skipped += 1;
                    }
                    MergeStrategy::PreferB => {
                        notes_to_update.push((pos, note_b.clone()));
                        items_merged += 1;
                    }
                    MergeStrategy::KeepBoth => {
                        let mut new_note = note_b.clone();
                        new_note.title = format!("{} (from B)", note_b.title);
                        notes_to_add.push(new_note);
                        items_merged += 1;
                    }
                    MergeStrategy::Skip => {
                        items_skipped += 1;
                    }
                    MergeStrategy::Manual => {
                        conflicts.push(MergeConflict {
                            conflict_type: ConflictType::NoteConflict,
                            description: format!("Note '{}' exists in both projects", note_b.title),
                            item_a: Some(serde_json::to_value(note_a).unwrap()),
                            item_b: Some(serde_json::to_value(note_b).unwrap()),
                            resolution: "Manual resolution required".to_string(),
                        });
                        items_skipped += 1;
                    }
                }
            }
        } else {
            notes_to_add.push(note_b.clone());
            items_merged += 1;
        }
    }
    
    // Apply note changes
    for (pos, note) in notes_to_update {
        merged.notes[pos] = note;
    }
    merged.notes.extend(notes_to_add);
    
    let summary = format!(
        "Merged {} items, skipped {} items, {} conflicts",
        items_merged, items_skipped, conflicts.len()
    );
    
    Ok(MergeResult {
        merged_project: merged,
        conflicts,
        items_merged,
        items_skipped,
        summary,
    })
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_empty_projects() {
        let project_a = FFXProject::new("/tmp/test_project_a");
        let project_b = FFXProject::new("/tmp/test_project_b");
        
        let comparison = compare_projects(&project_a, &project_b);
        assert_eq!(comparison.summary.similarity_percent, 100.0);
        assert_eq!(comparison.summary.total_differences, 0);
    }

    #[test]
    fn test_merge_strategy_prefer_a() {
        let mut project_a = FFXProject::new("/tmp/test_project_a");
        let mut project_b = FFXProject::new("/tmp/test_project_b");
        
        project_a.bookmarks.push(ProjectBookmark {
            id: "1".to_string(),
            target_type: "file".to_string(),
            target_path: "path_a".to_string(),
            name: "Test".to_string(),
            created_by: "user".to_string(),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            color: None,
            tags: vec![],
            notes: Some("From A".to_string()),
            context: None,
        });
        
        project_b.bookmarks.push(ProjectBookmark {
            id: "2".to_string(),
            target_type: "file".to_string(),
            target_path: "path_b".to_string(),
            name: "Test".to_string(),
            created_by: "user".to_string(),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            color: None,
            tags: vec![],
            notes: Some("From B".to_string()),
            context: None,
        });
        
        let result = merge_projects(&project_a, &project_b, MergeStrategy::PreferA).unwrap();
        assert_eq!(result.merged_project.bookmarks.len(), 1);
        assert_eq!(result.merged_project.bookmarks[0].notes, Some("From A".to_string()));
    }
}
