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
    "Description",
];

// =============================================================================
// CSV Export
// =============================================================================

/// Export evidence collection data to CSV
pub fn export_csv(ev: &EvidenceCollectionData, output_path: impl AsRef<Path>) -> ReportResult<()> {
    let file = std::fs::File::create(output_path)?;
    let mut wtr = csv::Writer::from_writer(file);

    wtr.write_record(HEADERS).map_err(csv_err)?;

    for item in &ev.collected_items {
        wtr.write_record(&item_to_row(item)).map_err(csv_err)?;
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
    ws.merge_range(0, 0, 0, (HEADERS.len() - 1) as u16, "EVIDENCE COLLECTION FORM", &title_fmt)
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
        ws.write_with_format(r, 0, *lbl, &meta_label).map_err(xlsx_err)?;
        ws.write_with_format(r, 1, *val, &meta_val).map_err(xlsx_err)?;
    }

    if !ev.witnesses.is_empty() {
        let r = meta_start + meta_rows.len() as u32;
        ws.write_with_format(r, 0, "Witnesses:", &meta_label).map_err(xlsx_err)?;
        ws.write_with_format(r, 1, &ev.witnesses.join(", "), &meta_val).map_err(xlsx_err)?;
    }

    // ---- Item headers ----
    let hdr_row = meta_start + meta_rows.len() as u32 + 2;
    for (c, h) in HEADERS.iter().enumerate() {
        ws.write_with_format(hdr_row, c as u16, *h, &header_fmt).map_err(xlsx_err)?;
    }

    // ---- Item data rows ----
    for (i, item) in ev.collected_items.iter().enumerate() {
        let r = hdr_row + 1 + i as u32;
        for (c, val) in item_to_row(item).iter().enumerate() {
            ws.write_with_format(r, c as u16, val.as_str(), &cell_fmt).map_err(xlsx_err)?;
        }
    }

    // ---- Column widths ----
    let widths: &[f64] = &[
        8.0, 18.0, 18.0, 18.0, 16.0, 14.0, 16.0, 12.0, 14.0, 10.0, 16.0, 16.0, 18.0,
        12.0, 10.0, 14.0, 18.0, 12.0, 18.0, 14.0, 12.0, 18.0,
        24.0, 14.0, 24.0,
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
    if ev.documentation_notes.as_ref().map_or(false, |n| !n.is_empty())
        || ev.conditions.as_ref().map_or(false, |c| !c.is_empty())
    {
        html.push_str("<div class=\"notes\"><h3>Notes</h3>\n");
        if let Some(ref n) = ev.documentation_notes {
            if !n.is_empty() {
                html.push_str(&format!("<p><strong>Documentation:</strong> {}</p>\n", esc(n)));
            }
        }
        if let Some(ref c) = ev.conditions {
            if !c.is_empty() {
                html.push_str(&format!("<p><strong>Conditions:</strong> {}</p>\n", esc(c)));
            }
        }
        html.push_str("</div>\n");
    }

    // ---- Footer ----
    html.push_str("<div class=\"footer\">");
    html.push_str("Evidence Collection Form &bull; v2026.02 &bull; CORE-FFX Forensic File Explorer");
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
    vec![
        item.item_number.clone(),
        item.item_collection_datetime.as_deref().unwrap_or("").to_string(),
        item.item_system_datetime.as_deref().unwrap_or("").to_string(),
        item.item_collecting_officer.as_deref().unwrap_or("").to_string(),
        item.item_authorization.as_deref().unwrap_or("").to_string(),
        if !item.device_type.is_empty() {
            item.device_type.clone()
        } else {
            item.item_type.clone()
        },
        item.brand.as_deref().unwrap_or("").to_string(),
        item.make.as_deref().unwrap_or("").to_string(),
        item.model.as_deref().unwrap_or("").to_string(),
        item.color.as_deref().unwrap_or("").to_string(),
        item.serial_number.as_deref().unwrap_or("").to_string(),
        item.imei.as_deref().unwrap_or("").to_string(),
        item.other_identifiers.as_deref().unwrap_or("").to_string(),
        item.building.as_deref().unwrap_or("").to_string(),
        item.room.as_deref().unwrap_or("").to_string(),
        item.location_other.as_deref().unwrap_or("").to_string(),
        item.found_location.clone(),
        item.image_format.as_deref().unwrap_or("").to_string(),
        item.acquisition_method.as_deref().unwrap_or("").to_string(),
        item.condition.clone(),
        item.packaging.clone(),
        item.storage_notes.as_deref().unwrap_or("").to_string(),
        item.notes.as_deref().unwrap_or("").to_string(),
        item.photo_refs.join(", "),
        item.description.clone(),
    ]
}

/// HTML-escape a string
fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn meta_row(html: &mut String, label: &str, value: &str) {
    html.push_str(&format!(
        "<tr><td class=\"lbl\">{}</td><td>{}</td></tr>\n",
        esc(label),
        esc(value)
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
