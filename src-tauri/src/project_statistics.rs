// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Project statistics, analytics, and insights.
//!
//! Tracks and analyzes:
//! - File operation statistics
//! - Time spent on tasks
//! - Activity patterns
//! - Productivity metrics
//! - Work session analytics

use crate::project::FFXProject;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::Timelike;

/// Comprehensive project statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStatistics {
    /// Project ID
    pub project_id: String,
    /// Project name
    pub project_name: String,
    /// Statistics generation timestamp
    pub generated_at: String,
    /// Time span covered by statistics
    pub time_span: TimeSpan,
    /// File operation stats
    pub file_operations: FileOperationStats,
    /// Hash computation stats
    pub hash_operations: HashOperationStats,
    /// Session stats
    pub session_stats: SessionStats,
    /// Activity patterns
    pub activity_patterns: ActivityPatterns,
    /// Productivity metrics
    pub productivity: ProductivityMetrics,
    /// User statistics
    pub users: Vec<UserStatistics>,
}

/// Time span for statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSpan {
    /// Start timestamp
    pub start: String,
    /// End timestamp
    pub end: String,
    /// Duration in seconds
    pub duration_seconds: u64,
}

/// File operation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOperationStats {
    /// Total files opened
    pub files_opened: usize,
    /// Total files viewed
    pub files_viewed: usize,
    /// Total files exported
    pub files_exported: usize,
    /// Unique files accessed
    pub unique_files: usize,
    /// Most accessed files
    pub most_accessed: Vec<FileAccessSummary>,
    /// Files by type
    pub by_type: HashMap<String, usize>,
    /// Total size processed (bytes)
    pub total_bytes_processed: u64,
}

/// Hash operation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashOperationStats {
    /// Total hashes computed
    pub total_computed: usize,
    /// Hashes by algorithm
    pub by_algorithm: HashMap<String, usize>,
    /// Total bytes hashed
    pub total_bytes_hashed: u64,
    /// Average hash time (seconds)
    pub avg_hash_time_seconds: f64,
    /// Successful verifications
    pub verifications_passed: usize,
    /// Failed verifications
    pub verifications_failed: usize,
}

/// Session statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    /// Total sessions
    pub total_sessions: usize,
    /// Total time spent (seconds)
    pub total_time_seconds: u64,
    /// Average session duration (seconds)
    pub avg_session_duration: f64,
    /// Longest session (seconds)
    pub longest_session: u64,
    /// Shortest session (seconds)
    pub shortest_session: u64,
    /// Sessions by day of week
    pub by_day_of_week: HashMap<String, usize>,
    /// Sessions by hour of day
    pub by_hour_of_day: HashMap<u8, usize>,
}

/// Activity patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityPatterns {
    /// Most active days
    pub most_active_days: Vec<DayActivity>,
    /// Most active hours
    pub most_active_hours: Vec<HourActivity>,
    /// Activity by category
    pub by_category: HashMap<String, usize>,
    /// Peak activity time
    pub peak_time: Option<PeakActivity>,
    /// Activity trend
    pub trend: ActivityTrend,
}

/// Day activity summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayActivity {
    /// Date (YYYY-MM-DD)
    pub date: String,
    /// Activity count
    pub count: usize,
    /// Duration (seconds)
    pub duration_seconds: u64,
}

/// Hour activity summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourActivity {
    /// Hour (0-23)
    pub hour: u8,
    /// Activity count
    pub count: usize,
}

/// Peak activity period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeakActivity {
    /// Time period
    pub period: String,
    /// Activity count
    pub count: usize,
    /// Start timestamp
    pub start: String,
    /// End timestamp
    pub end: String,
}

/// Activity trend
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActivityTrend {
    /// Activity increasing over time
    Increasing,
    /// Activity decreasing over time
    Decreasing,
    /// Activity stable
    Stable,
    /// Not enough data
    Unknown,
}

/// Productivity metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductivityMetrics {
    /// Actions per hour
    pub actions_per_hour: f64,
    /// Files per session
    pub files_per_session: f64,
    /// Bookmarks created
    pub bookmarks_created: usize,
    /// Notes created
    pub notes_created: usize,
    /// Reports generated
    pub reports_generated: usize,
    /// Searches performed
    pub searches_performed: usize,
    /// Efficiency score (0-100)
    pub efficiency_score: f64,
}

/// User-specific statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStatistics {
    /// Username
    pub username: String,
    /// Total sessions
    pub sessions: usize,
    /// Total time (seconds)
    pub total_time_seconds: u64,
    /// Activities performed
    pub activities: usize,
    /// Files accessed
    pub files_accessed: usize,
    /// Last access timestamp
    pub last_access: String,
}

