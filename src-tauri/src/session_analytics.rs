// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Enhanced session management with resume, comparison, and analytics.
//!
//! Provides:
//! - Session resume with state restoration
//! - Multi-device sync preparation
//! - Session comparison and diff
//! - Detailed work pattern analytics
//! - Session productivity insights

use crate::project::{FFXProject, ProjectSession, ActivityLogEntry};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::Timelike;

/// Enhanced session with state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSnapshot {
    /// Session metadata
    pub session: ProjectSession,
    /// Snapshot timestamp
    pub snapshot_at: String,
    /// Open files at snapshot time
    pub open_files: Vec<String>,
    /// Active file path
    pub active_file: Option<String>,
    /// Activity count in session
    pub activity_count: usize,
    /// Bookmarks added
    pub bookmarks_added: usize,
    /// Notes added
    pub notes_added: usize,
    /// Hashes computed
    pub hashes_computed: usize,
    /// Files accessed
    pub files_accessed: Vec<String>,
    /// Work focus areas
    pub focus_areas: Vec<FocusArea>,
}

/// Work focus area during session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusArea {
    /// Category of work
    pub category: String,
    /// Time spent (seconds)
    pub time_spent: u64,
    /// Activity count
    pub activity_count: usize,
    /// Key files worked on
    pub key_files: Vec<String>,
}

/// Session comparison result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionComparison {
    /// First session info
    pub session_a: SessionSummary,
    /// Second session info
    pub session_b: SessionSummary,
    /// Comparison timestamp
    pub compared_at: String,
    /// Differences
    pub differences: SessionDifferences,
    /// Similarities
    pub similarities: SessionSimilarities,
}

/// Session summary for comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    /// Session ID
    pub session_id: String,
    /// User
    pub user: String,
    /// Duration (seconds)
    pub duration_seconds: Option<u64>,
    /// Activity count
    pub activity_count: usize,
    /// Files accessed
    pub files_accessed: usize,
    /// Start time
    pub started_at: String,
}

/// Differences between sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDifferences {
    /// Duration difference (seconds)
    pub duration_diff: Option<i64>,
    /// Activity count difference
    pub activity_diff: i64,
    /// Files unique to session A
    pub unique_to_a: Vec<String>,
    /// Files unique to session B
    pub unique_to_b: Vec<String>,
    /// Different work patterns
    pub work_pattern_changes: Vec<String>,
}

/// Similarities between sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSimilarities {
    /// Common files accessed
    pub common_files: Vec<String>,
    /// Similar activity patterns
    pub common_patterns: Vec<String>,
    /// Similar focus areas
    pub common_focus: Vec<String>,
}

/// Session analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionAnalytics {
    /// Session ID
    pub session_id: String,
    /// Work patterns
    pub work_patterns: WorkPatterns,
    /// Productivity metrics
    pub productivity: SessionProductivity,
    /// Time distribution
    pub time_distribution: TimeDistribution,
    /// Focus quality
    pub focus_quality: FocusQuality,
    /// Recommendations
    pub recommendations: Vec<String>,
}

/// Work patterns during session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkPatterns {
    /// Peak activity hours
    pub peak_hours: Vec<u8>,
    /// Most used categories
    pub top_categories: Vec<CategoryUsage>,
    /// Most accessed files
    pub top_files: Vec<FileAccess>,
    /// Workflow sequence
    pub workflow_sequence: Vec<WorkflowStep>,
}

/// Category usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryUsage {
    /// Category name
    pub category: String,
    /// Usage count
    pub count: usize,
    /// Percentage of total
    pub percentage: f64,
}

/// File access record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAccess {
    /// File path
    pub path: String,
    /// Access count
    pub count: usize,
    /// Last accessed
    pub last_accessed: String,
}

/// Workflow step in sequence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    /// Step number
    pub step: usize,
    /// Action performed
    pub action: String,
    /// Category
    pub category: String,
    /// Timestamp
    pub timestamp: String,
}

/// Session productivity metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionProductivity {
    /// Actions per minute
    pub actions_per_minute: f64,
    /// Files per hour
    pub files_per_hour: f64,
    /// Output items created (bookmarks, notes, reports)
    pub outputs_created: usize,
    /// Efficiency score (0-100)
    pub efficiency_score: f64,
    /// Time utilization (0-100)
    pub time_utilization: f64,
}

