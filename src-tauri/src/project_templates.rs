// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Project templates for rapid case initialization.
//!
//! Provides:
//! - Pre-configured project templates for common scenarios
//! - Template metadata and categorization
//! - Template application to new/existing projects
//! - Custom template creation and sharing

use crate::project::FFXProject;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Template category
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TemplateCategory {
    /// Mobile device forensics
    Mobile,
    /// Computer forensics
    Computer,
    /// Network forensics
    Network,
    /// Cloud forensics
    Cloud,
    /// Incident response
    IncidentResponse,
    /// Memory analysis
    Memory,
    /// Malware analysis
    Malware,
    /// E-discovery
    EDiscovery,
    /// General purpose
    General,
    /// Custom user template
    Custom,
}

impl TemplateCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            TemplateCategory::Mobile => "Mobile",
            TemplateCategory::Computer => "Computer",
            TemplateCategory::Network => "Network",
            TemplateCategory::Cloud => "Cloud",
            TemplateCategory::IncidentResponse => "Incident Response",
            TemplateCategory::Memory => "Memory Analysis",
            TemplateCategory::Malware => "Malware Analysis",
            TemplateCategory::EDiscovery => "E-Discovery",
            TemplateCategory::General => "General",
            TemplateCategory::Custom => "Custom",
        }
    }
}

/// Project template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectTemplate {
    /// Template ID
    pub id: String,
    /// Template name
    pub name: String,
    /// Template category
    pub category: TemplateCategory,
    /// Template description
    pub description: String,
    /// Template author
    pub author: String,
    /// Template version
    pub version: String,
    /// Creation date
    pub created_at: String,
    /// Last updated date
    pub updated_at: String,
    /// Usage count
    pub usage_count: usize,
    /// Template tags
    pub tags: Vec<String>,
    /// Pre-configured bookmarks
    pub bookmarks: Vec<BookmarkTemplate>,
    /// Pre-configured notes
    pub notes: Vec<NoteTemplate>,
    /// Pre-configured tabs
    pub tabs: Vec<TabTemplate>,
    /// Hash algorithm presets
    pub hash_algorithms: Vec<String>,
    /// Tool recommendations
    pub recommended_tools: Vec<String>,
    /// Checklist items
    pub checklist: Vec<ChecklistItem>,
    /// Custom metadata fields
    pub metadata_fields: Vec<MetadataField>,
    /// Workspace profile ID
    pub workspace_profile: Option<String>,
}

/// Bookmark template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkTemplate {
    /// Bookmark name
    pub name: String,
    /// Bookmark description
    pub description: String,
    /// Category
    pub category: String,
    /// Tags
    pub tags: Vec<String>,
}

/// Note template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteTemplate {
    /// Note title
    pub title: String,
    /// Note content (markdown)
    pub content: String,
    /// Note category
    pub category: String,
    /// Tags
    pub tags: Vec<String>,
}

/// Tab template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabTemplate {
    /// Tab name
    pub name: String,
    /// Tab type (evidence, analysis, etc.)
    pub tab_type: String,
    /// Tab configuration
    pub config: HashMap<String, serde_json::Value>,
}

/// Checklist item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistItem {
    /// Item ID
    pub id: String,
    /// Item text
    pub text: String,
    /// Item category
    pub category: String,
    /// Required or optional
    pub required: bool,
    /// Default completion status
    pub completed: bool,
    /// Help text
    pub help: Option<String>,
}

/// Metadata field definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataField {
    /// Field name
    pub name: String,
    /// Field type (text, number, date, etc.)
    pub field_type: String,
    /// Default value
    pub default_value: Option<String>,
    /// Required field
    pub required: bool,
    /// Help text
    pub help: Option<String>,
}

/// Template manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateManager {
    /// All templates
    pub templates: Vec<ProjectTemplate>,
}

impl Default for TemplateManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateManager {
    /// Create new template manager with default templates
    pub fn new() -> Self {
        let mut manager = TemplateManager {
            templates: Vec::new(),
        };

        // Add default templates
        manager.templates.push(Self::create_mobile_template());
        manager.templates.push(Self::create_computer_template());
        manager
            .templates
            .push(Self::create_incident_response_template());
        manager.templates.push(Self::create_malware_template());
        manager.templates.push(Self::create_ediscovery_template());

        manager
    }

