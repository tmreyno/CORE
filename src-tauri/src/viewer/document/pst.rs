// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! PST/OST email archive parser using the `outlook-pst` crate.
//!
//! Provides read-only parsing of Outlook PST and OST files, extracting
//! folder hierarchy, message summaries, and full message content.
//!
//! **Important:** `UnicodePstFile` is `!Send` and `!Sync` (uses `Rc` internally).
//! All PST operations must open the file, extract data into owned `Serialize`
//! types, and close within a single synchronous block. Each Tauri command
//! independently opens the PST file via `spawn_blocking`.

use serde::Serialize;
use std::rc::Rc;

use outlook_pst::messaging::folder::UnicodeFolder;
use outlook_pst::messaging::message::UnicodeMessage;
use outlook_pst::messaging::store::UnicodeStore;
use outlook_pst::ltp::prop_context::PropertyValue;
use outlook_pst::UnicodePstFile;

use super::error::{DocumentError, DocumentResult};

// =============================================================================
// MAPI Property IDs
// =============================================================================

/// Subject line
const PID_TAG_SUBJECT: u16 = 0x0037;
/// Sender display name
const PID_TAG_SENDER_NAME: u16 = 0x0C1A;
/// Sender email address
const PID_TAG_SENDER_EMAIL: u16 = 0x0065;
/// Sender SMTP address (alternative)
const PID_TAG_SENDER_SMTP: u16 = 0x5D01;
/// Display To recipients
const PID_TAG_DISPLAY_TO: u16 = 0x0E04;
/// Display CC recipients
const PID_TAG_DISPLAY_CC: u16 = 0x0E03;
/// Display BCC recipients
const PID_TAG_DISPLAY_BCC: u16 = 0x0E02;
/// Plain text body
const PID_TAG_BODY: u16 = 0x1000;
/// HTML body
const PID_TAG_BODY_HTML: u16 = 0x1013;
/// Message delivery time
const PID_TAG_MESSAGE_DELIVERY_TIME: u16 = 0x0E06;
/// Client submit time
const PID_TAG_CLIENT_SUBMIT_TIME: u16 = 0x0039;
/// Message class (IPM.Note, etc.)
const PID_TAG_MESSAGE_CLASS: u16 = 0x001A;
/// Message size
const PID_TAG_MESSAGE_SIZE: u16 = 0x0E08;
/// Has attachments flag
const PID_TAG_HASATTACH: u16 = 0x0E1B;
/// Importance (0=low, 1=normal, 2=high)
const PID_TAG_IMPORTANCE: u16 = 0x0017;
/// Message flags (read/unread, etc.)
const PID_TAG_MESSAGE_FLAGS: u16 = 0x0E07;
/// Conversation topic
const PID_TAG_CONVERSATION_TOPIC: u16 = 0x0070;
/// Display name (for recipients/attachments)
const PID_TAG_DISPLAY_NAME: u16 = 0x3001;
/// Attachment filename
const PID_TAG_ATTACH_FILENAME: u16 = 0x3704;
/// Attachment long filename
const PID_TAG_ATTACH_LONG_FILENAME: u16 = 0x3707;
/// Attachment size
const PID_TAG_ATTACH_SIZE: u16 = 0x0E20;
/// Attachment content type (MIME)
const PID_TAG_ATTACH_MIME_TAG: u16 = 0x370E;

// Property IDs we want to request for message reading
const MSG_PROP_IDS: &[u16] = &[
    PID_TAG_SUBJECT,
    PID_TAG_SENDER_NAME,
    PID_TAG_SENDER_EMAIL,
    PID_TAG_SENDER_SMTP,
    PID_TAG_DISPLAY_TO,
    PID_TAG_DISPLAY_CC,
    PID_TAG_DISPLAY_BCC,
    PID_TAG_BODY,
    PID_TAG_BODY_HTML,
    PID_TAG_MESSAGE_DELIVERY_TIME,
    PID_TAG_CLIENT_SUBMIT_TIME,
    PID_TAG_MESSAGE_CLASS,
    PID_TAG_MESSAGE_SIZE,
    PID_TAG_HASATTACH,
    PID_TAG_IMPORTANCE,
    PID_TAG_MESSAGE_FLAGS,
    PID_TAG_CONVERSATION_TOPIC,
];

// =============================================================================
// Serializable output types
// =============================================================================

