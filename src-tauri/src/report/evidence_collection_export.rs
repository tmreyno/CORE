// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Multi-format evidence collection export
//!
//! Exports Evidence Collection form data as:
//!   - CSV  (csv crate)
//!   - XLSX (rust_xlsxwriter crate)
//!   - HTML (self-contained with inline CSS)
//!
//! Used as a complement to the PDF form renderer in `pdf_evidence_collection.rs`.

use std::path::Path;

use super::error::{ReportError, ReportResult};
use super::types::{CollectedItem, EvidenceCollectionData};

// =============================================================================
// Column definitions — shared across CSV & XLSX
// =============================================================================

const HEADERS: &[&str] = &[
    "Item #",
    "Collection Date/Time",
    "System Date/Time",
    "Collecting Officer",
    "Authorization",
    "Device Type",
    "Brand / Manufacturer",
    "Make",
    "Model",
    "Color",
    "Serial Number",
    "IMEI",
    "Other Identifiers",
    "Building",
    "Room",
    "Sub-Location",
    "Found Location",
    "Image Format",
    "Acquisition Method",
    "Condition",
    "Packaging",
    "Storage Notes",
    "Notes",
    "Photo Refs",
    "Evidence File ID",
    "Source ID",
    "Hash Algorithm",
    "Hash Value",
    "Hash Computed At",
    "Description",
];

const MAX_EXPORT_FIELD_CHARS: usize = 32_000;
const TRUNCATED_FIELD_SUFFIX: &str = "... [truncated]";

// =============================================================================
// CSV Export
// =============================================================================

/// Export evidence collection data to CSV
pub fn export_csv(ev: &EvidenceCollectionData, output_path: impl AsRef<Path>) -> ReportResult<()> {
    let file = std::fs::File::create(output_path)?;
    let mut wtr = csv::Writer::from_writer(file);

    wtr.write_record(HEADERS).map_err(csv_err)?;

    for item in &ev.collected_items {
        wtr.write_record(item_to_spreadsheet_row(item))
            .map_err(csv_err)?;
    }

    wtr.flush()?;
    Ok(())
}

// =============================================================================
// XLSX Export
// =============================================================================