    /// Create mobile forensics template
    fn create_mobile_template() -> ProjectTemplate {
        ProjectTemplate {
            id: "mobile_forensics".to_string(),
            name: "Mobile Device Forensics".to_string(),
            category: TemplateCategory::Mobile,
            description: "Template for mobile device investigation (iOS/Android)".to_string(),
            author: "CORE-FFX".to_string(),
            version: "1.0".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            usage_count: 0,
            tags: vec!["mobile".to_string(), "ios".to_string(), "android".to_string()],
            bookmarks: vec![
                BookmarkTemplate {
                    name: "SMS/Messages".to_string(),
                    description: "Text messages and iMessages".to_string(),
                    category: "Communications".to_string(),
                    tags: vec!["messages".to_string(), "sms".to_string()],
                },
                BookmarkTemplate {
                    name: "Call History".to_string(),
                    description: "Phone call logs".to_string(),
                    category: "Communications".to_string(),
                    tags: vec!["calls".to_string(), "phone".to_string()],
                },
                BookmarkTemplate {
                    name: "Contacts".to_string(),
                    description: "Address book entries".to_string(),
                    category: "Communications".to_string(),
                    tags: vec!["contacts".to_string()],
                },
                BookmarkTemplate {
                    name: "Photos/Videos".to_string(),
                    description: "Media files from camera roll".to_string(),
                    category: "Media".to_string(),
                    tags: vec!["photos".to_string(), "videos".to_string()],
                },
                BookmarkTemplate {
                    name: "App Data".to_string(),
                    description: "Application-specific data".to_string(),
                    category: "Applications".to_string(),
                    tags: vec!["apps".to_string(), "data".to_string()],
                },
            ],
            notes: vec![
                NoteTemplate {
                    title: "Device Information".to_string(),
                    content: "**Device Details:**\n- Device Type:\n- OS Version:\n- IMEI/Serial:\n- Acquisition Date:\n- Examiner:".to_string(),
                    category: "Case Notes".to_string(),
                    tags: vec!["device".to_string(), "metadata".to_string()],
                },
                NoteTemplate {
                    title: "Key Findings".to_string(),
                    content: "**Important Findings:**\n\n1. \n2. \n3. ".to_string(),
                    category: "Analysis".to_string(),
                    tags: vec!["findings".to_string()],
                },
            ],
            tabs: vec![
                TabTemplate {
                    name: "Evidence".to_string(),
                    tab_type: "evidence".to_string(),
                    config: HashMap::new(),
                },
                TabTemplate {
                    name: "Timeline".to_string(),
                    tab_type: "timeline".to_string(),
                    config: HashMap::new(),
                },
            ],
            hash_algorithms: vec!["SHA-256".to_string(), "MD5".to_string()],
            recommended_tools: vec![
                "plist_viewer".to_string(),
                "sqlite_viewer".to_string(),
                "hex_viewer".to_string(),
            ],
            checklist: vec![
                ChecklistItem {
                    id: "device_info".to_string(),
                    text: "Document device information".to_string(),
                    category: "Initial".to_string(),
                    required: true,
                    completed: false,
                    help: Some("Record make, model, OS version, IMEI, serial number".to_string()),
                },
                ChecklistItem {
                    id: "acquisition".to_string(),
                    text: "Verify acquisition integrity".to_string(),
                    category: "Initial".to_string(),
                    required: true,
                    completed: false,
                    help: Some("Verify hash values of acquired data".to_string()),
                },
                ChecklistItem {
                    id: "messages".to_string(),
                    text: "Review messages and communications".to_string(),
                    category: "Analysis".to_string(),
                    required: false,
                    completed: false,
                    help: None,
                },
                ChecklistItem {
                    id: "apps".to_string(),
                    text: "Analyze installed applications".to_string(),
                    category: "Analysis".to_string(),
                    required: false,
                    completed: false,
                    help: None,
                },
            ],
            metadata_fields: vec![
                MetadataField {
                    name: "case_number".to_string(),
                    field_type: "text".to_string(),
                    default_value: None,
                    required: true,
                    help: Some("Case or incident number".to_string()),
                },
                MetadataField {
                    name: "device_owner".to_string(),
                    field_type: "text".to_string(),
                    default_value: None,
                    required: false,
                    help: Some("Name of device owner/subject".to_string()),
                },
            ],
            workspace_profile: Some("mobile".to_string()),
        }
    }