/// File access summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAccessSummary {
    /// File path
    pub path: String,
    /// Access count
    pub count: usize,
    /// Last accessed
    pub last_accessed: String,
    /// File type
    pub file_type: Option<String>,
}

/// Compute comprehensive project statistics
pub fn compute_statistics(project: &FFXProject) -> ProjectStatistics {
    let now = chrono::Utc::now().to_rfc3339();

    // Determine time span
    let time_span = compute_time_span(project);

    // File operations
    let file_operations = compute_file_operations(project);

    // Hash operations
    let hash_operations = compute_hash_operations(project);

    // Session stats
    let session_stats = compute_session_stats(project);

    // Activity patterns
    let activity_patterns = compute_activity_patterns(project);

    // Productivity metrics
    let productivity = compute_productivity_metrics(project, &session_stats, &file_operations);

    // User statistics
    let users = compute_user_statistics(project);

    ProjectStatistics {
        project_id: project.project_id.clone(),
        project_name: project.name.clone(),
        generated_at: now,
        time_span,
        file_operations,
        hash_operations,
        session_stats,
        activity_patterns,
        productivity,
        users,
    }
}

fn compute_time_span(project: &FFXProject) -> TimeSpan {
    let start = project.created_at.clone();
    let end = project.saved_at.clone();

    // Parse timestamps
    let start_time = chrono::DateTime::parse_from_rfc3339(&start).ok();
    let end_time = chrono::DateTime::parse_from_rfc3339(&end).ok();

    let duration_seconds = match (start_time, end_time) {
        (Some(s), Some(e)) => (e - s).num_seconds() as u64,
        _ => 0,
    };

    TimeSpan {
        start,
        end,
        duration_seconds,
    }
}

fn compute_file_operations(project: &FFXProject) -> FileOperationStats {
    let mut files_opened = 0;
    let mut files_viewed = 0;
    let mut files_exported = 0;
    let mut unique_files = std::collections::HashSet::new();
    let mut file_access_counts: HashMap<String, usize> = HashMap::new();
    let mut by_type: HashMap<String, usize> = HashMap::new();

    // Analyze activity log
    for entry in &project.activity_log {
        match entry.category.as_str() {
            "file" => {
                if entry.action.contains("open") {
                    files_opened += 1;
                }
                if entry.action.contains("view") {
                    files_viewed += 1;
                }
                if let Some(path) = &entry.file_path {
                    unique_files.insert(path.clone());
                    *file_access_counts.entry(path.clone()).or_insert(0) += 1;

                    // Determine file type
                    if let Some(ext) = std::path::Path::new(path).extension() {
                        let ext_str = ext.to_string_lossy().to_uppercase();
                        *by_type.entry(ext_str).or_insert(0) += 1;
                    }
                }
            }
            "export" => {
                files_exported += 1;
            }
            _ => {}
        }
    }

    // Most accessed files
    let mut most_accessed: Vec<_> = file_access_counts
        .into_iter()
        .map(|(path, count)| {
            let file_type = std::path::Path::new(&path)
                .extension()
                .map(|s| s.to_string_lossy().to_uppercase().to_string());
            
            // Find last access in activity log
            let last_accessed = project
                .activity_log
                .iter()
                .rev()
                .find(|e| e.file_path.as_ref() == Some(&path))
                .map(|e| e.timestamp.clone())
                .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

            FileAccessSummary {
                path,
                count,
                last_accessed,
                file_type,
            }
        })
        .collect();
    most_accessed.sort_by(|a, b| b.count.cmp(&a.count));
    most_accessed.truncate(10);

    FileOperationStats {
        files_opened,
        files_viewed,
        files_exported,
        unique_files: unique_files.len(),
        most_accessed,
        by_type,
        total_bytes_processed: 0, // Would need to be tracked in activity details
    }
}

