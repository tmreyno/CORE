// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Enhanced activity timeline with visualization data.
//!
//! Provides:
//! - Timeline data for visualization (heatmap, charts)
//! - Activity aggregation and binning
//! - Work pattern analysis
//! - Exportable timeline reports

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::project::{FFXProject, ActivityLogEntry};
use chrono::{Timelike, Datelike};

/// Timeline visualization data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineVisualization {
    /// Timeline summary
    pub summary: TimelineSummary,
    /// Hourly activity heatmap data
    pub heatmap: ActivityHeatmap,
    /// Daily activity chart data
    pub daily_chart: Vec<DailyActivity>,
    /// Activity distribution by type
    pub type_distribution: Vec<TypeDistribution>,
    /// Activity by user
    pub user_activity: Vec<UserActivity>,
    /// Peak activity periods
    pub peak_periods: Vec<PeakPeriod>,
    /// Activity trends
    pub trends: ActivityTrends,
}

/// Timeline summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineSummary {
    /// Total activities
    pub total_activities: usize,
    /// Unique users
    pub unique_users: usize,
    /// Date range (start, end)
    pub date_range: (String, String),
    /// Total duration (hours)
    pub total_duration_hours: f64,
    /// Most active day
    pub most_active_day: String,
    /// Most active hour
    pub most_active_hour: u32,
    /// Average activities per session
    pub avg_activities_per_session: f64,
}

/// Activity heatmap data (24 hours x 7 days)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityHeatmap {
    /// Heatmap data [day][hour] = count
    /// Days: 0=Sunday, 1=Monday, ..., 6=Saturday
    pub data: Vec<Vec<usize>>,
    /// Maximum value (for color scaling)
    pub max_value: usize,
    /// Labels for days
    pub day_labels: Vec<String>,
    /// Labels for hours
    pub hour_labels: Vec<String>,
}

/// Daily activity data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyActivity {
    /// Date (YYYY-MM-DD)
    pub date: String,
    /// Day of week (0=Sunday)
    pub day_of_week: u32,
    /// Total activities
    pub count: usize,
    /// Activities by type
    pub by_type: HashMap<String, usize>,
    /// Duration (minutes)
    pub duration_minutes: f64,
    /// Unique users active
    pub unique_users: usize,
}

/// Activity distribution by type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeDistribution {
    /// Activity type
    pub activity_type: String,
    /// Count
    pub count: usize,
    /// Percentage
    pub percentage: f64,
    /// Color for visualization
    pub color: String,
}

/// User activity data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserActivity {
    /// User name
    pub user: String,
    /// Total activities
    pub total_activities: usize,
    /// Activities by type
    pub by_type: HashMap<String, usize>,
    /// First activity timestamp
    pub first_activity: String,
    /// Last activity timestamp
    pub last_activity: String,
    /// Active days count
    pub active_days: usize,
}

/// Peak activity period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeakPeriod {
    /// Start time
    pub start_time: String,
    /// End time
    pub end_time: String,
    /// Duration (minutes)
    pub duration_minutes: f64,
    /// Activity count
    pub activity_count: usize,
    /// Activities per minute
    pub activities_per_minute: f64,
    /// Description
    pub description: String,
}

/// Activity trends over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityTrends {
    /// Overall trend (increasing, decreasing, stable)
    pub overall_trend: String,
    /// Trend by activity type
    pub by_type: HashMap<String, String>,
    /// Weekly average activities
    pub weekly_avg: f64,
    /// Trend confidence (0-1)
    pub confidence: f64,
    /// Insights
    pub insights: Vec<String>,
}

/// Timeline export format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineExport {
    /// Export metadata
    pub metadata: ExportMetadata,
    /// All activities in export
    pub activities: Vec<ActivityExportEntry>,
    /// Statistics
    pub statistics: TimelineSummary,
}

/// Export metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    /// Project name
    pub project_name: String,
    /// Export date
    pub export_date: String,
    /// Exported by user
    pub exported_by: String,
    /// Export format version
    pub format_version: String,
}

/// Activity export entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityExportEntry {
    /// Timestamp
    pub timestamp: String,
    /// Activity type
    pub activity_type: String,
    /// Description
    pub description: String,
    /// User
    pub user: String,
    /// Details (JSON)
    pub details: serde_json::Value,
}