/// Export evidence collection data to XLSX (Excel)
pub fn export_xlsx(
    ev: &EvidenceCollectionData,
    case_number: &str,
    output_path: impl AsRef<Path>,
) -> ReportResult<()> {
    use rust_xlsxwriter::{Color, Format, FormatAlign, FormatBorder, Workbook};

    let mut wb = Workbook::new();
    let ws = wb.add_worksheet();
    ws.set_name("Evidence Collection").map_err(xlsx_err)?;

    // ---- Formats ----
    let title_fmt = Format::new()
        .set_bold()
        .set_font_size(14)
        .set_font_color(Color::White)
        .set_background_color(Color::RGB(0x1F3864))
        .set_align(FormatAlign::Center);

    let header_fmt = Format::new()
        .set_bold()
        .set_font_size(10)
        .set_font_color(Color::White)
        .set_background_color(Color::RGB(0x2E75B6))
        .set_border(FormatBorder::Thin)
        .set_text_wrap()
        .set_align(FormatAlign::Center);

    let cell_fmt = Format::new()
        .set_font_size(10)
        .set_border(FormatBorder::Thin)
        .set_text_wrap();

    let meta_label = Format::new()
        .set_bold()
        .set_font_size(10)
        .set_background_color(Color::RGB(0xD9E2F3));

    let meta_val = Format::new().set_font_size(10);

    // ---- Title row ----
    ws.merge_range(
        0,
        0,
        0,
        (HEADERS.len() - 1) as u16,
        "EVIDENCE COLLECTION FORM",
        &title_fmt,
    )
    .map_err(xlsx_err)?;
    ws.set_row_height(0, 28).map_err(xlsx_err)?;

    // ---- Collection metadata (rows 2-5) ----
    let meta_start = 2u32;
    let meta_rows: &[(&str, &str)] = &[
        ("Case Number:", case_number),
        ("Collecting Officer:", &ev.collecting_officer),
        ("Collection Date:", &ev.collection_date),
        ("Authorization:", &ev.authorization),
    ];
    for (i, (lbl, val)) in meta_rows.iter().enumerate() {
        let r = meta_start + i as u32;
        let val = export_field(val);
        ws.write_with_format(r, 0, *lbl, &meta_label)
            .map_err(xlsx_err)?;
        ws.write_with_format(r, 1, val.as_str(), &meta_val)
            .map_err(xlsx_err)?;
    }

    if !ev.witnesses.is_empty() {
        let r = meta_start + meta_rows.len() as u32;
        let witnesses = export_field(&ev.witnesses.join(", "));
        ws.write_with_format(r, 0, "Witnesses:", &meta_label)
            .map_err(xlsx_err)?;
        ws.write_with_format(r, 1, witnesses.as_str(), &meta_val)
            .map_err(xlsx_err)?;
    }

    // ---- Item headers ----
    let hdr_row = meta_start + meta_rows.len() as u32 + 2;
    for (c, h) in HEADERS.iter().enumerate() {
        ws.write_with_format(hdr_row, c as u16, *h, &header_fmt)
            .map_err(xlsx_err)?;
    }

    // ---- Item data rows ----
    for (i, item) in ev.collected_items.iter().enumerate() {
        let r = hdr_row + 1 + i as u32;
        for (c, val) in item_to_spreadsheet_row(item).iter().enumerate() {
            ws.write_with_format(r, c as u16, val.as_str(), &cell_fmt)
                .map_err(xlsx_err)?;
        }
    }

    // ---- Column widths ----
    let widths: &[f64] = &[
        8.0, 18.0, 18.0, 18.0, 16.0, 14.0, 16.0, 12.0, 14.0, 10.0, 16.0, 16.0, 18.0, 12.0, 10.0,
        14.0, 18.0, 12.0, 18.0, 14.0, 12.0, 18.0, 24.0, 14.0, 18.0, 28.0, 16.0, 36.0, 20.0, 24.0,
    ];
    for (c, w) in widths.iter().enumerate() {
        ws.set_column_width(c as u16, *w).map_err(xlsx_err)?;
    }

    // ---- Freeze header row ----
    ws.set_freeze_panes(hdr_row + 1, 0).map_err(xlsx_err)?;

    wb.save(output_path).map_err(xlsx_err)?;
    Ok(())
}

// =============================================================================
// HTML Export
// =============================================================================

/// Export evidence collection data to a self-contained HTML file
pub fn export_html(
    ev: &EvidenceCollectionData,
    case_number: &str,
    output_path: impl AsRef<Path>,
) -> ReportResult<()> {
    let mut html = String::with_capacity(8192);
    html.push_str(HTML_HEADER);

    // ---- Title ----
    html.push_str("<div class=\"title\">FORENSIC LABORATORY</div>\n");
    html.push_str("<div class=\"title\">EVIDENCE COLLECTION FORM</div>\n");
    html.push_str("<hr>\n");

    // ---- Collection metadata ----
    html.push_str("<table class=\"meta\">\n");
    meta_row(&mut html, "Case Number", case_number);
    meta_row(&mut html, "Collecting Officer", &ev.collecting_officer);
    meta_row(&mut html, "Collection Date", &ev.collection_date);
    meta_row(&mut html, "Authorization", &ev.authorization);
    if let Some(ref d) = ev.authorization_date {
        meta_row(&mut html, "Auth. Date", d);
    }
    if let Some(ref a) = ev.authorizing_authority {
        meta_row(&mut html, "Authorizing Authority", a);
    }
    if !ev.witnesses.is_empty() {
        meta_row(&mut html, "Witnesses", &ev.witnesses.join(", "));
    }
    html.push_str("</table>\n<br>\n");

    // ---- Items table ----
    if !ev.collected_items.is_empty() {
        html.push_str("<table class=\"items\">\n<thead><tr>\n");
        for h in HEADERS {
            html.push_str(&format!("<th>{}</th>", esc(h)));
        }
        html.push_str("</tr></thead>\n<tbody>\n");

        for item in &ev.collected_items {
            html.push_str("<tr>\n");
            for val in item_to_row(item) {
                html.push_str(&format!("<td>{}</td>", esc(&val)));
            }
            html.push_str("</tr>\n");
        }
        html.push_str("</tbody></table>\n");
    } else {
        html.push_str("<p class=\"empty\">No items collected.</p>\n");
    }

    // ---- Notes ----
    if ev
        .documentation_notes
        .as_ref()
        .is_some_and(|n| !n.is_empty())
        || ev.conditions.as_ref().is_some_and(|c| !c.is_empty())
    {
        html.push_str("<div class=\"notes\"><h3>Notes</h3>\n");
        if let Some(ref n) = ev.documentation_notes {
            if !n.is_empty() {
                html.push_str(&format!(
                    "<p><strong>Documentation:</strong> {}</p>\n",
                    esc_export(n)
                ));
            }
        }
        if let Some(ref c) = ev.conditions {
            if !c.is_empty() {
                html.push_str(&format!(
                    "<p><strong>Conditions:</strong> {}</p>\n",
                    esc_export(c)
                ));
            }
        }
        html.push_str("</div>\n");
    }

    // ---- Footer ----
    html.push_str("<div class=\"footer\">");
    html.push_str(
        "Evidence Collection Form &bull; v2026.02 &bull; CORE-FFX Forensic File Explorer",
    );
    html.push_str("</div>\n");
    html.push_str("</body></html>");

    std::fs::write(output_path, &html)?;
    Ok(())
}

