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
const MAX_EMAIL_MESSAGES: usize = 1_000;
const MAX_EMAIL_ADDRESSES: usize = 1_024;
const MAX_EMAIL_HEADERS: usize = 512;
const MAX_EMAIL_ATTACHMENTS: usize = 1_024;
const MAX_EMAIL_FIELD_CHARS: usize = 4_096;
const MAX_EMAIL_BODY_CHARS: usize = 1_048_576;

fn ensure_email_size_allowed(size: u64, kind: &str) -> DocumentResult<()> {
    if size > MAX_EMAIL_SIZE {
        return Err(DocumentError::Parse(format!(
            "{} file too large ({:.1} MB, max 50 MB)",
            kind,
            size as f64 / (1024.0 * 1024.0)
        )));
    }
    Ok(())
}

fn normalized_mbox_message_limit(max_messages: Option<usize>) -> usize {
    max_messages.unwrap_or(100).clamp(1, MAX_EMAIL_MESSAGES)
}

fn truncate_email_text(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }

    let mut truncated: String = value.chars().take(max_chars).collect();
    truncated.push_str("...");
    truncated
}

/// Parse an EML file
pub fn parse_eml(path: impl AsRef<Path>) -> DocumentResult<EmailInfo> {
    let path = path.as_ref();
    let file_size = fs::metadata(path)?.len();
    ensure_email_size_allowed(file_size, "Email")?;
    let data = fs::read(path)?;
    parse_eml_bytes(path.to_string_lossy(), &data)
}

/// Parse EML bytes from any evidence source.
pub fn parse_eml_bytes(source_id: impl Into<String>, data: &[u8]) -> DocumentResult<EmailInfo> {
    ensure_email_size_allowed(data.len() as u64, "Email")?;
    parse_message_bytes(data, &source_id.into())
}