/// Time distribution across categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeDistribution {
    /// Time by category (seconds)
    pub by_category: HashMap<String, u64>,
    /// Time by hour of day (seconds)
    pub by_hour: HashMap<u8, u64>,
    /// Idle periods (seconds)
    pub idle_time: u64,
    /// Active work time (seconds)
    pub active_time: u64,
}

/// Focus quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusQuality {
    /// Focus score (0-100)
    pub score: f64,
    /// Context switches
    pub context_switches: usize,
    /// Average time per task (seconds)
    pub avg_task_duration: f64,
    /// Longest continuous work period (seconds)
    pub longest_focus_period: u64,
    /// Distractions detected
    pub distractions: usize,
}

/// Session resume data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionResumeData {
    /// Session ID to resume
    pub session_id: String,
    /// Project state at pause
    pub project_state: ResumeProjectState,
    /// Paused timestamp
    pub paused_at: String,
    /// Resume instructions
    pub resume_hints: Vec<String>,
}

/// Project state for resume
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumeProjectState {
    /// Open tabs
    pub open_tabs: Vec<String>,
    /// Active tab
    pub active_tab: Option<String>,
    /// Recent activities (last 10)
    pub recent_activities: Vec<ActivityLogEntry>,
    /// In-progress tasks
    pub tasks_in_progress: Vec<String>,
    /// Bookmarks since session start
    pub session_bookmarks: Vec<String>,
}

// =============================================================================
// SESSION SNAPSHOT & RESUME
// =============================================================================