// =============================================================================
// Shared helpers
// =============================================================================

/// Convert a CollectedItem into a flat row of string values (column-aligned with HEADERS)
fn item_to_row(item: &CollectedItem) -> Vec<String> {
    item_to_raw_row(item)
        .into_iter()
        .map(|value| export_field(&value))
        .collect()
}

fn item_to_spreadsheet_row(item: &CollectedItem) -> Vec<String> {
    item_to_raw_row(item)
        .into_iter()
        .map(|value| spreadsheet_field(&value))
        .collect()
}

fn item_to_raw_row(item: &CollectedItem) -> Vec<String> {
    vec![
        item.item_number.clone(),
        item.item_collection_datetime.clone().unwrap_or_default(),
        item.item_system_datetime.clone().unwrap_or_default(),
        item.item_collecting_officer.clone().unwrap_or_default(),
        item.item_authorization.clone().unwrap_or_default(),
        if !item.device_type.is_empty() {
            item.device_type.clone()
        } else {
            item.item_type.clone()
        },
        item.brand.clone().unwrap_or_default(),
        item.make.clone().unwrap_or_default(),
        item.model.clone().unwrap_or_default(),
        item.color.clone().unwrap_or_default(),
        item.serial_number.clone().unwrap_or_default(),
        item.imei.clone().unwrap_or_default(),
        item.other_identifiers.clone().unwrap_or_default(),
        item.building.clone().unwrap_or_default(),
        item.room.clone().unwrap_or_default(),
        item.location_other.clone().unwrap_or_default(),
        item.found_location.clone(),
        item.image_format.clone().unwrap_or_default(),
        item.acquisition_method.clone().unwrap_or_default(),
        item.condition.clone(),
        item.packaging.clone(),
        item.storage_notes.clone().unwrap_or_default(),
        item.notes.clone().unwrap_or_default(),
        item.photo_refs.join(", "),
        item.evidence_file_id.clone().unwrap_or_default(),
        item.source_id.clone().unwrap_or_default(),
        item.hash_algorithm.clone().unwrap_or_default(),
        item.hash_value.clone().unwrap_or_default(),
        item.hash_computed_at.clone().unwrap_or_default(),
        item.description.clone(),
    ]
}

fn export_field(value: &str) -> String {
    truncate_chars(value, MAX_EXPORT_FIELD_CHARS)
}

fn spreadsheet_field(value: &str) -> String {
    if !needs_spreadsheet_escape(value) {
        return export_field(value);
    }

    let mut escaped =
        String::with_capacity(MAX_EXPORT_FIELD_CHARS.min(value.len().saturating_add(1)));
    escaped.push('\'');
    escaped.push_str(&truncate_chars(
        value,
        MAX_EXPORT_FIELD_CHARS.saturating_sub(1),
    ));
    escaped
}