fn extract_address(addr: &mail_parser::Address) -> Vec<EmailAddress> {
    match addr {
        mail_parser::Address::List(list) => list
            .iter()
            .take(MAX_EMAIL_ADDRESSES)
            .map(|a| EmailAddress {
                name: a
                    .name()
                    .map(|s| truncate_email_text(s, MAX_EMAIL_FIELD_CHARS)),
                address: truncate_email_text(
                    a.address().unwrap_or_default(),
                    MAX_EMAIL_FIELD_CHARS,
                ),
            })
            .collect(),
        mail_parser::Address::Group(groups) => groups
            .iter()
            .flat_map(|g| g.addresses.iter())
            .take(MAX_EMAIL_ADDRESSES)
            .map(|a| EmailAddress {
                name: a
                    .name()
                    .map(|s| truncate_email_text(s, MAX_EMAIL_FIELD_CHARS)),
                address: truncate_email_text(
                    a.address().unwrap_or_default(),
                    MAX_EMAIL_FIELD_CHARS,
                ),
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
    let file_size = fs::metadata(path)?.len();
    ensure_email_size_allowed(file_size, "MBOX")?;
    // Use read + from_utf8_lossy to handle non-UTF-8 bytes in MBOX files
    let raw = fs::read(path)?;
    parse_mbox_bytes(path.to_string_lossy(), &raw, max_messages)
}

/// Parse MBOX bytes from any evidence source.
pub fn parse_mbox_bytes(
    source_id: impl Into<String>,
    raw: &[u8],
    max_messages: Option<usize>,
) -> DocumentResult<Vec<EmailInfo>> {
    ensure_email_size_allowed(raw.len() as u64, "MBOX")?;
    let source_id = source_id.into();
    let data = String::from_utf8_lossy(raw);
    let max = normalized_mbox_message_limit(max_messages);

    // Simple MBOX parsing - split on "From " at line start
    let mut messages = Vec::new();
    let mut current_message = String::new();
    let mut next_message_index = 1usize;
    let mut seen_mbox_separator = false;

    for line in data.lines() {
        if line.starts_with("From ") {
            if !current_message.is_empty() {
                if messages.len() >= max {
                    break;
                }
                let info =
                    parse_message_bytes(current_message.as_bytes(), &source_id).map_err(|e| {
                        DocumentError::Parse(format!(
                            "Failed to parse MBOX message {}: {}",
                            next_message_index, e
                        ))
                    })?;
                messages.push(info);
                next_message_index += 1;
                current_message.clear();
            }
            seen_mbox_separator = true;
            continue;
        }

        if !seen_mbox_separator && current_message.is_empty() && line.trim().is_empty() {
            continue;
        }

        current_message.push_str(line);
        current_message.push('\n');
    }

    // Don't forget the last message
    if !current_message.is_empty() && messages.len() < max {
        let info = parse_message_bytes(current_message.as_bytes(), &source_id).map_err(|e| {
            DocumentError::Parse(format!(
                "Failed to parse MBOX message {}: {}",
                next_message_index, e
            ))
        })?;
        messages.push(info);
    }

    Ok(messages)
}

fn parse_message_bytes(data: &[u8], source_id: &str) -> DocumentResult<EmailInfo> {
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
        .take(MAX_EMAIL_ATTACHMENTS)
        .map(|att| EmailAttachment {
            filename: att
                .attachment_name()
                .map(|n| truncate_email_text(n, MAX_EMAIL_FIELD_CHARS)),
            content_type: att
                .content_type()
                .map(|ct| format!("{}/{}", ct.ctype(), ct.subtype().unwrap_or("octet-stream")))
                .map(|s| truncate_email_text(&s, MAX_EMAIL_FIELD_CHARS))
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
        .take(MAX_EMAIL_HEADERS)
        .map(|h| EmailHeader {
            name: truncate_email_text(h.name.as_str(), MAX_EMAIL_FIELD_CHARS),
            value: truncate_email_text(
                h.value.as_text().unwrap_or_default(),
                MAX_EMAIL_FIELD_CHARS,
            ),
        })
        .collect();

    Ok(EmailInfo {
        path: source_id.to_string(),
        message_id: msg
            .message_id()
            .map(|s| truncate_email_text(s, MAX_EMAIL_FIELD_CHARS)),
        subject: msg
            .subject()
            .map(|s| truncate_email_text(s, MAX_EMAIL_FIELD_CHARS)),
        from,
        to,
        cc,
        bcc,
        date: msg.date().map(|d| d.to_rfc3339()),
        body_text: msg
            .body_text(0)
            .map(|s| truncate_email_text(s.as_ref(), MAX_EMAIL_BODY_CHARS)),
        body_html: msg
            .body_html(0)
            .map(|s| truncate_email_text(s.as_ref(), MAX_EMAIL_BODY_CHARS)),
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
    let file_size = fs::metadata(path)?.len();
    ensure_email_size_allowed(file_size, "MSG")?;

    let outlook = msg_parser::Outlook::from_path(path)
        .map_err(|e| DocumentError::Parse(format!("Failed to parse MSG file: {:?}", e)))?;

    // Convert sender
    let from = vec![EmailAddress {
        name: if outlook.sender.name.is_empty() {
            None
        } else {
            Some(truncate_email_text(
                &outlook.sender.name,
                MAX_EMAIL_FIELD_CHARS,
            ))
        },
        address: truncate_email_text(&outlook.sender.email, MAX_EMAIL_FIELD_CHARS),
    }];

    // Convert To recipients
    let to: Vec<EmailAddress> = outlook
        .to
        .iter()
        .take(MAX_EMAIL_ADDRESSES)
        .map(|p| EmailAddress {
            name: if p.name.is_empty() {
                None
            } else {
                Some(truncate_email_text(&p.name, MAX_EMAIL_FIELD_CHARS))
            },
            address: truncate_email_text(&p.email, MAX_EMAIL_FIELD_CHARS),
        })
        .collect();

    // Convert CC recipients
    let cc: Vec<EmailAddress> = outlook
        .cc
        .iter()
        .take(MAX_EMAIL_ADDRESSES)
        .map(|p| EmailAddress {
            name: if p.name.is_empty() {
                None
            } else {
                Some(truncate_email_text(&p.name, MAX_EMAIL_FIELD_CHARS))
            },
            address: truncate_email_text(&p.email, MAX_EMAIL_FIELD_CHARS),
        })
        .collect();

    // BCC is a plain string in msg_parser
    let bcc: Vec<EmailAddress> = if outlook.bcc.is_empty() {
        Vec::new()
    } else {
        vec![EmailAddress {
            name: None,
            address: truncate_email_text(&outlook.bcc, MAX_EMAIL_FIELD_CHARS),
        }]
    };

    // Body text (MSG format stores plain text in body, RTF in rtf_compressed, no HTML field)
    let body_text = if outlook.body.is_empty() {
        None
    } else {
        Some(truncate_email_text(&outlook.body, MAX_EMAIL_BODY_CHARS))
    };
    let body_html: Option<String> = None;

    // Extract message-id from transport headers
    let message_id = if !outlook.headers.message_id.is_empty() {
        Some(truncate_email_text(
            &outlook.headers.message_id,
            MAX_EMAIL_FIELD_CHARS,
        ))
    } else {
        None
    };

    // Extract transport headers as EmailHeader entries
    let mut headers = Vec::new();
    let h = &outlook.headers;
    if !h.content_type.is_empty() {
        headers.push(EmailHeader {
            name: "Content-Type".to_string(),
            value: truncate_email_text(&h.content_type, MAX_EMAIL_FIELD_CHARS),
        });
    }
    if !h.date.is_empty() {
        headers.push(EmailHeader {
            name: "Date".to_string(),
            value: truncate_email_text(&h.date, MAX_EMAIL_FIELD_CHARS),
        });
    }

    // Extract date from transport headers
    let date = if outlook.headers.date.is_empty() {
        None
    } else {
        Some(truncate_email_text(
            &outlook.headers.date,
            MAX_EMAIL_FIELD_CHARS,
        ))
    };

    // Convert attachments
    let attachments: Vec<EmailAttachment> = outlook
        .attachments
        .iter()
        .take(MAX_EMAIL_ATTACHMENTS)
        .map(|att| EmailAttachment {
            filename: if att.file_name.is_empty() {
                if att.display_name.is_empty() {
                    None
                } else {
                    Some(truncate_email_text(
                        &att.display_name,
                        MAX_EMAIL_FIELD_CHARS,
                    ))
                }
            } else {
                Some(truncate_email_text(&att.file_name, MAX_EMAIL_FIELD_CHARS))
            },
            content_type: if att.mime_tag.is_empty() {
                "application/octet-stream".to_string()
            } else {
                truncate_email_text(&att.mime_tag, MAX_EMAIL_FIELD_CHARS)
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
            Some(truncate_email_text(&outlook.subject, MAX_EMAIL_FIELD_CHARS))
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

    #[test]
    fn normalized_mbox_message_limit_clamps_bounds() {
        assert_eq!(normalized_mbox_message_limit(None), 100);
        assert_eq!(normalized_mbox_message_limit(Some(0)), 1);
        assert_eq!(
            normalized_mbox_message_limit(Some(usize::MAX)),
            MAX_EMAIL_MESSAGES
        );
    }

    #[test]
    fn truncate_email_text_is_unicode_safe() {
        let value = "é".repeat(MAX_EMAIL_FIELD_CHARS + 1);

        let truncated = truncate_email_text(&value, MAX_EMAIL_FIELD_CHARS);

        assert!(truncated.ends_with("..."));
        assert_eq!(
            truncated.trim_end_matches("...").chars().count(),
            MAX_EMAIL_FIELD_CHARS
        );
    }

    #[test]
    fn parse_eml_bytes_reads_source_metadata() {
        let data = b"Message-ID: <one@example.com>\r\nSubject: Source Email\r\nFrom: Alice <alice@example.com>\r\nTo: Bob <bob@example.com>\r\n\r\nHello from source.\r\n";

        let info = parse_eml_bytes("container.ad1:mail/message.eml", data).unwrap();

        assert_eq!(info.path, "container.ad1:mail/message.eml");
        assert_eq!(info.message_id.as_deref(), Some("one@example.com"));
        assert_eq!(info.subject.as_deref(), Some("Source Email"));
        assert_eq!(info.from[0].address, "alice@example.com");
        assert_eq!(info.to[0].address, "bob@example.com");
        assert!(info
            .body_text
            .as_deref()
            .is_some_and(|body| body.contains("Hello from source.")));
    }

    #[test]
    fn parse_eml_bytes_truncates_long_body_text() {
        let body = "é".repeat(MAX_EMAIL_BODY_CHARS + 1);
        let data = format!(
            "Subject: Long Body\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n{body}\r\n"
        );

        let info = parse_eml_bytes("container.ad1:mail/long.eml", data.as_bytes()).unwrap();
        let body_text = info.body_text.as_deref().unwrap();

        assert!(body_text.ends_with("..."));
        assert_eq!(
            body_text.trim_end_matches("...").chars().count(),
            MAX_EMAIL_BODY_CHARS
        );
    }

    #[test]
    fn parse_mbox_bytes_reads_multiple_messages_from_source() {
        let data = b"From alice@example.com Sat Jan 01 00:00:00 2024\nSubject: First\nFrom: Alice <alice@example.com>\nTo: Bob <bob@example.com>\n\nFirst body\nFrom carol@example.com Sat Jan 01 00:00:01 2024\nSubject: Second\nFrom: Carol <carol@example.com>\nTo: Dave <dave@example.com>\n\nSecond body\n";

        let messages = parse_mbox_bytes("container.ad1:mail/archive.mbox", data, Some(10)).unwrap();

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].path, "container.ad1:mail/archive.mbox");
        assert_eq!(messages[0].subject.as_deref(), Some("First"));
        assert_eq!(messages[1].subject.as_deref(), Some("Second"));
    }

    #[test]
    fn parse_mbox_bytes_clamps_requested_message_count() {
        let mut data = String::new();
        for index in 0..(MAX_EMAIL_MESSAGES + 8) {
            data.push_str(&format!(
                "From sender{index}@example.com Sat Jan 01 00:00:00 2024\nSubject: Message {index}\nFrom: Sender <sender{index}@example.com>\nTo: Receiver <receiver@example.com>\n\nBody {index}\n"
            ));
        }

        let messages = parse_mbox_bytes(
            "container.ad1:mail/large.mbox",
            data.as_bytes(),
            Some(usize::MAX),
        )
        .unwrap();

        assert_eq!(messages.len(), MAX_EMAIL_MESSAGES);
        assert_eq!(
            messages
                .last()
                .and_then(|message| message.subject.as_deref()),
            Some("Message 999")
        );
    }

    #[test]
    fn parse_mbox_bytes_rejects_unparseable_message() {
        let data =
            b"From broken@example.com Sat Jan 01 00:00:00 2024\nnot an RFC5322 message body only\n";

        let err = parse_mbox_bytes("container.ad1:mail/broken.mbox", data, Some(10)).unwrap_err();

        assert!(err.to_string().contains("Failed to parse MBOX message 1"));
    }

    #[test]
    fn parse_eml_reports_missing_file_metadata_error() {
        let missing = tempfile::tempdir().unwrap().path().join("missing.eml");

        let err = parse_eml(&missing).unwrap_err();

        assert!(matches!(err, DocumentError::Io(_)));
    }

    #[test]
    fn parse_mbox_reports_missing_file_metadata_error() {
        let missing = tempfile::tempdir().unwrap().path().join("missing.mbox");

        let err = parse_mbox(&missing, Some(10)).unwrap_err();

        assert!(matches!(err, DocumentError::Io(_)));
    }

    #[test]
    fn parse_msg_reports_missing_file_metadata_error() {
        let missing = tempfile::tempdir().unwrap().path().join("missing.msg");

        let err = parse_msg(&missing).unwrap_err();

        assert!(matches!(err, DocumentError::Io(_)));
    }

    #[test]
    fn parse_msg_rejects_oversized_file_before_parsing() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.as_file().set_len(MAX_EMAIL_SIZE + 1).unwrap();

        let err = parse_msg(tmp.path()).unwrap_err();

        assert!(err.to_string().contains("MSG file too large"));
    }
}
