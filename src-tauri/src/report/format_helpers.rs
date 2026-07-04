// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Shared formatting helpers for professional report output.

/// Narrative content blocks derived from freeform text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextBlock {
    Paragraph(String),
    BulletList(Vec<String>),
}

/// Return true when an optional string contains non-whitespace text.
pub fn has_text(value: Option<&str>) -> bool {
    value.map(|text| !text.trim().is_empty()).unwrap_or(false)
}

/// Normalize freeform narrative text into structured blocks.
pub fn text_blocks(input: &str) -> Vec<TextBlock> {
    let normalized = input.replace("\r\n", "\n").replace('\r', "\n");
    let trimmed = normalized.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    let mut blocks = Vec::new();
    let mut current_lines = Vec::new();

    for line in trimmed.lines() {
        let line = line.trim();
        if line.is_empty() {
            push_block(&mut blocks, &mut current_lines);
            continue;
        }
        current_lines.push(line.to_string());
    }

    push_block(&mut blocks, &mut current_lines);
    blocks
}

/// Normalize freeform text into paragraphs separated by blank lines.
pub fn normalized_paragraph_text(input: &str) -> String {
    let mut parts = Vec::new();

    for block in text_blocks(input) {
        match block {
            TextBlock::Paragraph(paragraph) => parts.push(paragraph),
            TextBlock::BulletList(items) => {
                parts.push(
                    items
                        .into_iter()
                        .map(|item| format!("- {item}"))
                        .collect::<Vec<_>>()
                        .join("\n"),
                );
            }
        }
    }

    parts.join("\n\n")
}

/// Return a stable appendix label for a zero-based index: A..Z, AA..AZ, BA...
pub fn appendix_label(index: usize) -> String {
    let mut value = index;
    let mut chars = Vec::new();

    loop {
        let rem = value % 26;
        chars.push((b'A' + rem as u8) as char);
        if value < 26 {
            break;
        }
        value = (value / 26).saturating_sub(1);
    }

    chars.into_iter().rev().collect()
}

fn push_block(blocks: &mut Vec<TextBlock>, current_lines: &mut Vec<String>) {
    if current_lines.is_empty() {
        return;
    }

    let lines = std::mem::take(current_lines);
    let bullet_items: Vec<String> = lines
        .iter()
        .filter_map(|line| strip_list_prefix(line))
        .collect();

    if bullet_items.len() == lines.len() {
        blocks.push(TextBlock::BulletList(bullet_items));
        return;
    }

    blocks.push(TextBlock::Paragraph(lines.join(" ")));
}

fn strip_list_prefix(line: &str) -> Option<String> {
    for prefix in ["- ", "* ", "• "] {
        if let Some(rest) = line.strip_prefix(prefix) {
            return Some(rest.trim().to_string());
        }
    }

    let bytes = line.as_bytes();
    let mut end = 0;
    while end < bytes.len() && bytes[end].is_ascii_digit() {
        end += 1;
    }

    if end > 0
        && end + 1 < bytes.len()
        && (bytes[end] == b'.' || bytes[end] == b')')
        && bytes[end + 1] == b' '
    {
        return Some(line[end + 2..].trim().to_string());
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_blocks_paragraphs() {
        let blocks = text_blocks("First line\nsecond line\n\nThird block");
        assert_eq!(
            blocks,
            vec![
                TextBlock::Paragraph("First line second line".to_string()),
                TextBlock::Paragraph("Third block".to_string())
            ]
        );
    }

    #[test]
    fn test_text_blocks_bullets() {
        let blocks = text_blocks("- Alpha\n- Beta\n- Gamma");
        assert_eq!(
            blocks,
            vec![TextBlock::BulletList(vec![
                "Alpha".to_string(),
                "Beta".to_string(),
                "Gamma".to_string()
            ])]
        );
    }

    #[test]
    fn appendix_label_handles_more_than_twenty_six_entries() {
        assert_eq!(appendix_label(0), "A");
        assert_eq!(appendix_label(25), "Z");
        assert_eq!(appendix_label(26), "AA");
        assert_eq!(appendix_label(27), "AB");
        assert_eq!(appendix_label(51), "AZ");
        assert_eq!(appendix_label(52), "BA");
        assert_eq!(appendix_label(701), "ZZ");
        assert_eq!(appendix_label(702), "AAA");
    }
}