fn needs_spreadsheet_escape(value: &str) -> bool {
    let Some(first) = value.chars().next() else {
        return false;
    };
    if is_spreadsheet_formula_prefix(first) {
        return true;
    }
    first == ' '
        && value
            .trim_start_matches(' ')
            .chars()
            .next()
            .is_some_and(is_spreadsheet_formula_prefix)
}

fn is_spreadsheet_formula_prefix(ch: char) -> bool {
    matches!(ch, '=' | '+' | '-' | '@' | '\t' | '\r' | '\n')
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }

    let suffix_chars = TRUNCATED_FIELD_SUFFIX.chars().count();
    let keep_chars = max_chars.saturating_sub(suffix_chars);
    let end = value
        .char_indices()
        .nth(keep_chars)
        .map(|(idx, _)| idx)
        .unwrap_or(value.len());
    let mut truncated = String::with_capacity(end + TRUNCATED_FIELD_SUFFIX.len());
    truncated.push_str(&value[..end]);
    truncated.push_str(TRUNCATED_FIELD_SUFFIX);
    truncated
}

/// HTML-escape a string
fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn esc_export(s: &str) -> String {
    esc(&export_field(s))
}

fn meta_row(html: &mut String, label: &str, value: &str) {
    html.push_str(&format!(
        "<tr><td class=\"lbl\">{}</td><td>{}</td></tr>\n",
        esc(label),
        esc_export(value)
    ));
}

fn csv_err(e: csv::Error) -> ReportError {
    ReportError::Pdf(format!("CSV error: {}", e))
}

fn xlsx_err(e: impl std::fmt::Display) -> ReportError {
    ReportError::Pdf(format!("XLSX error: {}", e))
}

// =============================================================================
// HTML template (inline CSS for self-contained output)
// =============================================================================

