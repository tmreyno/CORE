// =============================================================================
// CORE-FFX - Forensic File Explorer
// Email Parser - EML/MBOX parsing for forensic analysis
// =============================================================================

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use mail_parser::{MessageParser, MimeHeaders};

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

/// Parse an EML file
pub fn parse_eml(path: impl AsRef<Path>) -> DocumentResult<EmailInfo> {
    let path = path.as_ref();
    let data = fs::read(path)?;
    let size = data.len() as u64;
    
    let msg = MessageParser::default()
        .parse(&data)
        .ok_or_else(|| DocumentError::Parse("Failed to parse email".to_string()))?;
    
    // Extract from addresses
    let from = msg.from()
        .map(|addr| extract_address(addr))
        .unwrap_or_default();
    
    // Extract to addresses
    let to = msg.to()
        .map(|addr| extract_address(addr))
        .unwrap_or_default();
    
    // Extract cc addresses
    let cc = msg.cc()
        .map(|addr| extract_address(addr))
        .unwrap_or_default();
    
    // Extract bcc addresses
    let bcc = msg.bcc()
        .map(|addr| extract_address(addr))
        .unwrap_or_default();
    
    // Extract date
    let date = msg.date().map(|d| d.to_rfc3339());
    
    // Extract bodies
    let body_text = msg.body_text(0).map(|s| s.to_string());
    let body_html = msg.body_html(0).map(|s| s.to_string());
    
    // Extract attachments
    let attachments: Vec<EmailAttachment> = msg.attachments()
        .map(|att| {
            let filename = att.attachment_name().map(|s| s.to_string());
            let content_type = att.content_type()
                .map(|ct| ct.c_type.to_string())
                .unwrap_or_else(|| "application/octet-stream".to_string());
            EmailAttachment {
                filename,
                content_type,
                size: att.len(),
                is_inline: att.is_message(),
            }
        })
        .collect();
    
    // Extract all headers
    let headers: Vec<EmailHeader> = msg.headers()
        .iter()
        .map(|h| EmailHeader {
            name: h.name().to_string(),
            value: format!("{:?}", h.value()),
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
        mail_parser::Address::List(list) => {
            list.iter()
                .map(|a| EmailAddress {
                    name: a.name().map(|s| s.to_string()),
                    address: a.address().unwrap_or_default().to_string(),
                })
                .collect()
        }
        mail_parser::Address::Group(groups) => {
            groups.iter()
                .flat_map(|g| g.addresses.iter())
                .map(|a| EmailAddress {
                    name: a.name().map(|s| s.to_string()),
                    address: a.address().unwrap_or_default().to_string(),
                })
                .collect()
        }
    }
}

/// Parse an MBOX file (returns multiple emails)
pub fn parse_mbox(path: impl AsRef<Path>, max_messages: Option<usize>) -> DocumentResult<Vec<EmailInfo>> {
    let path = path.as_ref();
    let data = fs::read_to_string(path)?;
    let max = max_messages.unwrap_or(100);
    
    // Simple MBOX parsing - split on "From " at line start
    let mut messages = Vec::new();
    let mut current_message = String::new();
    
    for line in data.lines() {
        if line.starts_with("From ") && !current_message.is_empty() {
            if messages.len() >= max { break; }
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
    
    let from = msg.from()
        .map(|addr| extract_address(addr))
        .unwrap_or_default();
    
    let to = msg.to()
        .map(|addr| extract_address(addr))
        .unwrap_or_default();
    
    Ok(EmailInfo {
        path: path.to_string_lossy().to_string(),
        message_id: msg.message_id().map(|s| s.to_string()),
        subject: msg.subject().map(|s| s.to_string()),
        from,
        to,
        cc: Vec::new(),
        bcc: Vec::new(),
        date: msg.date().map(|d| d.to_rfc3339()),
        body_text: msg.body_text(0).map(|s| s.to_string()),
        body_html: msg.body_html(0).map(|s| s.to_string()),
        attachments: Vec::new(),
        headers: Vec::new(),
        size: data.len() as u64,
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
}