fn compute_hash_operations(project: &FFXProject) -> HashOperationStats {
    let mut total_computed = 0;
    let mut by_algorithm: HashMap<String, usize> = HashMap::new();
    let mut verifications_passed = 0;
    let mut verifications_failed = 0;

    // Count hashes in history
    for file_hashes in project.hash_history.files.values() {
        for hash in file_hashes {
            total_computed += 1;
            *by_algorithm.entry(hash.algorithm.clone()).or_insert(0) += 1;

            if let Some(verification) = &hash.verification {
                if verification.result.to_lowercase() == "success" || verification.result.to_lowercase() == "verified" {
                    verifications_passed += 1;
                } else {
                    verifications_failed += 1;
                }
            }
        }
    }

    // Analyze hash activities
    for entry in &project.activity_log {
        if entry.category == "hash" {
            if entry.action.contains("verify") {
                if entry.description.contains("match") || entry.description.contains("success") {
                    verifications_passed += 1;
                } else if entry.description.contains("fail") || entry.description.contains("mismatch") {
                    verifications_failed += 1;
                }
            }
        }
    }

    HashOperationStats {
        total_computed,
        by_algorithm,
        total_bytes_hashed: 0, // Would need to be tracked
        avg_hash_time_seconds: 0.0, // Would need timing data
        verifications_passed,
        verifications_failed,
    }
}

fn compute_session_stats(project: &FFXProject) -> SessionStats {
    if project.sessions.is_empty() {
        return SessionStats {
            total_sessions: 0,
            total_time_seconds: 0,
            avg_session_duration: 0.0,
            longest_session: 0,
            shortest_session: 0,
            by_day_of_week: HashMap::new(),
            by_hour_of_day: HashMap::new(),
        };
    }

    let mut total_time_seconds = 0u64;
    let mut longest_session = 0u64;
    let mut shortest_session = u64::MAX;
    let mut by_day_of_week: HashMap<String, usize> = HashMap::new();
    let mut by_hour_of_day: HashMap<u8, usize> = HashMap::new();

    for session in &project.sessions {
        if let Some(duration) = session.duration_seconds {
            total_time_seconds += duration;
            longest_session = longest_session.max(duration);
            shortest_session = shortest_session.min(duration);
        }

        // Parse start time for patterns
        if let Ok(start_time) = chrono::DateTime::parse_from_rfc3339(&session.started_at) {
            let day_of_week = start_time.format("%A").to_string();
            *by_day_of_week.entry(day_of_week).or_insert(0) += 1;

            let hour = start_time.hour() as u8;
            *by_hour_of_day.entry(hour).or_insert(0) += 1;
        }
    }

    let avg_session_duration = if !project.sessions.is_empty() {
        total_time_seconds as f64 / project.sessions.len() as f64
    } else {
        0.0
    };

    SessionStats {
        total_sessions: project.sessions.len(),
        total_time_seconds,
        avg_session_duration,
        longest_session,
        shortest_session: if shortest_session == u64::MAX { 0 } else { shortest_session },
        by_day_of_week,
        by_hour_of_day,
    }
}

fn compute_activity_patterns(project: &FFXProject) -> ActivityPatterns {
    let mut by_category: HashMap<String, usize> = HashMap::new();
    let mut day_counts: HashMap<String, usize> = HashMap::new();
    let mut hour_counts: HashMap<u8, usize> = HashMap::new();

    for entry in &project.activity_log {
        *by_category.entry(entry.category.clone()).or_insert(0) += 1;

        if let Ok(timestamp) = chrono::DateTime::parse_from_rfc3339(&entry.timestamp) {
            let date = timestamp.format("%Y-%m-%d").to_string();
            *day_counts.entry(date).or_insert(0) += 1;

            let hour = timestamp.hour() as u8;
            *hour_counts.entry(hour).or_insert(0) += 1;
        }
    }

    // Most active days
    let mut most_active_days: Vec<DayActivity> = day_counts
        .into_iter()
        .map(|(date, count)| DayActivity {
            date,
            count,
            duration_seconds: 0, // Would need session data
        })
        .collect();
    most_active_days.sort_by(|a, b| b.count.cmp(&a.count));
    most_active_days.truncate(7);

    // Most active hours
    let mut most_active_hours: Vec<HourActivity> = hour_counts
        .into_iter()
        .map(|(hour, count)| HourActivity { hour, count })
        .collect();
    most_active_hours.sort_by(|a, b| b.count.cmp(&a.count));

    // Determine trend (simple heuristic)
    let trend = if project.activity_log.len() < 10 {
        ActivityTrend::Unknown
    } else {
        let mid_point = project.activity_log.len() / 2;
        let first_half = &project.activity_log[0..mid_point];
        let second_half = &project.activity_log[mid_point..];

        let first_rate = first_half.len() as f64;
        let second_rate = second_half.len() as f64;

        if second_rate > first_rate * 1.2 {
            ActivityTrend::Increasing
        } else if second_rate < first_rate * 0.8 {
            ActivityTrend::Decreasing
        } else {
            ActivityTrend::Stable
        }
    };

    ActivityPatterns {
        most_active_days,
        most_active_hours,
        by_category,
        peak_time: None, // Would need more analysis
        trend,
    }
}

