// =============================================================================
// CORE-FFX - Forensic File Explorer
// Email Parser - EML/MBOX parsing for forensic analysis
// =============================================================================

use mail_parser::{MessageParser, MimeHeaders};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use super::error::{DocumentError, DocumentResult};

/// Email address
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAddress {
    pub name: Option<String>,
    pub address: String,
}

/// Email header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailHeader {
    pub name: String,
    pub value: String,
}

/// Email attachment info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAttachment {
    pub filename: Option<String>,
    pub content_type: String,
    pub size: usize,
    pub is_inline: bool,
}

/// Parsed email information (read-only)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailInfo {
    pub path: String,
    pub message_id: Option<String>,
    pub subject: Option<String>,
    pub from: Vec<EmailAddress>,
    pub to: Vec<EmailAddress>,
    pub cc: Vec<EmailAddress>,
    pub bcc: Vec<EmailAddress>,
    pub date: Option<String>,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub attachments: Vec<EmailAttachment>,
    pub headers: Vec<EmailHeader>,
    pub size: u64,
}

/// Maximum email file size (50 MB) to prevent OOM on malformed/huge files
const MAX_EMAIL_SIZE: u64 = 50 * 1024 * 1024;

/// Parse an EML file
pub fn parse_eml(path: impl AsRef<Path>) -> DocumentResult<EmailInfo> {
    let path = path.as_ref();
    let file_size = std::fs::metadata(path)
        .map(|m| m.len())
        .unwrap_or(0);
    if file_size > MAX_EMAIL_SIZE {
        return Err(DocumentError::Parse(format!(
            "Email file too large ({:.1} MB, max 50 MB)",
            file_size as f64 / (1024.0 * 1024.0)
        )));
    }
    let data = fs::read(path)?;
    let size = data.len() as u64;

    let msg = MessageParser::default()
        .parse(&data)
        .ok_or_else(|| DocumentError::Parse("Failed to parse email".to_string()))?;

    // Extract from addresses
    let from = msg
        .from()
        .map(|addr| extract_address(addr))
        .unwrap_or_default();

    // Extract to addresses
    let to = msg
        .to()
        .map(|addr| extract_address(addr))
        .unwrap_or_default();

    // Extract cc addresses
    let cc = msg
        .cc()
        .map(|addr| extract_address(addr))
        .unwrap_or_default();

    // Extract bcc addresses
    let bcc = msg
        .bcc()
        .map(|addr| extract_address(addr))
        .unwrap_or_default();

    // Extract date
    let date = msg.date().map(|d| d.to_rfc3339());

    // Extract bodies
    let body_text = msg.body_text(0).map(|s| s.to_string());
    let body_html = msg.body_html(0).map(|s| s.to_string());

    // Extract attachments
    let attachments: Vec<EmailAttachment> = msg
        .attachments()
        .map(|att| {
            let filename = att.attachment_name().map(|s| s.to_string());
            let content_type = att
                .content_type()
                .map(|ct| ct.c_type.to_string())
                .unwrap_or_else(|| "application/octet-stream".to_string());
            EmailAttachment {
                filename,
                content_type,
                size: att.len(),
                is_inline: att
                    .content_disposition()
                    .map(|d| d.ctype() == "inline")
                    .unwrap_or(false),
            }
        })
        .collect();

    // Extract all headers — use as_text() for human-readable values
    let headers: Vec<EmailHeader> = msg
        .headers()
        .iter()
        .map(|h| EmailHeader {
            name: h.name.as_str().to_string(),
            value: h.value.as_text().unwrap_or_default().to_string(),
        })
        .collect();

    Ok(EmailInfo {
        path: path.to_string_lossy().to_string(),
        message_id: msg.message_id().map(|s| s.to_string()),
        subject: msg.subject().map(|s| s.to_string()),
        from,
        to,
        cc,
        bcc,
        date,
        body_text,
        body_html,
        attachments,
        headers,
        size,
    })
}