const HTML_HEADER: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Evidence Collection Form</title>
<style>
  @page { size: landscape; margin: 1cm; }
  body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    font-size: 11px; color: #1a1a1a; max-width: 1200px; margin: 0 auto; padding: 20px;
  }
  .title {
    text-align: center; font-weight: 700; font-size: 16px;
    letter-spacing: 1px; color: #1F3864; margin: 2px 0;
  }
  hr { border: none; border-top: 2px solid #1F3864; margin: 8px 0 12px; }
  table { border-collapse: collapse; width: 100%; margin-bottom: 8px; }
  .meta td { padding: 3px 8px; border: 1px solid #ccc; }
  .meta .lbl { font-weight: 700; background: #D9E2F3; width: 180px; white-space: nowrap; }
  .items th {
    background: #2E75B6; color: #fff; font-weight: 600; font-size: 10px;
    padding: 6px 4px; border: 1px solid #2E75B6; text-align: center;
    position: sticky; top: 0; z-index: 1;
  }
  .items td {
    padding: 4px 6px; border: 1px solid #ddd; vertical-align: top;
    font-size: 10px; word-break: break-word;
  }
  .items tr:nth-child(even) { background: #F2F6FC; }
  .items tr:hover { background: #E0ECFA; }
  .notes { margin-top: 12px; padding: 8px 12px; background: #FFFBE6; border: 1px solid #E0D48E; border-radius: 4px; }
  .notes h3 { margin: 0 0 4px; font-size: 12px; }
  .notes p { margin: 2px 0; font-size: 11px; }
  .empty { color: #888; font-style: italic; text-align: center; padding: 20px; }
  .footer {
    margin-top: 20px; text-align: center; font-size: 9px; color: #888;
    border-top: 1px solid #ccc; padding-top: 6px;
  }
  @media print {
    .items th { background: #2E75B6 !important; -webkit-print-color-adjust: exact; print-color-adjust: exact; }
    .items tr:nth-child(even) { background: #F2F6FC !important; -webkit-print-color-adjust: exact; print-color-adjust: exact; }
  }
</style>
</head>
<body>
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_row_includes_source_and_hash_snapshot() {
        let item = CollectedItem {
            item_number: "ITEM-001".to_string(),
            description: "Logical image".to_string(),
            source_id: Some("ad1:/case/logical.ad1:/docs/a.txt".to_string()),
            evidence_file_id: Some("ev-1".to_string()),
            hash_algorithm: Some("SHA-256".to_string()),
            hash_value: Some(
                "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
            ),
            hash_computed_at: Some("2026-04-14T10:02:00Z".to_string()),
            ..Default::default()
        };

        let row = item_to_row(&item);

        assert_eq!(row.len(), HEADERS.len());
        assert_eq!(
            row[HEADERS
                .iter()
                .position(|header| *header == "Evidence File ID")
                .expect("evidence file column")],
            "ev-1"
        );
        assert_eq!(
            row[HEADERS
                .iter()
                .position(|header| *header == "Source ID")
                .expect("source column")],
            "ad1:/case/logical.ad1:/docs/a.txt"
        );
        assert_eq!(
            row[HEADERS
                .iter()
                .position(|header| *header == "Hash Algorithm")
                .expect("hash algorithm column")],
            "SHA-256"
        );
        assert_eq!(
            row[HEADERS
                .iter()
                .position(|header| *header == "Hash Value")
                .expect("hash value column")],
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
        );
        assert_eq!(
            row[HEADERS
                .iter()
                .position(|header| *header == "Hash Computed At")
                .expect("hash computed column")],
            "2026-04-14T10:02:00Z"
        );
    }

    #[test]
    fn item_row_truncates_oversized_export_fields() {
        let oversized_notes = "n".repeat(MAX_EXPORT_FIELD_CHARS + 256);
        let item = CollectedItem {
            item_number: "ITEM-001".to_string(),
            notes: Some(oversized_notes),
            ..Default::default()
        };

        let row = item_to_row(&item);
        let notes = &row[HEADERS
            .iter()
            .position(|header| *header == "Notes")
            .expect("notes column")];

        assert_eq!(notes.chars().count(), MAX_EXPORT_FIELD_CHARS);
        assert!(notes.ends_with(TRUNCATED_FIELD_SUFFIX));
    }

    #[test]
    fn spreadsheet_row_neutralizes_formula_like_cells() {
        let item = CollectedItem {
            item_number: "=2+2".to_string(),
            description: " +SUM(A1:A2)".to_string(),
            notes: Some("@cmd".to_string()),
            found_location: "-10".to_string(),
            ..Default::default()
        };

        let row = item_to_spreadsheet_row(&item);

        assert_eq!(row[0], "'=2+2");
        assert_eq!(
            row[HEADERS
                .iter()
                .position(|header| *header == "Description")
                .expect("description column")],
            "' +SUM(A1:A2)"
        );
        assert_eq!(
            row[HEADERS
                .iter()
                .position(|header| *header == "Notes")
                .expect("notes column")],
            "'@cmd"
        );
        assert_eq!(
            row[HEADERS
                .iter()
                .position(|header| *header == "Found Location")
                .expect("found location column")],
            "'-10"
        );

        let html_row = item_to_row(&item);
        assert_eq!(html_row[0], "=2+2");
    }

    #[test]
    fn spreadsheet_formula_escape_preserves_field_cap() {
        let value = format!("={}", "x".repeat(MAX_EXPORT_FIELD_CHARS + 256));
        let escaped = spreadsheet_field(&value);

        assert!(escaped.starts_with("'="));
        assert_eq!(escaped.chars().count(), MAX_EXPORT_FIELD_CHARS);
        assert!(escaped.ends_with(TRUNCATED_FIELD_SUFFIX));
    }

    #[test]
    fn metadata_values_are_truncated_before_html_escaping() {
        let value = format!("<tag>{}", "w".repeat(MAX_EXPORT_FIELD_CHARS + 128));
        let mut html = String::new();

        meta_row(&mut html, "Witnesses", &value);

        assert!(html.contains(TRUNCATED_FIELD_SUFFIX));
        assert!(html.contains("&lt;tag&gt;"));
        assert!(!html.contains("<tag>"));
    }
}