fn compute_productivity_metrics(
    project: &FFXProject,
    session_stats: &SessionStats,
    file_ops: &FileOperationStats,
) -> ProductivityMetrics {
    let actions_per_hour = if session_stats.total_time_seconds > 0 {
        (project.activity_log.len() as f64) / (session_stats.total_time_seconds as f64 / 3600.0)
    } else {
        0.0
    };

    let files_per_session = if session_stats.total_sessions > 0 {
        file_ops.unique_files as f64 / session_stats.total_sessions as f64
    } else {
        0.0
    };

    // Count specific productivity indicators
    let mut bookmarks_created = 0;
    let mut notes_created = 0;
    let mut reports_generated = 0;
    let mut searches_performed = 0;

    for entry in &project.activity_log {
        match entry.category.as_str() {
            "bookmark" if entry.action.contains("add") => bookmarks_created += 1,
            "note" if entry.action.contains("create") => notes_created += 1,
            "export" if entry.action.contains("report") => reports_generated += 1,
            "search" => searches_performed += 1,
            _ => {}
        }
    }

    // Simple efficiency score (0-100)
    let efficiency_score = calculate_efficiency_score(
        actions_per_hour,
        files_per_session,
        bookmarks_created,
        notes_created,
        reports_generated,
    );

    ProductivityMetrics {
        actions_per_hour,
        files_per_session,
        bookmarks_created,
        notes_created,
        reports_generated,
        searches_performed,
        efficiency_score,
    }
}

fn calculate_efficiency_score(
    actions_per_hour: f64,
    files_per_session: f64,
    bookmarks: usize,
    notes: usize,
    reports: usize,
) -> f64 {
    // Normalize metrics to 0-100 scale
    let action_score = (actions_per_hour.min(100.0) / 100.0) * 30.0;
    let file_score = (files_per_session.min(20.0) / 20.0) * 20.0;
    let bookmark_score = (bookmarks.min(50) as f64 / 50.0) * 15.0;
    let note_score = (notes.min(50) as f64 / 50.0) * 20.0;
    let report_score = (reports.min(20) as f64 / 20.0) * 15.0;

    (action_score + file_score + bookmark_score + note_score + report_score).min(100.0)
}

fn compute_user_statistics(project: &FFXProject) -> Vec<UserStatistics> {
    let mut user_stats: HashMap<String, UserStatistics> = HashMap::new();

    // Initialize from users
    for user in &project.users {
        user_stats.insert(
            user.username.clone(),
            UserStatistics {
                username: user.username.clone(),
                sessions: 0,
                total_time_seconds: 0,
                activities: 0,
                files_accessed: 0,
                last_access: user.last_access.clone(),
            },
        );
    }

    // Count sessions
    for session in &project.sessions {
        if let Some(stats) = user_stats.get_mut(&session.user) {
            stats.sessions += 1;
            if let Some(duration) = session.duration_seconds {
                stats.total_time_seconds += duration;
            }
        }
    }

    // Count activities
    let mut user_files: HashMap<String, std::collections::HashSet<String>> = HashMap::new();
    for entry in &project.activity_log {
        if let Some(stats) = user_stats.get_mut(&entry.user) {
            stats.activities += 1;
        }

        if let Some(path) = &entry.file_path {
            user_files
                .entry(entry.user.clone())
                .or_insert_with(std::collections::HashSet::new)
                .insert(path.clone());
        }
    }

    // Set file counts
    for (username, files) in user_files {
        if let Some(stats) = user_stats.get_mut(&username) {
            stats.files_accessed = files.len();
        }
    }

    let mut result: Vec<_> = user_stats.into_values().collect();
    result.sort_by(|a, b| b.total_time_seconds.cmp(&a.total_time_seconds));
    result
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::FFXProject;

    #[test]
    fn test_compute_statistics_empty_project() {
        let project = FFXProject::new("/test/case");
        let stats = compute_statistics(&project);

        assert_eq!(stats.project_name, "case");
        assert_eq!(stats.file_operations.files_opened, 0);
        assert_eq!(stats.session_stats.total_sessions, 0);
    }

    #[test]
    fn test_efficiency_score_calculation() {
        let score = calculate_efficiency_score(50.0, 10.0, 10, 10, 5);
        assert!(score >= 0.0 && score <= 100.0);
    }

    #[test]
    fn test_activity_trend_unknown() {
        let project = FFXProject::new("/test/case");
        let patterns = compute_activity_patterns(&project);
        assert_eq!(patterns.trend, ActivityTrend::Unknown);
    }
}