/// Top-level PST store information with folder tree
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PstInfo {
    /// File path
    pub path: String,
    /// Store display name
    pub display_name: String,
    /// Folder tree (recursive)
    pub folders: Vec<PstFolderInfo>,
    /// Total folder count
    pub total_folders: usize,
}

/// A single folder in the PST hierarchy
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PstFolderInfo {
    /// Folder display name
    pub name: String,
    /// Node ID (used as identifier for subsequent queries)
    pub node_id: u32,
    /// Number of messages in this folder
    pub content_count: i32,
    /// Number of unread messages
    pub unread_count: i32,
    /// Whether this folder has subfolders
    pub has_sub_folders: bool,
    /// Child folders (recursive)
    pub children: Vec<PstFolderInfo>,
}

/// Lightweight message summary for listing in a folder
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PstMessageSummary {
    /// Node ID (used as identifier for full message fetch)
    pub node_id: u32,
    /// Subject line
    pub subject: Option<String>,
    /// Sender display name
    pub sender_name: Option<String>,
    /// Sender email
    pub sender_email: Option<String>,
    /// Display To field
    pub display_to: Option<String>,
    /// Date as ISO 8601 string
    pub date: Option<String>,
    /// Message size in bytes
    pub size: Option<i32>,
    /// Whether message has attachments
    pub has_attachments: bool,
    /// Importance level (0=low, 1=normal, 2=high)
    pub importance: Option<i32>,
    /// Whether message has been read
    pub is_read: bool,
    /// Message class (IPM.Note, etc.)
    pub message_class: Option<String>,
}

/// Full message detail with body content
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PstMessageDetail {
    /// Node ID
    pub node_id: u32,
    /// Subject line
    pub subject: Option<String>,
    /// Sender display name
    pub sender_name: Option<String>,
    /// Sender email
    pub sender_email: Option<String>,
    /// Display To
    pub display_to: Option<String>,
    /// Display CC
    pub display_cc: Option<String>,
    /// Display BCC
    pub display_bcc: Option<String>,
    /// Date as ISO 8601 string
    pub date: Option<String>,
    /// Plain text body
    pub body_text: Option<String>,
    /// HTML body
    pub body_html: Option<String>,
    /// Message size in bytes
    pub size: Option<i32>,
    /// Whether message has attachments
    pub has_attachments: bool,
    /// Importance level
    pub importance: Option<i32>,
    /// Whether message has been read
    pub is_read: bool,
    /// Message class
    pub message_class: Option<String>,
    /// Conversation topic
    pub conversation_topic: Option<String>,
    /// Attachment metadata
    pub attachments: Vec<PstAttachmentInfo>,
}

/// Attachment metadata (no binary content — forensic read-only)
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PstAttachmentInfo {
    /// Display name or filename
    pub filename: Option<String>,
    /// Size in bytes
    pub size: Option<i32>,
    /// MIME content type
    pub content_type: Option<String>,
}

// =============================================================================
// Helper: extract string from PropertyValue
// =============================================================================

fn prop_string(props: &dyn PropGetter, id: u16) -> Option<String> {
    match props.get_prop(id)? {
        PropertyValue::Unicode(s) => Some(s.to_string()),
        PropertyValue::String8(s) => Some(s.to_string()),
        _ => None,
    }
}

fn prop_i32(props: &dyn PropGetter, id: u16) -> Option<i32> {
    match props.get_prop(id)? {
        PropertyValue::Integer32(v) => Some(*v),
        PropertyValue::Integer16(v) => Some(*v as i32),
        _ => None,
    }
}

fn prop_bool(props: &dyn PropGetter, id: u16) -> Option<bool> {
    match props.get_prop(id)? {
        PropertyValue::Boolean(v) => Some(*v),
        _ => None,
    }
}

fn prop_time_iso(props: &dyn PropGetter, id: u16) -> Option<String> {
    match props.get_prop(id)? {
        PropertyValue::Time(t) => {
            // Time is i64 Windows FILETIME (100-ns intervals since 1601-01-01)
            // Convert to Unix timestamp
            let unix_100ns = t - 116_444_736_000_000_000i64; // diff between 1601 and 1970
            if unix_100ns < 0 {
                return None;
            }
            let secs = unix_100ns / 10_000_000;
            let nanos = ((unix_100ns % 10_000_000) * 100) as u32;
            let dt = chrono::DateTime::from_timestamp(secs, nanos)?;
            Some(dt.to_rfc3339())
        }
        _ => None,
    }
}