/// Compute timeline visualization data
pub fn compute_timeline_visualization(project: &FFXProject) -> TimelineVisualization {
    let summary = compute_timeline_summary(project);
    let heatmap = compute_activity_heatmap(project);
    let daily_chart = compute_daily_activities(project);
    let type_distribution = compute_type_distribution(project);
    let user_activity = compute_user_activity(project);
    let peak_periods = identify_peak_periods(project);
    let trends = analyze_activity_trends(project);

    TimelineVisualization {
        summary,
        heatmap,
        daily_chart,
        type_distribution,
        user_activity,
        peak_periods,
        trends,
    }
}

/// Compute timeline summary
fn compute_timeline_summary(project: &FFXProject) -> TimelineSummary {
    let activities = &project.activity_log;
    let total_activities = activities.len();

    if total_activities == 0 {
        return TimelineSummary {
            total_activities: 0,
            unique_users: 0,
            date_range: (String::new(), String::new()),
            total_duration_hours: 0.0,
            most_active_day: String::new(),
            most_active_hour: 0,
            avg_activities_per_session: 0.0,
        };
    }

    // Unique users
    let unique_users: std::collections::HashSet<_> =
        activities.iter().map(|a| a.user.clone()).collect();

    // Date range
    let timestamps: Vec<_> = activities
        .iter()
        .filter_map(|a| chrono::DateTime::parse_from_rfc3339(&a.timestamp).ok())
        .collect();

    let (start, end) = if !timestamps.is_empty() {
        let min = timestamps.iter().min().unwrap();
        let max = timestamps.iter().max().unwrap();
        (min.to_rfc3339(), max.to_rfc3339())
    } else {
        (String::new(), String::new())
    };

    // Total duration
    let total_duration_hours = if timestamps.len() >= 2 {
        let duration = timestamps.iter().max().unwrap().signed_duration_since(*timestamps.iter().min().unwrap());
        duration.num_hours() as f64 + (duration.num_minutes() % 60) as f64 / 60.0
    } else {
        0.0
    };

    // Most active day and hour
    let mut day_counts: HashMap<String, usize> = HashMap::new();
    let mut hour_counts: HashMap<u32, usize> = HashMap::new();

    for activity in activities {
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&activity.timestamp) {
            let date = dt.format("%Y-%m-%d").to_string();
            *day_counts.entry(date).or_insert(0) += 1;
            *hour_counts.entry(dt.hour()).or_insert(0) += 1;
        }
    }

    let most_active_day = day_counts
        .iter()
        .max_by_key(|(_, count)| *count)
        .map(|(day, _)| day.clone())
        .unwrap_or_default();

    let most_active_hour = hour_counts
        .iter()
        .max_by_key(|(_, count)| *count)
        .map(|(hour, _)| *hour)
        .unwrap_or(0);

    // Average activities per session
    let session_count = project.sessions.len().max(1);
    let avg_activities_per_session = total_activities as f64 / session_count as f64;

    TimelineSummary {
        total_activities,
        unique_users: unique_users.len(),
        date_range: (start, end),
        total_duration_hours,
        most_active_day,
        most_active_hour,
        avg_activities_per_session,
    }
}

/// Compute activity heatmap (day of week x hour of day)
fn compute_activity_heatmap(project: &FFXProject) -> ActivityHeatmap {
    let mut data = vec![vec![0usize; 24]; 7]; // 7 days x 24 hours
    let mut max_value = 0usize;

    for activity in &project.activity_log {
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&activity.timestamp) {
            let day = dt.weekday().num_days_from_sunday() as usize;
            let hour = dt.hour() as usize;
            data[day][hour] += 1;
            max_value = max_value.max(data[day][hour]);
        }
    }

    ActivityHeatmap {
        data,
        max_value,
        day_labels: vec![
            "Sun".to_string(),
            "Mon".to_string(),
            "Tue".to_string(),
            "Wed".to_string(),
            "Thu".to_string(),
            "Fri".to_string(),
            "Sat".to_string(),
        ],
        hour_labels: (0..24).map(|h| format!("{:02}:00", h)).collect(),
    }
}