    /// Create computer forensics template
    fn create_computer_template() -> ProjectTemplate {
        ProjectTemplate {
            id: "computer_forensics".to_string(),
            name: "Computer Forensics".to_string(),
            category: TemplateCategory::Computer,
            description: "Template for computer/laptop investigation".to_string(),
            author: "CORE-FFX".to_string(),
            version: "1.0".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            usage_count: 0,
            tags: vec!["computer".to_string(), "windows".to_string(), "macos".to_string()],
            bookmarks: vec![
                BookmarkTemplate {
                    name: "System Files".to_string(),
                    description: "OS and system files".to_string(),
                    category: "System".to_string(),
                    tags: vec!["system".to_string(), "os".to_string()],
                },
                BookmarkTemplate {
                    name: "User Documents".to_string(),
                    description: "User documents and files".to_string(),
                    category: "User Data".to_string(),
                    tags: vec!["documents".to_string(), "files".to_string()],
                },
                BookmarkTemplate {
                    name: "Browser History".to_string(),
                    description: "Web browser artifacts".to_string(),
                    category: "Internet".to_string(),
                    tags: vec!["browser".to_string(), "web".to_string()],
                },
            ],
            notes: vec![
                NoteTemplate {
                    title: "System Information".to_string(),
                    content: "**Computer Details:**\n- Hostname:\n- OS:\n- Last Logged In User:\n- Acquisition Date:".to_string(),
                    category: "Case Notes".to_string(),
                    tags: vec!["system".to_string()],
                },
            ],
            tabs: vec![
                TabTemplate {
                    name: "Evidence".to_string(),
                    tab_type: "evidence".to_string(),
                    config: HashMap::new(),
                },
            ],
            hash_algorithms: vec!["SHA-256".to_string()],
            recommended_tools: vec![
                "registry_viewer".to_string(),
                "event_log_viewer".to_string(),
                "hex_viewer".to_string(),
            ],
            checklist: vec![
                ChecklistItem {
                    id: "system_info".to_string(),
                    text: "Document system information".to_string(),
                    category: "Initial".to_string(),
                    required: true,
                    completed: false,
                    help: None,
                },
                ChecklistItem {
                    id: "timeline".to_string(),
                    text: "Create timeline of events".to_string(),
                    category: "Analysis".to_string(),
                    required: false,
                    completed: false,
                    help: None,
                },
            ],
            metadata_fields: vec![],
            workspace_profile: Some("computer".to_string()),
        }
    }