fn prop_html_string(props: &dyn PropGetter, id: u16) -> Option<String> {
    match props.get_prop(id)? {
        PropertyValue::Unicode(s) => Some(s.to_string()),
        PropertyValue::String8(s) => Some(s.to_string()),
        PropertyValue::Binary(bytes) => String::from_utf8(bytes.buffer().to_vec()).ok(),
        _ => None,
    }
}

/// Extract a string from a PropertyValue (owned conversion)
fn pv_to_string(val: &PropertyValue) -> Option<String> {
    match val {
        PropertyValue::Unicode(s) => Some(s.to_string()),
        PropertyValue::String8(s) => Some(s.to_string()),
        _ => None,
    }
}

/// Trait abstraction for getting a property by ID
trait PropGetter {
    fn get_prop(&self, id: u16) -> Option<&PropertyValue>;
}

/// Wrapper for MessageProperties
struct MsgProps<'a>(&'a outlook_pst::messaging::message::MessageProperties);

impl<'a> PropGetter for MsgProps<'a> {
    fn get_prop(&self, id: u16) -> Option<&PropertyValue> {
        self.0.get(id)
    }
}

// =============================================================================
// Public API functions
// =============================================================================

/// Open a PST file and list all folders recursively.
///
/// This is the entry point — returns the folder tree for the sidebar.
pub fn pst_list_folders(path: &str) -> DocumentResult<PstInfo> {
    let pst = UnicodePstFile::open(path)
        .map_err(|e| DocumentError::InvalidDocument(format!("Failed to open PST: {}", e)))?;
    let pst_rc = Rc::new(pst);

    let store = UnicodeStore::read(pst_rc)
        .map_err(|e| DocumentError::InvalidDocument(format!("Failed to read PST store: {}", e)))?;

    let store_props = store.properties();
    let display_name = store_props.display_name().unwrap_or_else(|_| "Unknown".to_string());

    // Get the IPM subtree root (where user folders live)
    let root_entry_id = store_props
        .ipm_sub_tree_entry_id()
        .map_err(|e| DocumentError::InvalidDocument(format!("Failed to get root folder: {}", e)))?;

    let root_folder = UnicodeFolder::read(store.clone(), &root_entry_id)
        .map_err(|e| DocumentError::InvalidDocument(format!("Failed to read root folder: {}", e)))?;

    let mut folders = Vec::new();
    let mut total_folders = 0;

    // Walk the hierarchy table to discover subfolders
    walk_folders(&store, &root_folder, &mut folders, &mut total_folders)?;

    Ok(PstInfo {
        path: path.to_string(),
        display_name,
        folders,
        total_folders,
    })
}