/// Compute daily activities
fn compute_daily_activities(project: &FFXProject) -> Vec<DailyActivity> {
    let mut daily_map: HashMap<String, DailyActivity> = HashMap::new();

    for activity in &project.activity_log {
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&activity.timestamp) {
            let date = dt.format("%Y-%m-%d").to_string();
            let day_of_week = dt.weekday().num_days_from_sunday();

            let entry = daily_map.entry(date.clone()).or_insert(DailyActivity {
                date: date.clone(),
                day_of_week,
                count: 0,
                by_type: HashMap::new(),
                duration_minutes: 0.0,
                unique_users: 0,
            });

            entry.count += 1;
            *entry.by_type.entry(activity.action.clone()).or_insert(0) += 1;
        }
    }

    // Count unique users per day
    for (date, entry) in daily_map.iter_mut() {
        let users: std::collections::HashSet<_> = project
            .activity_log
            .iter()
            .filter(|a| {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&a.timestamp) {
                    dt.format("%Y-%m-%d").to_string() == *date
                } else {
                    false
                }
            })
            .map(|a| a.user.clone())
            .collect();
        entry.unique_users = users.len();
    }

    let mut result: Vec<_> = daily_map.into_values().collect();
    result.sort_by(|a, b| a.date.cmp(&b.date));
    result
}

/// Compute type distribution
fn compute_type_distribution(project: &FFXProject) -> Vec<TypeDistribution> {
    let mut type_counts: HashMap<String, usize> = HashMap::new();
    let total = project.activity_log.len();

    for activity in &project.activity_log {
        *type_counts.entry(activity.action.clone()).or_insert(0) += 1;
    }

    let colors = vec![
        "#3B82F6", "#10B981", "#F59E0B", "#EF4444", "#8B5CF6",
        "#EC4899", "#06B6D4", "#84CC16", "#F97316", "#6366F1",
    ];

    let mut result: Vec<_> = type_counts
        .into_iter()
        .enumerate()
        .map(|(i, (activity_type, count))| TypeDistribution {
            activity_type,
            count,
            percentage: if total > 0 { (count as f64 / total as f64) * 100.0 } else { 0.0 },
            color: colors[i % colors.len()].to_string(),
        })
        .collect();

    result.sort_by(|a, b| b.count.cmp(&a.count));
    result
}

/// Compute user activity
fn compute_user_activity(project: &FFXProject) -> Vec<UserActivity> {
    let mut user_map: HashMap<String, UserActivity> = HashMap::new();

    for activity in &project.activity_log {
        let entry = user_map.entry(activity.user.clone()).or_insert(UserActivity {
            user: activity.user.clone(),
            total_activities: 0,
            by_type: HashMap::new(),
            first_activity: activity.timestamp.clone(),
            last_activity: activity.timestamp.clone(),
            active_days: 0,
        });

        entry.total_activities += 1;
        *entry.by_type.entry(activity.action.clone()).or_insert(0) += 1;

        // Update timestamps
        if activity.timestamp < entry.first_activity {
            entry.first_activity = activity.timestamp.clone();
        }
        if activity.timestamp > entry.last_activity {
            entry.last_activity = activity.timestamp.clone();
        }
    }

    // Count active days per user
    for (user, entry) in user_map.iter_mut() {
        let dates: std::collections::HashSet<_> = project
            .activity_log
            .iter()
            .filter(|a| a.user == *user)
            .filter_map(|a| {
                chrono::DateTime::parse_from_rfc3339(&a.timestamp)
                    .ok()
                    .map(|dt| dt.format("%Y-%m-%d").to_string())
            })
            .collect();
        entry.active_days = dates.len();
    }

    let mut result: Vec<_> = user_map.into_values().collect();
    result.sort_by(|a, b| b.total_activities.cmp(&a.total_activities));
    result
}