    /// Create incident response template
    fn create_incident_response_template() -> ProjectTemplate {
        ProjectTemplate {
            id: "incident_response".to_string(),
            name: "Incident Response".to_string(),
            category: TemplateCategory::IncidentResponse,
            description: "Template for incident response investigations".to_string(),
            author: "CORE-FFX".to_string(),
            version: "1.0".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            usage_count: 0,
            tags: vec!["ir".to_string(), "incident".to_string(), "response".to_string()],
            bookmarks: vec![
                BookmarkTemplate {
                    name: "IOCs".to_string(),
                    description: "Indicators of Compromise".to_string(),
                    category: "Indicators".to_string(),
                    tags: vec!["ioc".to_string(), "indicators".to_string()],
                },
                BookmarkTemplate {
                    name: "Suspicious Files".to_string(),
                    description: "Potentially malicious files".to_string(),
                    category: "Malware".to_string(),
                    tags: vec!["suspicious".to_string(), "malware".to_string()],
                },
                BookmarkTemplate {
                    name: "Log Files".to_string(),
                    description: "System and application logs".to_string(),
                    category: "Logs".to_string(),
                    tags: vec!["logs".to_string(), "events".to_string()],
                },
            ],
            notes: vec![
                NoteTemplate {
                    title: "Incident Details".to_string(),
                    content: "**Incident Information:**\n- Incident Type:\n- Detection Date:\n- Reported By:\n- Affected Systems:\n- Initial Observations:".to_string(),
                    category: "Incident".to_string(),
                    tags: vec!["incident".to_string()],
                },
                NoteTemplate {
                    title: "Timeline of Events".to_string(),
                    content: "**Timeline:**\n\n| Time | Event | Source |\n|------|-------|--------|\n| | | |".to_string(),
                    category: "Timeline".to_string(),
                    tags: vec!["timeline".to_string()],
                },
            ],
            tabs: vec![],
            hash_algorithms: vec!["SHA-256".to_string(), "MD5".to_string()],
            recommended_tools: vec![
                "hash".to_string(),
                "hex_viewer".to_string(),
                "strings".to_string(),
            ],
            checklist: vec![
                ChecklistItem {
                    id: "containment".to_string(),
                    text: "Document containment actions".to_string(),
                    category: "Response".to_string(),
                    required: true,
                    completed: false,
                    help: Some("Record all containment and mitigation steps taken".to_string()),
                },
                ChecklistItem {
                    id: "iocs".to_string(),
                    text: "Identify and document IOCs".to_string(),
                    category: "Analysis".to_string(),
                    required: true,
                    completed: false,
                    help: None,
                },
                ChecklistItem {
                    id: "root_cause".to_string(),
                    text: "Determine root cause".to_string(),
                    category: "Analysis".to_string(),
                    required: true,
                    completed: false,
                    help: None,
                },
            ],
            metadata_fields: vec![
                MetadataField {
                    name: "incident_id".to_string(),
                    field_type: "text".to_string(),
                    default_value: None,
                    required: true,
                    help: Some("Incident tracking ID".to_string()),
                },
                MetadataField {
                    name: "severity".to_string(),
                    field_type: "select".to_string(),
                    default_value: Some("medium".to_string()),
                    required: true,
                    help: Some("Incident severity level".to_string()),
                },
            ],
            workspace_profile: Some("incident_response".to_string()),
        }
    }

    /// Create malware analysis template
    fn create_malware_template() -> ProjectTemplate {
        ProjectTemplate {
            id: "malware_analysis".to_string(),
            name: "Malware Analysis".to_string(),
            category: TemplateCategory::Malware,
            description: "Template for malware analysis and reverse engineering".to_string(),
            author: "CORE-FFX".to_string(),
            version: "1.0".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            usage_count: 0,
            tags: vec!["malware".to_string(), "analysis".to_string(), "reverse".to_string()],
            bookmarks: vec![
                BookmarkTemplate {
                    name: "Sample Files".to_string(),
                    description: "Malware samples under analysis".to_string(),
                    category: "Samples".to_string(),
                    tags: vec!["samples".to_string(), "malware".to_string()],
                },
                BookmarkTemplate {
                    name: "Dropped Files".to_string(),
                    description: "Files dropped by malware".to_string(),
                    category: "Artifacts".to_string(),
                    tags: vec!["dropped".to_string(), "artifacts".to_string()],
                },
            ],
            notes: vec![
                NoteTemplate {
                    title: "Sample Information".to_string(),
                    content: "**Sample Details:**\n- Filename:\n- File Size:\n- File Type:\n- MD5:\n- SHA-256:\n- First Seen:".to_string(),
                    category: "Sample".to_string(),
                    tags: vec!["sample".to_string()],
                },
                NoteTemplate {
                    title: "Behavioral Analysis".to_string(),
                    content: "**Observed Behaviors:**\n\n- File System Activity:\n- Network Activity:\n- Registry Changes:\n- Process Activity:".to_string(),
                    category: "Analysis".to_string(),
                    tags: vec!["behavior".to_string()],
                },
            ],
            tabs: vec![],
            hash_algorithms: vec!["SHA-256".to_string(), "SHA-1".to_string(), "MD5".to_string()],
            recommended_tools: vec![
                "hex_viewer".to_string(),
                "strings".to_string(),
                "entropy".to_string(),
            ],
            checklist: vec![
                ChecklistItem {
                    id: "static".to_string(),
                    text: "Perform static analysis".to_string(),
                    category: "Analysis".to_string(),
                    required: true,
                    completed: false,
                    help: Some("Analyze without execution".to_string()),
                },
                ChecklistItem {
                    id: "dynamic".to_string(),
                    text: "Perform dynamic analysis".to_string(),
                    category: "Analysis".to_string(),
                    required: false,
                    completed: false,
                    help: Some("Analyze in controlled environment".to_string()),
                },
            ],
            metadata_fields: vec![],
            workspace_profile: Some("analysis".to_string()),
        }
    }