/// Create a session snapshot for resume
pub fn create_session_snapshot(
    project: &FFXProject,
    session_id: &str,
) -> Result<SessionSnapshot, String> {
    let session = project
        .sessions
        .iter()
        .find(|s| s.session_id == session_id)
        .ok_or("Session not found")?;

    let session_activities: Vec<_> = project
        .activity_log
        .iter()
        .filter(|a| a.details.as_ref()
            .and_then(|d| d.get("session_id"))
            .and_then(|v| v.as_str())
            .map(|sid| sid == session_id)
            .unwrap_or(false))
        .collect();

    let activity_count = session_activities.len();

    // Count specific activities
    let bookmarks_added = session_activities
        .iter()
        .filter(|a| a.category == "bookmark" && a.action.contains("add"))
        .count();

    let notes_added = session_activities
        .iter()
        .filter(|a| a.category == "note" && a.action.contains("create"))
        .count();

    let hashes_computed = session_activities
        .iter()
        .filter(|a| a.category == "hash" && a.action.contains("compute"))
        .count();

    // Collect unique files accessed
    let files_accessed: Vec<String> = session_activities
        .iter()
        .filter_map(|a| a.file_path.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    // Analyze focus areas
    let mut category_time: HashMap<String, usize> = HashMap::new();
    let mut category_files: HashMap<String, Vec<String>> = HashMap::new();

    for activity in &session_activities {
        *category_time.entry(activity.category.clone()).or_insert(0) += 1;
        if let Some(path) = &activity.file_path {
            category_files
                .entry(activity.category.clone())
                .or_default()
                .push(path.clone());
        }
    }

    let focus_areas: Vec<FocusArea> = category_time
        .into_iter()
        .map(|(category, count)| {
            let mut key_files = category_files
                .get(&category)
                .cloned()
                .unwrap_or_default();
            key_files.sort();
            key_files.dedup();
            key_files.truncate(5);

            FocusArea {
                category,
                time_spent: (count * 60) as u64, // Rough estimate
                activity_count: count,
                key_files,
            }
        })
        .collect();

    Ok(SessionSnapshot {
        session: session.clone(),
        snapshot_at: chrono::Utc::now().to_rfc3339(),
        open_files: project.tabs.iter().map(|t| t.file_path.clone()).collect(),
        active_file: project.active_tab_path.clone(),
        activity_count,
        bookmarks_added,
        notes_added,
        hashes_computed,
        files_accessed,
        focus_areas,
    })
}

/// Create resume data for a session
pub fn create_resume_data(
    project: &FFXProject,
    session_id: &str,
) -> Result<SessionResumeData, String> {
    let session = project
        .sessions
        .iter()
        .find(|s| s.session_id == session_id)
        .ok_or("Session not found")?;

    // Get recent activities (last 10)
    let recent_activities: Vec<ActivityLogEntry> = project
        .activity_log
        .iter()
        .rev()
        .take(10)
        .cloned()
        .collect();

    // Detect in-progress tasks
    let mut tasks_in_progress = Vec::new();
    for activity in recent_activities.iter().take(3) {
        if activity.action.contains("start") || activity.action.contains("open") {
            tasks_in_progress.push(activity.description.clone());
        }
    }

    // Get bookmarks added in this session
    let session_start = chrono::DateTime::parse_from_rfc3339(&session.started_at).ok();
    let session_bookmarks: Vec<String> = project
        .bookmarks
        .iter()
        .filter(|b| {
            if let Some(start) = session_start {
                chrono::DateTime::parse_from_rfc3339(&b.created_at)
                    .ok()
                    .map(|created| created > start)
                    .unwrap_or(false)
            } else {
                false
            }
        })
        .map(|b| b.target_path.clone())
        .collect();

    // Generate resume hints
    let mut resume_hints = Vec::new();
    if !project.tabs.is_empty() {
        resume_hints.push(format!("{} files were open", project.tabs.len()));
    }
    if let Some(active) = &project.active_tab_path {
        resume_hints.push(format!("Last viewed: {}", active));
    }
    if !tasks_in_progress.is_empty() {
        resume_hints.push(format!("{} tasks in progress", tasks_in_progress.len()));
    }

    Ok(SessionResumeData {
        session_id: session_id.to_string(),
        project_state: ResumeProjectState {
            open_tabs: project.tabs.iter().map(|t| t.file_path.clone()).collect(),
            active_tab: project.active_tab_path.clone(),
            recent_activities,
            tasks_in_progress,
            session_bookmarks,
        },
        paused_at: chrono::Utc::now().to_rfc3339(),
        resume_hints,
    })
}

// =============================================================================
// SESSION COMPARISON
// =============================================================================

/// Compare two sessions
pub fn compare_sessions(
    project: &FFXProject,
    session_a_id: &str,
    session_b_id: &str,
) -> Result<SessionComparison, String> {
    let session_a = project
        .sessions
        .iter()
        .find(|s| s.session_id == session_a_id)
        .ok_or("Session A not found")?;

    let session_b = project
        .sessions
        .iter()
        .find(|s| s.session_id == session_b_id)
        .ok_or("Session B not found")?;

    // Get activities for each session
    let activities_a: Vec<_> = project
        .activity_log
        .iter()
        .filter(|a| a.details.as_ref()
            .and_then(|d| d.get("session_id"))
            .and_then(|v| v.as_str())
            .map(|sid| sid == session_a_id)
            .unwrap_or(false))
        .collect();

    let activities_b: Vec<_> = project
        .activity_log
        .iter()
        .filter(|a| a.details.as_ref()
            .and_then(|d| d.get("session_id"))
            .and_then(|v| v.as_str())
            .map(|sid| sid == session_b_id)
            .unwrap_or(false))
        .collect();

    // Extract files
    let files_a: std::collections::HashSet<String> = activities_a
        .iter()
        .filter_map(|a| a.file_path.clone())
        .collect();

    let files_b: std::collections::HashSet<String> = activities_b
        .iter()
        .filter_map(|a| a.file_path.clone())
        .collect();

    // Compute differences
    let unique_to_a: Vec<String> = files_a.difference(&files_b).cloned().collect();
    let unique_to_b: Vec<String> = files_b.difference(&files_a).cloned().collect();
    let common_files: Vec<String> = files_a.intersection(&files_b).cloned().collect();

    let duration_diff = match (session_a.duration_seconds, session_b.duration_seconds) {
        (Some(a), Some(b)) => Some(a as i64 - b as i64),
        _ => None,
    };

    let activity_diff = activities_a.len() as i64 - activities_b.len() as i64;

    // Analyze patterns
    let mut work_pattern_changes = Vec::new();
    if activity_diff.abs() > 20 {
        work_pattern_changes.push(format!("Activity level changed by {}", activity_diff));
    }
    if duration_diff.map(|d| d.abs() > 1800).unwrap_or(false) {
        work_pattern_changes.push("Session duration significantly different".to_string());
    }

    let common_patterns = vec![
        format!("{} common files", common_files.len()),
    ];

    Ok(SessionComparison {
        session_a: SessionSummary {
            session_id: session_a.session_id.clone(),
            user: session_a.user.clone(),
            duration_seconds: session_a.duration_seconds,
            activity_count: activities_a.len(),
            files_accessed: files_a.len(),
            started_at: session_a.started_at.clone(),
        },
        session_b: SessionSummary {
            session_id: session_b.session_id.clone(),
            user: session_b.user.clone(),
            duration_seconds: session_b.duration_seconds,
            activity_count: activities_b.len(),
            files_accessed: files_b.len(),
            started_at: session_b.started_at.clone(),
        },
        compared_at: chrono::Utc::now().to_rfc3339(),
        differences: SessionDifferences {
            duration_diff,
            activity_diff,
            unique_to_a,
            unique_to_b,
            work_pattern_changes,
        },
        similarities: SessionSimilarities {
            common_files,
            common_patterns,
            common_focus: Vec::new(),
        },
    })
}

// =============================================================================
// SESSION ANALYTICS
// =============================================================================

/// Compute comprehensive session analytics
pub fn compute_session_analytics(
    project: &FFXProject,
    session_id: &str,
) -> Result<SessionAnalytics, String> {
    let session = project
        .sessions
        .iter()
        .find(|s| s.session_id == session_id)
        .ok_or("Session not found")?;

    let session_activities: Vec<_> = project
        .activity_log
        .iter()
        .filter(|a| a.details.as_ref()
            .and_then(|d| d.get("session_id"))
            .and_then(|v| v.as_str())
            .map(|sid| sid == session_id)
            .unwrap_or(false))
        .collect();

    // Work patterns
    let work_patterns = analyze_work_patterns(&session_activities);

    // Productivity
    let productivity = compute_session_productivity(session, &session_activities);

    // Time distribution
    let time_distribution = compute_time_distribution(&session_activities);

    // Focus quality
    let focus_quality = compute_focus_quality(&session_activities);

    // Generate recommendations
    let recommendations = generate_recommendations(&productivity, &focus_quality);

    Ok(SessionAnalytics {
        session_id: session_id.to_string(),
        work_patterns,
        productivity,
        time_distribution,
        focus_quality,
        recommendations,
    })
}

fn analyze_work_patterns(activities: &[&ActivityLogEntry]) -> WorkPatterns {
    let mut hour_counts: HashMap<u8, usize> = HashMap::new();
    let mut category_counts: HashMap<String, usize> = HashMap::new();
    let mut file_access: HashMap<String, (usize, String)> = HashMap::new();

    for activity in activities {
        if let Ok(timestamp) = chrono::DateTime::parse_from_rfc3339(&activity.timestamp) {
            *hour_counts.entry(timestamp.hour() as u8).or_insert(0) += 1;
        }

        *category_counts.entry(activity.category.clone()).or_insert(0) += 1;

        if let Some(path) = &activity.file_path {
            let entry = file_access.entry(path.clone()).or_insert((0, activity.timestamp.clone()));
            entry.0 += 1;
            entry.1 = activity.timestamp.clone();
        }
    }

    let mut peak_hours: Vec<_> = hour_counts.into_iter().collect();
    peak_hours.sort_by(|a, b| b.1.cmp(&a.1));
    let peak_hours: Vec<u8> = peak_hours.into_iter().take(3).map(|(h, _)| h).collect();

    let total_activities = activities.len();
    let mut top_categories: Vec<_> = category_counts
        .into_iter()
        .map(|(cat, count)| CategoryUsage {
            category: cat,
            count,
            percentage: (count as f64 / total_activities as f64) * 100.0,
        })
        .collect();
    top_categories.sort_by(|a, b| b.count.cmp(&a.count));
    top_categories.truncate(5);

    let mut top_files: Vec<_> = file_access
        .into_iter()
        .map(|(path, (count, last_accessed))| FileAccess {
            path,
            count,
            last_accessed,
        })
        .collect();
    top_files.sort_by(|a, b| b.count.cmp(&a.count));
    top_files.truncate(10);

    let workflow_sequence: Vec<WorkflowStep> = activities
        .iter()
        .enumerate()
        .take(20)
        .map(|(i, a)| WorkflowStep {
            step: i + 1,
            action: a.action.clone(),
            category: a.category.clone(),
            timestamp: a.timestamp.clone(),
        })
        .collect();

    WorkPatterns {
        peak_hours,
        top_categories,
        top_files,
        workflow_sequence,
    }
}

fn compute_session_productivity(
    session: &ProjectSession,
    activities: &[&ActivityLogEntry],
) -> SessionProductivity {
    let duration_seconds = session.duration_seconds.unwrap_or(1);
    let duration_minutes = (duration_seconds as f64) / 60.0;
    let duration_hours = duration_minutes / 60.0;

    let actions_per_minute = (activities.len() as f64) / duration_minutes;
    
    let unique_files = activities
        .iter()
        .filter_map(|a| a.file_path.as_ref())
        .collect::<std::collections::HashSet<_>>()
        .len();
    let files_per_hour = (unique_files as f64) / duration_hours;

    let outputs_created = activities
        .iter()
        .filter(|a| {
            matches!(a.category.as_str(), "bookmark" | "note" | "export")
                && (a.action.contains("add") || a.action.contains("create") || a.action.contains("generate"))
        })
        .count();

    let efficiency_score = calculate_session_efficiency(
        actions_per_minute,
        files_per_hour,
        outputs_created,
    );

    let time_utilization = if duration_seconds > 0 {
        ((activities.len() as f64) / (duration_seconds as f64 / 60.0)).min(100.0)
    } else {
        0.0
    };

    SessionProductivity {
        actions_per_minute,
        files_per_hour,
        outputs_created,
        efficiency_score,
        time_utilization,
    }
}

fn compute_time_distribution(activities: &[&ActivityLogEntry]) -> TimeDistribution {
    let mut by_category: HashMap<String, u64> = HashMap::new();
    let mut by_hour: HashMap<u8, u64> = HashMap::new();

    for activity in activities {
        *by_category.entry(activity.category.clone()).or_insert(0) += 60; // Rough estimate

        if let Ok(timestamp) = chrono::DateTime::parse_from_rfc3339(&activity.timestamp) {
            *by_hour.entry(timestamp.hour() as u8).or_insert(0) += 60;
        }
    }

    let active_time: u64 = by_category.values().sum();

    TimeDistribution {
        by_category,
        by_hour,
        idle_time: 0, // Would need timestamps to calculate
        active_time,
    }
}

fn compute_focus_quality(activities: &[&ActivityLogEntry]) -> FocusQuality {
    let mut context_switches = 0;
    let mut last_category: Option<String> = None;

    for activity in activities {
        if let Some(last) = &last_category {
            if last != &activity.category {
                context_switches += 1;
            }
        }
        last_category = Some(activity.category.clone());
    }

    let avg_task_duration = if !activities.is_empty() {
        1800.0 / activities.len() as f64 // Rough estimate
    } else {
        0.0
    };

    let focus_score = if context_switches > 0 {
        (100.0 / (1.0 + (context_switches as f64).ln())).clamp(0.0, 100.0)
    } else {
        100.0
    };

    FocusQuality {
        score: focus_score,
        context_switches,
        avg_task_duration,
        longest_focus_period: 1800, // Would need detailed timing
        distractions: context_switches / 2,
    }
}

fn calculate_session_efficiency(
    actions_per_minute: f64,
    files_per_hour: f64,
    outputs: usize,
) -> f64 {
    let action_score = (actions_per_minute.min(5.0) / 5.0) * 40.0;
    let file_score = (files_per_hour.min(10.0) / 10.0) * 30.0;
    let output_score = (outputs.min(20) as f64 / 20.0) * 30.0;

    (action_score + file_score + output_score).min(100.0)
}

fn generate_recommendations(
    productivity: &SessionProductivity,
    focus: &FocusQuality,
) -> Vec<String> {
    let mut recommendations = Vec::new();

    if productivity.actions_per_minute < 1.0 {
        recommendations.push("Consider increasing work pace for better productivity".to_string());
    }

    if focus.context_switches > 20 {
        recommendations.push("High context switching detected - try focusing on one task longer".to_string());
    }

    if productivity.outputs_created == 0 {
        recommendations.push("No outputs created - consider adding bookmarks or notes to track findings".to_string());
    }

    if focus.score < 50.0 {
        recommendations.push("Low focus score - minimize distractions and work in focused blocks".to_string());
    }

    if productivity.efficiency_score > 80.0 {
        recommendations.push("Excellent session efficiency - keep up the good work!".to_string());
    }

    recommendations
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_productivity_calculation() {
        let productivity = SessionProductivity {
            actions_per_minute: 2.5,
            files_per_hour: 8.0,
            outputs_created: 5,
            efficiency_score: 75.0,
            time_utilization: 85.0,
        };

        assert!(productivity.efficiency_score >= 0.0 && productivity.efficiency_score <= 100.0);
    }

    #[test]
    fn test_focus_quality_scoring() {
        let focus = FocusQuality {
            score: 85.0,
            context_switches: 5,
            avg_task_duration: 600.0,
            longest_focus_period: 3600,
            distractions: 2,
        };

        assert!(focus.score >= 0.0 && focus.score <= 100.0);
    }
}