/// List message summaries in a specific folder (identified by node_id).
///
/// The node_id comes from `PstFolderInfo.node_id` returned by `pst_list_folders`.
pub fn pst_list_messages(
    path: &str,
    folder_node_id: u32,
    offset: Option<usize>,
    limit: Option<usize>,
) -> DocumentResult<Vec<PstMessageSummary>> {
    let pst = UnicodePstFile::open(path)
        .map_err(|e| DocumentError::InvalidDocument(format!("Failed to open PST: {}", e)))?;
    let pst_rc = Rc::new(pst);

    let store = UnicodeStore::read(pst_rc)
        .map_err(|e| DocumentError::InvalidDocument(format!("Failed to read PST store: {}", e)))?;

    // Create an EntryId from the node_id
    let entry_id = store
        .properties()
        .make_entry_id(folder_node_id.into())
        .map_err(|e| DocumentError::InvalidDocument(format!("Failed to make entry ID: {}", e)))?;

    let folder = UnicodeFolder::read(store.clone(), &entry_id)
        .map_err(|e| DocumentError::InvalidDocument(format!("Failed to read folder: {}", e)))?;

    let contents_table = match folder.contents_table() {
        Some(tc) => tc,
        None => return Ok(Vec::new()), // Empty folder
    };

    let offset = offset.unwrap_or(0);
    let limit = limit.unwrap_or(200);

    let mut messages = Vec::new();
    for row in contents_table.rows_matrix().skip(offset).take(limit) {
        let row_node_id: u32 = row.id().into();

        // Try to read the message to get its properties
        let msg_entry_id = match store.properties().make_entry_id(row_node_id.into()) {
            Ok(eid) => eid,
            Err(_) => continue,
        };

        let msg = match UnicodeMessage::read(store.clone(), &msg_entry_id, Some(MSG_PROP_IDS)) {
            Ok(m) => m,
            Err(_) => continue,
        };

        let props = MsgProps(msg.properties());

        let flags = prop_i32(&props, PID_TAG_MESSAGE_FLAGS).unwrap_or(0);
        let is_read = (flags & 0x0001) != 0; // MSGFLAG_READ

        let date = prop_time_iso(&props, PID_TAG_MESSAGE_DELIVERY_TIME)
            .or_else(|| prop_time_iso(&props, PID_TAG_CLIENT_SUBMIT_TIME));

        let sender_email = prop_string(&props, PID_TAG_SENDER_EMAIL)
            .or_else(|| prop_string(&props, PID_TAG_SENDER_SMTP));

        messages.push(PstMessageSummary {
            node_id: row_node_id,
            subject: prop_string(&props, PID_TAG_SUBJECT),
            sender_name: prop_string(&props, PID_TAG_SENDER_NAME),
            sender_email,
            display_to: prop_string(&props, PID_TAG_DISPLAY_TO),
            date,
            size: prop_i32(&props, PID_TAG_MESSAGE_SIZE),
            has_attachments: prop_bool(&props, PID_TAG_HASATTACH).unwrap_or(false),
            importance: prop_i32(&props, PID_TAG_IMPORTANCE),
            is_read,
            message_class: prop_string(&props, PID_TAG_MESSAGE_CLASS),
        });
    }

    Ok(messages)
}

/// Get full message detail including body and attachment metadata.
///
/// The node_id comes from `PstMessageSummary.node_id`.
pub fn pst_get_message(path: &str, message_node_id: u32) -> DocumentResult<PstMessageDetail> {
    let pst = UnicodePstFile::open(path)
        .map_err(|e| DocumentError::InvalidDocument(format!("Failed to open PST: {}", e)))?;
    let pst_rc = Rc::new(pst);

    let store = UnicodeStore::read(pst_rc)
        .map_err(|e| DocumentError::InvalidDocument(format!("Failed to read PST store: {}", e)))?;

    let entry_id = store
        .properties()
        .make_entry_id(message_node_id.into())
        .map_err(|e| DocumentError::InvalidDocument(format!("Failed to make entry ID: {}", e)))?;

    let msg = UnicodeMessage::read(store.clone(), &entry_id, Some(MSG_PROP_IDS))
        .map_err(|e| DocumentError::InvalidDocument(format!("Failed to read message: {}", e)))?;

    let props = MsgProps(msg.properties());

    let flags = prop_i32(&props, PID_TAG_MESSAGE_FLAGS).unwrap_or(0);
    let is_read = (flags & 0x0001) != 0;

    let date = prop_time_iso(&props, PID_TAG_MESSAGE_DELIVERY_TIME)
        .or_else(|| prop_time_iso(&props, PID_TAG_CLIENT_SUBMIT_TIME));

    let sender_email = prop_string(&props, PID_TAG_SENDER_EMAIL)
        .or_else(|| prop_string(&props, PID_TAG_SENDER_SMTP));

    // Extract attachment metadata from the attachment table
    // We iterate rows and try to read properties via store.read_table_column
    let mut attachments = Vec::new();
    if let Some(att_table) = msg.attachment_table() {
        let tc_info = att_table.context();
        let col_descs = tc_info.columns();
        
        for row in att_table.rows_matrix() {
            let cols = match row.columns(tc_info) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let mut filename: Option<String> = None;
            let mut size: Option<i32> = None;
            let mut content_type: Option<String> = None;

            // Read column values using the table context column descriptors
            for (i, col_desc) in col_descs.iter().enumerate() {
                if let Some(Some(col_val)) = cols.get(i) {
                    let prop_val = match store.read_table_column(att_table, col_val, col_desc.prop_type()) {
                        Ok(v) => v,
                        Err(_) => continue,
                    };
                    let prop_id = col_desc.prop_id();
                    if prop_id == PID_TAG_ATTACH_LONG_FILENAME || prop_id == PID_TAG_ATTACH_FILENAME {
                        if filename.is_none() {
                            filename = pv_to_string(&prop_val);
                        }
                    } else if prop_id == PID_TAG_DISPLAY_NAME {
                        if filename.is_none() {
                            filename = pv_to_string(&prop_val);
                        }
                    } else if prop_id == PID_TAG_ATTACH_SIZE {
                        size = match &prop_val {
                            PropertyValue::Integer32(v) => Some(*v),
                            _ => None,
                        };
                    } else if prop_id == PID_TAG_ATTACH_MIME_TAG {
                        content_type = pv_to_string(&prop_val);
                    }
                }
            }

            attachments.push(PstAttachmentInfo {
                filename,
                size,
                content_type,
            });
        }
    }

    Ok(PstMessageDetail {
        node_id: message_node_id,
        subject: prop_string(&props, PID_TAG_SUBJECT),
        sender_name: prop_string(&props, PID_TAG_SENDER_NAME),
        sender_email,
        display_to: prop_string(&props, PID_TAG_DISPLAY_TO),
        display_cc: prop_string(&props, PID_TAG_DISPLAY_CC),
        display_bcc: prop_string(&props, PID_TAG_DISPLAY_BCC),
        date,
        body_text: prop_string(&props, PID_TAG_BODY),
        body_html: prop_html_string(&props, PID_TAG_BODY_HTML),
        size: prop_i32(&props, PID_TAG_MESSAGE_SIZE),
        has_attachments: prop_bool(&props, PID_TAG_HASATTACH).unwrap_or(false),
        importance: prop_i32(&props, PID_TAG_IMPORTANCE),
        is_read,
        message_class: prop_string(&props, PID_TAG_MESSAGE_CLASS),
        conversation_topic: prop_string(&props, PID_TAG_CONVERSATION_TOPIC),
        attachments,
    })
}