    /// Create e-discovery template
    fn create_ediscovery_template() -> ProjectTemplate {
        ProjectTemplate {
            id: "ediscovery".to_string(),
            name: "E-Discovery".to_string(),
            category: TemplateCategory::EDiscovery,
            description: "Template for electronic discovery and legal investigations".to_string(),
            author: "CORE-FFX".to_string(),
            version: "1.0".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            usage_count: 0,
            tags: vec!["ediscovery".to_string(), "legal".to_string()],
            bookmarks: vec![
                BookmarkTemplate {
                    name: "Relevant Documents".to_string(),
                    description: "Documents relevant to case".to_string(),
                    category: "Evidence".to_string(),
                    tags: vec!["relevant".to_string(), "documents".to_string()],
                },
                BookmarkTemplate {
                    name: "Privileged Materials".to_string(),
                    description: "Attorney-client privileged materials".to_string(),
                    category: "Privilege".to_string(),
                    tags: vec!["privileged".to_string(), "attorney".to_string()],
                },
            ],
            notes: vec![NoteTemplate {
                title: "Case Information".to_string(),
                content: "**Case Details:**\n- Case Number:\n- Court:\n- Parties:\n- Search Terms:"
                    .to_string(),
                category: "Case".to_string(),
                tags: vec!["case".to_string()],
            }],
            tabs: vec![],
            hash_algorithms: vec!["SHA-256".to_string()],
            recommended_tools: vec![
                "search".to_string(),
                "viewer".to_string(),
                "report".to_string(),
            ],
            checklist: vec![
                ChecklistItem {
                    id: "preservation".to_string(),
                    text: "Verify data preservation".to_string(),
                    category: "Initial".to_string(),
                    required: true,
                    completed: false,
                    help: None,
                },
                ChecklistItem {
                    id: "review".to_string(),
                    text: "Complete document review".to_string(),
                    category: "Review".to_string(),
                    required: true,
                    completed: false,
                    help: None,
                },
            ],
            metadata_fields: vec![MetadataField {
                name: "case_number".to_string(),
                field_type: "text".to_string(),
                default_value: None,
                required: true,
                help: None,
            }],
            workspace_profile: Some("review".to_string()),
        }
    }

    /// Get template by ID
    pub fn get_template(&self, id: &str) -> Option<&ProjectTemplate> {
        self.templates.iter().find(|t| t.id == id)
    }

    /// Apply template to project
    pub fn apply_template(
        &self,
        template_id: &str,
        project: &mut FFXProject,
    ) -> Result<(), String> {
        let template = self.get_template(template_id).ok_or("Template not found")?;

        // Apply bookmarks
        for bookmark_template in &template.bookmarks {
            // In real implementation, would create actual bookmark
            // For now, just add to project notes as documentation
            project.notes.push(crate::project::ProjectNote {
                id: uuid::Uuid::new_v4().to_string(),
                target_type: "template_bookmark".to_string(),
                target_path: None,
                title: bookmark_template.name.clone(),
                content: bookmark_template.description.clone(),
                created_by: "template".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
                modified_at: chrono::Utc::now().to_rfc3339(),
                tags: bookmark_template.tags.clone(),
                priority: Some(bookmark_template.category.clone()),
            });
        }

        // Apply notes
        for note_template in &template.notes {
            project.notes.push(crate::project::ProjectNote {
                id: uuid::Uuid::new_v4().to_string(),
                target_type: "template_note".to_string(),
                target_path: None,
                title: note_template.title.clone(),
                content: note_template.content.clone(),
                created_by: "template".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
                modified_at: chrono::Utc::now().to_rfc3339(),
                tags: note_template.tags.clone(),
                priority: Some(note_template.category.clone()),
            });
        }

        // Add template metadata to project
        let metadata = project
            .processed_databases
            .cached_metadata
            .get_or_insert_default();
        metadata.insert("template_id".to_string(), serde_json::json!(template.id));
        metadata.insert(
            "template_name".to_string(),
            serde_json::json!(template.name),
        );
        metadata.insert(
            "template_category".to_string(),
            serde_json::json!(template.category.as_str()),
        );

        Ok(())
    }