fn extract_address(addr: &mail_parser::Address) -> Vec<EmailAddress> {
    match addr {
        mail_parser::Address::List(list) => list
            .iter()
            .map(|a| EmailAddress {
                name: a.name().map(|s| s.to_string()),
                address: a.address().unwrap_or_default().to_string(),
            })
            .collect(),
        mail_parser::Address::Group(groups) => groups
            .iter()
            .flat_map(|g| g.addresses.iter())
            .map(|a| EmailAddress {
                name: a.name().map(|s| s.to_string()),
                address: a.address().unwrap_or_default().to_string(),
            })
            .collect(),
    }
}

/// Parse an MBOX file (returns multiple emails)
pub fn parse_mbox(
    path: impl AsRef<Path>,
    max_messages: Option<usize>,
) -> DocumentResult<Vec<EmailInfo>> {
    let path = path.as_ref();
    let file_size = std::fs::metadata(path)
        .map(|m| m.len())
        .unwrap_or(0);
    if file_size > MAX_EMAIL_SIZE {
        return Err(DocumentError::Parse(format!(
            "MBOX file too large ({:.1} MB, max 50 MB)",
            file_size as f64 / (1024.0 * 1024.0)
        )));
    }
    // Use read + from_utf8_lossy to handle non-UTF-8 bytes in MBOX files
    let raw = fs::read(path)?;
    let data = String::from_utf8_lossy(&raw);
    let max = max_messages.unwrap_or(100);

    // Simple MBOX parsing - split on "From " at line start
    let mut messages = Vec::new();
    let mut current_message = String::new();

    for line in data.lines() {
        if line.starts_with("From ") && !current_message.is_empty() {
            if messages.len() >= max {
                break;
            }
            if let Ok(info) = parse_message_bytes(current_message.as_bytes(), path) {
                messages.push(info);
            }
            current_message.clear();
        }
        current_message.push_str(line);
        current_message.push('\n');
    }

    // Don't forget the last message
    if !current_message.is_empty() && messages.len() < max {
        if let Ok(info) = parse_message_bytes(current_message.as_bytes(), path) {
            messages.push(info);
        }
    }

    Ok(messages)
}

fn parse_message_bytes(data: &[u8], path: &Path) -> DocumentResult<EmailInfo> {
    let msg = MessageParser::default()
        .parse(data)
        .ok_or_else(|| DocumentError::Parse("Failed to parse email".to_string()))?;

    let from = msg
        .from()
        .map(|addr| extract_address(addr))
        .unwrap_or_default();

    let to = msg
        .to()
        .map(|addr| extract_address(addr))
        .unwrap_or_default();

    let cc = msg
        .cc()
        .map(|addr| extract_address(addr))
        .unwrap_or_default();

    let bcc = msg
        .bcc()
        .map(|addr| extract_address(addr))
        .unwrap_or_default();

    // Extract attachments
    let attachments: Vec<EmailAttachment> = msg
        .attachments()
        .map(|att| EmailAttachment {
            filename: att.attachment_name().map(|n| n.to_string()),
            content_type: att
                .content_type()
                .map(|ct| format!("{}/{}", ct.ctype(), ct.subtype().unwrap_or("octet-stream")))
                .unwrap_or_else(|| "application/octet-stream".to_string()),
            size: att.len(),
            is_inline: att
                .content_disposition()
                .map(|d| d.ctype() == "inline")
                .unwrap_or(false),
        })
        .collect();

    // Extract headers
    let headers: Vec<EmailHeader> = msg
        .headers()
        .iter()
        .map(|h| EmailHeader {
            name: h.name.as_str().to_string(),
            value: h.value.as_text().unwrap_or_default().to_string(),
        })
        .collect();

    Ok(EmailInfo {
        path: path.to_string_lossy().to_string(),
        message_id: msg.message_id().map(|s| s.to_string()),
        subject: msg.subject().map(|s| s.to_string()),
        from,
        to,
        cc,
        bcc,
        date: msg.date().map(|d| d.to_rfc3339()),
        body_text: msg.body_text(0).map(|s| s.to_string()),
        body_html: msg.body_html(0).map(|s| s.to_string()),
        attachments,
        headers,
        size: data.len() as u64,
    })
}