// =============================================================================
// Internal helpers
// =============================================================================

/// Recursively walk the folder hierarchy table and collect folder info.
fn walk_folders(
    store: &Rc<UnicodeStore>,
    folder: &Rc<UnicodeFolder>,
    out: &mut Vec<PstFolderInfo>,
    total: &mut usize,
) -> DocumentResult<()> {
    let hierarchy_table = match folder.hierarchy_table() {
        Some(ht) => ht,
        None => return Ok(()), // No subfolders
    };

    let _context = hierarchy_table.context();

    for row in hierarchy_table.rows_matrix() {
        let row_node_id: u32 = row.id().into();

        // Try to open this subfolder
        let sub_entry_id = match store.properties().make_entry_id(row_node_id.into()) {
            Ok(eid) => eid,
            Err(_) => continue,
        };

        let sub_folder = match UnicodeFolder::read(store.clone(), &sub_entry_id) {
            Ok(f) => f,
            Err(_) => continue,
        };

        let sub_props = sub_folder.properties();
        let name = sub_props.display_name().unwrap_or_else(|_| format!("Folder {}", row_node_id));
        let content_count = sub_props.content_count().unwrap_or(0);
        let unread_count = sub_props.unread_count().unwrap_or(0);
        let has_sub_folders = sub_props.has_sub_folders().unwrap_or(false);

        *total += 1;

        let mut children = Vec::new();
        if has_sub_folders {
            walk_folders(store, &sub_folder, &mut children, total)?;
        }

        out.push(PstFolderInfo {
            name,
            node_id: row_node_id,
            content_count,
            unread_count,
            has_sub_folders,
            children,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_filetime_conversion() {
        // Known FILETIME for 2024-01-15T12:00:00Z
        // Just test that the conversion doesn't panic with valid values
        let filetime: i64 = 133_497_264_000_000_000; // approx 2024-01-15
        let unix_100ns = filetime - 116_444_736_000_000_000i64;
        assert!(unix_100ns > 0);
        let secs = unix_100ns / 10_000_000;
        assert!(secs > 0);
    }

    #[test]
    fn test_prop_extractors_with_none() {
        // Ensure prop functions handle missing properties gracefully
        struct EmptyProps;
        impl PropGetter for EmptyProps {
            fn get_prop(&self, _id: u16) -> Option<&PropertyValue> {
                None
            }
        }
        let empty = EmptyProps;
        assert!(prop_string(&empty, 0x0037).is_none());
        assert!(prop_i32(&empty, 0x0E08).is_none());
        assert!(prop_bool(&empty, 0x0E1B).is_none());
        assert!(prop_time_iso(&empty, 0x0E06).is_none());
    }
}