    /// List all templates
    pub fn list_templates(&self) -> Vec<TemplateSummary> {
        self.templates
            .iter()
            .map(|t| TemplateSummary {
                id: t.id.clone(),
                name: t.name.clone(),
                category: t.category,
                description: t.description.clone(),
                tags: t.tags.clone(),
                usage_count: t.usage_count,
            })
            .collect()
    }

    /// Get templates by category
    pub fn get_templates_by_category(&self, category: TemplateCategory) -> Vec<&ProjectTemplate> {
        self.templates
            .iter()
            .filter(|t| t.category == category)
            .collect()
    }

    /// Add custom template
    pub fn add_template(&mut self, template: ProjectTemplate) {
        self.templates.push(template);
    }

    /// Create template from existing project
    pub fn create_from_project(
        &mut self,
        project: &FFXProject,
        name: String,
        category: TemplateCategory,
        description: String,
    ) -> Result<String, String> {
        let template_id = format!("custom_{}", uuid::Uuid::new_v4());

        // Extract bookmarks and notes from project
        let bookmarks: Vec<BookmarkTemplate> = project
            .bookmarks
            .iter()
            .map(|b| BookmarkTemplate {
                name: b.name.clone(),
                description: b.notes.clone().unwrap_or_default(),
                category: b.color.clone().unwrap_or_else(|| "default".to_string()),
                tags: b.tags.clone(),
            })
            .collect();

        let notes: Vec<NoteTemplate> = project
            .notes
            .iter()
            .map(|n| NoteTemplate {
                title: n.title.clone(),
                content: n.content.clone(),
                category: n.priority.clone().unwrap_or_else(|| "normal".to_string()),
                tags: n.tags.clone(),
            })
            .collect();

        let template = ProjectTemplate {
            id: template_id.clone(),
            name,
            category,
            description,
            author: "User".to_string(),
            version: "1.0".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            usage_count: 0,
            tags: Vec::new(),
            bookmarks,
            notes,
            tabs: Vec::new(),
            hash_algorithms: vec!["SHA-256".to_string()],
            recommended_tools: Vec::new(),
            checklist: Vec::new(),
            metadata_fields: Vec::new(),
            workspace_profile: None,
        };

        self.templates.push(template);
        Ok(template_id)
    }

    /// Export template to JSON
    pub fn export_template(&self, id: &str) -> Result<String, String> {
        let template = self.get_template(id).ok_or("Template not found")?;
        serde_json::to_string_pretty(template).map_err(|e| e.to_string())
    }

    /// Import template from JSON
    pub fn import_template(&mut self, json: &str) -> Result<String, String> {
        let template: ProjectTemplate = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let id = template.id.clone();
        self.templates.push(template);
        Ok(id)
    }

    /// Delete a template by ID
    pub fn delete_template(&mut self, id: &str) -> Result<(), String> {
        let initial_len = self.templates.len();
        self.templates.retain(|t| t.id != id);

        if self.templates.len() == initial_len {
            Err("Template not found".to_string())
        } else {
            Ok(())
        }
    }
}