// =============================================================================
// MSG (Outlook) Parsing
// =============================================================================

/// Parse an Outlook .msg file (OLE compound document format)
pub fn parse_msg(path: impl AsRef<Path>) -> DocumentResult<EmailInfo> {
    let path = path.as_ref();
    let file_size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);

    let outlook = msg_parser::Outlook::from_path(path)
        .map_err(|e| DocumentError::Parse(format!("Failed to parse MSG file: {:?}", e)))?;

    // Convert sender
    let from = vec![EmailAddress {
        name: if outlook.sender.name.is_empty() {
            None
        } else {
            Some(outlook.sender.name.clone())
        },
        address: outlook.sender.email.clone(),
    }];

    // Convert To recipients
    let to: Vec<EmailAddress> = outlook
        .to
        .iter()
        .map(|p| EmailAddress {
            name: if p.name.is_empty() {
                None
            } else {
                Some(p.name.clone())
            },
            address: p.email.clone(),
        })
        .collect();

    // Convert CC recipients
    let cc: Vec<EmailAddress> = outlook
        .cc
        .iter()
        .map(|p| EmailAddress {
            name: if p.name.is_empty() {
                None
            } else {
                Some(p.name.clone())
            },
            address: p.email.clone(),
        })
        .collect();

    // BCC is a plain string in msg_parser
    let bcc: Vec<EmailAddress> = if outlook.bcc.is_empty() {
        Vec::new()
    } else {
        vec![EmailAddress {
            name: None,
            address: outlook.bcc.clone(),
        }]
    };

    // Body text (MSG format stores plain text in body, RTF in rtf_compressed, no HTML field)
    let body_text = if outlook.body.is_empty() {
        None
    } else {
        Some(outlook.body.clone())
    };
    let body_html: Option<String> = None;

    // Extract message-id from transport headers
    let message_id = if !outlook.headers.message_id.is_empty() {
        Some(outlook.headers.message_id.clone())
    } else {
        None
    };

    // Extract transport headers as EmailHeader entries
    let mut headers = Vec::new();
    let h = &outlook.headers;
    if !h.content_type.is_empty() {
        headers.push(EmailHeader {
            name: "Content-Type".to_string(),
            value: h.content_type.clone(),
        });
    }
    if !h.date.is_empty() {
        headers.push(EmailHeader {
            name: "Date".to_string(),
            value: h.date.clone(),
        });
    }

    // Extract date from transport headers
    let date = if outlook.headers.date.is_empty() {
        None
    } else {
        Some(outlook.headers.date.clone())
    };

    // Convert attachments
    let attachments: Vec<EmailAttachment> = outlook
        .attachments
        .iter()
        .map(|att| EmailAttachment {
            filename: if att.file_name.is_empty() {
                if att.display_name.is_empty() {
                    None
                } else {
                    Some(att.display_name.clone())
                }
            } else {
                Some(att.file_name.clone())
            },
            content_type: if att.mime_tag.is_empty() {
                "application/octet-stream".to_string()
            } else {
                att.mime_tag.clone()
            },
            size: att.payload.len(),
            is_inline: false,
        })
        .collect();

    Ok(EmailInfo {
        path: path.to_string_lossy().to_string(),
        message_id,
        subject: if outlook.subject.is_empty() {
            None
        } else {
            Some(outlook.subject)
        },
        from,
        to,
        cc,
        bcc,
        date,
        body_text,
        body_html,
        attachments,
        headers,
        size: file_size,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_address_struct() {
        let addr = EmailAddress {
            name: Some("John Doe".to_string()),
            address: "john@example.com".to_string(),
        };
        assert_eq!(addr.address, "john@example.com");
    }

    #[test]
    fn test_email_attachment_struct() {
        let att = EmailAttachment {
            filename: Some("report.pdf".to_string()),
            content_type: "application/pdf".to_string(),
            size: 1024,
            is_inline: false,
        };
        assert_eq!(att.filename, Some("report.pdf".to_string()));
        assert_eq!(att.size, 1024);
    }
}