/// Identify peak activity periods
fn identify_peak_periods(project: &FFXProject) -> Vec<PeakPeriod> {
    // Group activities into 30-minute windows
    let mut windows: HashMap<String, Vec<&ActivityLogEntry>> = HashMap::new();

    for activity in &project.activity_log {
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&activity.timestamp) {
            // Round down to 30-minute window
            let window_start = dt
                .with_minute((dt.minute() / 30) * 30)
                .unwrap()
                .with_second(0)
                .unwrap();
            let key = window_start.to_rfc3339();
            windows.entry(key).or_insert_with(Vec::new).push(activity);
        }
    }

    // Find top 5 peak periods
    let mut peaks: Vec<_> = windows
        .into_iter()
        .map(|(start, activities)| {
            let count = activities.len();
            let start_dt = chrono::DateTime::parse_from_rfc3339(&start).unwrap();
            let end_dt = start_dt + chrono::Duration::minutes(30);

            PeakPeriod {
                start_time: start,
                end_time: end_dt.to_rfc3339(),
                duration_minutes: 30.0,
                activity_count: count,
                activities_per_minute: count as f64 / 30.0,
                description: format!(
                    "{} activities in 30 minutes",
                    count
                ),
            }
        })
        .collect();

    peaks.sort_by(|a, b| b.activity_count.cmp(&a.activity_count));
    peaks.truncate(5);
    peaks
}

/// Analyze activity trends
fn analyze_activity_trends(project: &FFXProject) -> ActivityTrends {
    let daily_activities = compute_daily_activities(project);

    if daily_activities.len() < 2 {
        return ActivityTrends {
            overall_trend: "stable".to_string(),
            by_type: HashMap::new(),
            weekly_avg: 0.0,
            confidence: 0.0,
            insights: vec!["Insufficient data for trend analysis".to_string()],
        };
    }

    // Split into first and second half
    let mid = daily_activities.len() / 2;
    let first_half: Vec<_> = daily_activities[..mid].to_vec();
    let second_half: Vec<_> = daily_activities[mid..].to_vec();

    let first_avg = first_half.iter().map(|d| d.count).sum::<usize>() as f64 / first_half.len() as f64;
    let second_avg = second_half.iter().map(|d| d.count).sum::<usize>() as f64 / second_half.len() as f64;

    let change = (second_avg - first_avg) / first_avg;
    let overall_trend = if change > 0.2 {
        "increasing"
    } else if change < -0.2 {
        "decreasing"
    } else {
        "stable"
    };

    // Weekly average
    let total_days = daily_activities.len() as f64;
    let total_activities: usize = daily_activities.iter().map(|d| d.count).sum();
    let weekly_avg = (total_activities as f64 / total_days) * 7.0;

    // Confidence based on data points
    let confidence = (daily_activities.len() as f64 / 30.0).min(1.0);

    // Generate insights
    let mut insights = Vec::new();
    if overall_trend == "increasing" {
        insights.push(format!("Activity increased by {:.1}% in recent period", change * 100.0));
    } else if overall_trend == "decreasing" {
        insights.push(format!("Activity decreased by {:.1}% in recent period", change.abs() * 100.0));
    }

    if weekly_avg > 100.0 {
        insights.push("High activity level maintained".to_string());
    }

    ActivityTrends {
        overall_trend: overall_trend.to_string(),
        by_type: HashMap::new(), // TODO: Implement per-type trends
        weekly_avg,
        confidence,
        insights,
    }
}

/// Export timeline to JSON
pub fn export_timeline(project: &FFXProject, exported_by: String) -> TimelineExport {
    let activities: Vec<ActivityExportEntry> = project
        .activity_log
        .iter()
        .map(|a| ActivityExportEntry {
            timestamp: a.timestamp.clone(),
            activity_type: a.action.clone(),
            description: a.description.clone(),
            user: a.user.clone(),
            details: a.details.as_ref()
                .map(|d| serde_json::to_value(d).unwrap_or(serde_json::Value::Null))
                .unwrap_or(serde_json::Value::Null),
        })
        .collect();

    let statistics = compute_timeline_summary(project);

    TimelineExport {
        metadata: ExportMetadata {
            project_name: project.name.clone(),
            export_date: chrono::Utc::now().to_rfc3339(),
            exported_by,
            format_version: "1.0".to_string(),
        },
        activities,
        statistics,
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_timeline() {
        let project = FFXProject::new("/tmp/test_project");
        let viz = compute_timeline_visualization(&project);
        assert_eq!(viz.summary.total_activities, 0);
    }

    #[test]
    fn test_heatmap_dimensions() {
        let project = FFXProject::new("/tmp/test_project");
        let heatmap = compute_activity_heatmap(&project);
        assert_eq!(heatmap.data.len(), 7); // 7 days
        assert_eq!(heatmap.data[0].len(), 24); // 24 hours
    }
}