/// Template summary for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateSummary {
    pub id: String,
    pub name: String,
    pub category: TemplateCategory,
    pub description: String,
    pub tags: Vec<String>,
    pub usage_count: usize,
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // TemplateCategory
    // =========================================================================

    #[test]
    fn test_template_category_as_str_all_variants() {
        assert_eq!(TemplateCategory::Mobile.as_str(), "Mobile");
        assert_eq!(TemplateCategory::Computer.as_str(), "Computer");
        assert_eq!(TemplateCategory::Network.as_str(), "Network");
        assert_eq!(TemplateCategory::Cloud.as_str(), "Cloud");
        assert_eq!(
            TemplateCategory::IncidentResponse.as_str(),
            "Incident Response"
        );
        assert_eq!(TemplateCategory::Memory.as_str(), "Memory Analysis");
        assert_eq!(TemplateCategory::Malware.as_str(), "Malware Analysis");
        assert_eq!(TemplateCategory::EDiscovery.as_str(), "E-Discovery");
        assert_eq!(TemplateCategory::General.as_str(), "General");
        assert_eq!(TemplateCategory::Custom.as_str(), "Custom");
    }

    #[test]
    fn test_template_category_equality() {
        assert_eq!(TemplateCategory::Mobile, TemplateCategory::Mobile);
        assert_ne!(TemplateCategory::Mobile, TemplateCategory::Computer);
    }

    #[test]
    fn test_template_category_serialization_roundtrip() {
        for cat in [
            TemplateCategory::Mobile,
            TemplateCategory::Computer,
            TemplateCategory::Network,
            TemplateCategory::Cloud,
            TemplateCategory::IncidentResponse,
            TemplateCategory::Memory,
            TemplateCategory::Malware,
            TemplateCategory::EDiscovery,
            TemplateCategory::General,
            TemplateCategory::Custom,
        ] {
            let json = serde_json::to_string(&cat).unwrap();
            let back: TemplateCategory = serde_json::from_str(&json).unwrap();
            assert_eq!(back, cat);
        }
    }

    // =========================================================================
    // TemplateManager creation
    // =========================================================================

    #[test]
    fn test_template_manager_creation() {
        let manager = TemplateManager::new();
        assert_eq!(manager.templates.len(), 5);
    }

    #[test]
    fn test_template_manager_default() {
        let manager = TemplateManager::default();
        assert_eq!(manager.templates.len(), 5);
    }

    #[test]
    fn test_default_template_ids() {
        let manager = TemplateManager::new();
        let ids: Vec<&str> = manager.templates.iter().map(|t| t.id.as_str()).collect();
        assert!(ids.contains(&"mobile_forensics"));
        assert!(ids.contains(&"computer_forensics"));
        assert!(ids.contains(&"incident_response"));
        assert!(ids.contains(&"malware_analysis"));
        assert!(ids.contains(&"ediscovery"));
    }

    #[test]
    fn test_default_template_categories() {
        let manager = TemplateManager::new();
        let cats: Vec<TemplateCategory> = manager.templates.iter().map(|t| t.category).collect();
        assert!(cats.contains(&TemplateCategory::Mobile));
        assert!(cats.contains(&TemplateCategory::Computer));
        assert!(cats.contains(&TemplateCategory::IncidentResponse));
        assert!(cats.contains(&TemplateCategory::Malware));
        assert!(cats.contains(&TemplateCategory::EDiscovery));
    }

    // =========================================================================
    // get_template
    // =========================================================================

    #[test]
    fn test_get_template_existing() {
        let manager = TemplateManager::new();
        let template = manager.get_template("mobile_forensics");
        assert!(template.is_some());
        assert_eq!(template.unwrap().category, TemplateCategory::Mobile);
    }

    #[test]
    fn test_get_template_not_found() {
        let manager = TemplateManager::new();
        assert!(manager.get_template("nonexistent").is_none());
    }

    // =========================================================================
    // get_templates_by_category
    // =========================================================================

    #[test]
    fn test_get_templates_by_category() {
        let manager = TemplateManager::new();
        let mobile = manager.get_templates_by_category(TemplateCategory::Mobile);
        assert_eq!(mobile.len(), 1);
        assert_eq!(mobile[0].id, "mobile_forensics");
    }

    #[test]
    fn test_get_templates_by_category_empty() {
        let manager = TemplateManager::new();
        let cloud = manager.get_templates_by_category(TemplateCategory::Cloud);
        assert!(cloud.is_empty());
    }

    // =========================================================================
    // list_templates
    // =========================================================================

    #[test]
    fn test_list_templates() {
        let manager = TemplateManager::new();
        let summaries = manager.list_templates();
        assert_eq!(summaries.len(), 5);
        // Each summary should have correct fields
        for summary in &summaries {
            assert!(!summary.id.is_empty());
            assert!(!summary.name.is_empty());
            assert!(!summary.description.is_empty());
        }
    }

    // =========================================================================
    // add_template
    // =========================================================================

    #[test]
    fn test_add_template() {
        let mut manager = TemplateManager::new();
        let initial_count = manager.templates.len();

        manager.add_template(ProjectTemplate {
            id: "custom_test".to_string(),
            name: "Custom Test".to_string(),
            category: TemplateCategory::Custom,
            description: "A test template".to_string(),
            author: "Test".to_string(),
            version: "1.0".to_string(),
            created_at: String::new(),
            updated_at: String::new(),
            usage_count: 0,
            tags: vec![],
            bookmarks: vec![],
            notes: vec![],
            tabs: vec![],
            hash_algorithms: vec![],
            recommended_tools: vec![],
            checklist: vec![],
            metadata_fields: vec![],
            workspace_profile: None,
        });

        assert_eq!(manager.templates.len(), initial_count + 1);
        assert!(manager.get_template("custom_test").is_some());
    }

    // =========================================================================
    // delete_template
    // =========================================================================

    #[test]
    fn test_delete_template_existing() {
        let mut manager = TemplateManager::new();
        let initial_count = manager.templates.len();
        let result = manager.delete_template("mobile_forensics");
        assert!(result.is_ok());
        assert_eq!(manager.templates.len(), initial_count - 1);
        assert!(manager.get_template("mobile_forensics").is_none());
    }

    #[test]
    fn test_delete_template_not_found() {
        let mut manager = TemplateManager::new();
        let result = manager.delete_template("nonexistent");
        assert!(result.is_err());
    }

    // =========================================================================
    // export/import template
    // =========================================================================

    #[test]
    fn test_export_template() {
        let manager = TemplateManager::new();
        let json = manager.export_template("mobile_forensics");
        assert!(json.is_ok());
        let json_str = json.unwrap();
        assert!(json_str.contains("mobile_forensics"));
        assert!(json_str.contains("Mobile Device Forensics"));
    }

    #[test]
    fn test_export_template_not_found() {
        let manager = TemplateManager::new();
        assert!(manager.export_template("nonexistent").is_err());
    }

    #[test]
    fn test_import_export_roundtrip() {
        let manager = TemplateManager::new();
        let json = manager.export_template("mobile_forensics").unwrap();

        let mut manager2 = TemplateManager { templates: vec![] };
        let result = manager2.import_template(&json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "mobile_forensics");
        assert_eq!(manager2.templates.len(), 1);
        assert_eq!(manager2.templates[0].name, "Mobile Device Forensics");
    }

    #[test]
    fn test_import_template_invalid_json() {
        let mut manager = TemplateManager::new();
        let result = manager.import_template("not valid json");
        assert!(result.is_err());
    }

    // =========================================================================
    // Template content validation
    // =========================================================================

    #[test]
    fn test_mobile_template_has_bookmarks() {
        let manager = TemplateManager::new();
        let template = manager.get_template("mobile_forensics").unwrap();
        assert!(!template.bookmarks.is_empty());
        // Should have SMS, Call History, etc.
        let names: Vec<&str> = template.bookmarks.iter().map(|b| b.name.as_str()).collect();
        assert!(names.contains(&"SMS/Messages"));
        assert!(names.contains(&"Call History"));
    }

    #[test]
    fn test_mobile_template_has_checklist() {
        let manager = TemplateManager::new();
        let template = manager.get_template("mobile_forensics").unwrap();
        assert!(!template.checklist.is_empty());
        // Should have required items
        assert!(template.checklist.iter().any(|c| c.required));
    }

    #[test]
    fn test_incident_response_template_has_notes() {
        let manager = TemplateManager::new();
        let template = manager.get_template("incident_response").unwrap();
        assert!(!template.notes.is_empty());
    }

    #[test]
    fn test_malware_template_has_hash_algorithms() {
        let manager = TemplateManager::new();
        let template = manager.get_template("malware_analysis").unwrap();
        assert!(template.hash_algorithms.contains(&"SHA-256".to_string()));
        assert!(template.hash_algorithms.contains(&"MD5".to_string()));
    }

    #[test]
    fn test_all_templates_have_author() {
        let manager = TemplateManager::new();
        for template in &manager.templates {
            assert_eq!(template.author, "CORE-FFX");
        }
    }

    #[test]
    fn test_all_templates_have_version() {
        let manager = TemplateManager::new();
        for template in &manager.templates {
            assert_eq!(template.version, "1.0");
        }
    }

    #[test]
    fn test_all_templates_have_workspace_profile() {
        let manager = TemplateManager::new();
        for template in &manager.templates {
            assert!(
                template.workspace_profile.is_some(),
                "Template {} missing workspace_profile",
                template.id
            );
        }
    }
}
