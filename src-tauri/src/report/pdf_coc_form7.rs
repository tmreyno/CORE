// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! PDF renderer for EPA CID OCEFT Form 7-01 — Chain of Custody
//!
//! Generates a PDF that reproduces the exact layout of the EPA CID Criminal
//! Investigation Division's Chain of Custody Form 7-01 (Rev 03/2017).
//!
//! Page structure:
//!   Page 1: Case header, owner info, source checkboxes, collected by,
//!           relinquishment rows (6), final disposition, item/description table
//!   Page 2+: Continuation — Item/Box Number + Description table
//!   Last page(s): Continuation — Additional relinquishment/transfer rows

use genpdf::{
    elements::{Break, LinearLayout, Paragraph, StyledElement, TableLayout, Text},
    fonts, style, Alignment, Document, Element,
};
use std::path::Path;

use super::error::{ReportError, ReportResult};
use super::types::{CocItem, ForensicReport};

/// Maximum relinquishment rows on page 1
const PAGE1_RELINQUISHMENT_ROWS: usize = 6;
/// Maximum relinquishment rows on a continuation page
const CONTINUATION_RELINQUISHMENT_ROWS: usize = 20;
/// Maximum item rows on page 1 (after relinquishment section)
const PAGE1_ITEM_ROWS: usize = 8;
/// Maximum item rows on a continuation page
const CONTINUATION_ITEM_ROWS: usize = 30;

/// Generate the EPA CID OCEFT Form 7-01 Chain of Custody PDF
pub fn generate_coc_form7(
    report: &ForensicReport,
    font_family: fonts::FontFamily<fonts::FontData>,
    output_path: impl AsRef<Path>,
) -> ReportResult<()> {
    let coc_items = report.coc_items.as_deref().unwrap_or(&[]);
    if coc_items.is_empty() {
        return Err(ReportError::Pdf(
            "No Chain of Custody items found in report data".to_string(),
        ));
    }

    let mut doc = Document::new(font_family);
    doc.set_title("Chain of Custody - OCEFT Form 7-01");
    doc.set_minimal_conformance();

    // Use the first COC item for case-level header fields
    let primary = &coc_items[0];

    // Calculate total pages
    let total_item_rows: usize = coc_items.len();
    let total_transfers: usize = coc_items.iter().map(|c| c.transfers.len()).sum();

    // Page 1 content
    add_form_header(&mut doc)?;
    add_case_header(&mut doc, report, primary)?;
    add_owner_section(&mut doc, primary)?;
    add_source_section(&mut doc, primary)?;
    add_collected_by_section(&mut doc, primary)?;

    // Relinquishment rows (page 1: up to 6 from first item's transfers)
    let transfers = &primary.transfers;
    add_relinquishment_section(&mut doc, transfers, 0, PAGE1_RELINQUISHMENT_ROWS)?;

    // Final disposition section
    add_disposition_section(&mut doc, primary)?;

    // Item/Description table header + first batch of items
    let items_on_page1 = total_item_rows.min(PAGE1_ITEM_ROWS);
    add_item_table(&mut doc, coc_items, 0, items_on_page1)?;

    add_form_footer(&mut doc, 1)?;

    // Continuation pages for additional items
    let mut item_offset = items_on_page1;
    let mut page_num = 2;
    while item_offset < total_item_rows {
        doc.push(Break::new(4.0)); // page break approximation
        add_continuation_header(&mut doc, report, primary, "Items")?;

        let batch = (total_item_rows - item_offset).min(CONTINUATION_ITEM_ROWS);
        add_item_table(&mut doc, coc_items, item_offset, batch)?;

        item_offset += batch;
        add_form_footer(&mut doc, page_num)?;
        page_num += 1;
    }

    // Continuation pages for additional transfers (beyond page 1's 6)
    if total_transfers > PAGE1_RELINQUISHMENT_ROWS {
        doc.push(Break::new(4.0));
        add_continuation_header(&mut doc, report, primary, "Transfers")?;

        // Gather all transfers across all COC items
        let all_transfers: Vec<_> = coc_items.iter().flat_map(|c| c.transfers.iter()).collect();

        let remaining_start = PAGE1_RELINQUISHMENT_ROWS;
        let mut transfer_offset = remaining_start;
        while transfer_offset < all_transfers.len() {
            let batch =
                (all_transfers.len() - transfer_offset).min(CONTINUATION_RELINQUISHMENT_ROWS);
            add_relinquishment_rows_from_slice(
                &mut doc,
                &all_transfers[transfer_offset..transfer_offset + batch],
            )?;
            transfer_offset += batch;

            add_form_footer(&mut doc, page_num)?;
            page_num += 1;

            if transfer_offset < all_transfers.len() {
                doc.push(Break::new(4.0));
                add_continuation_header(&mut doc, report, primary, "Transfers")?;
            }
        }
    }

    // Render
    doc.render_to_file(output_path)
        .map_err(|e| ReportError::Pdf(e.to_string()))?;

    Ok(())
}

// =============================================================================
// Form sections
// =============================================================================

fn add_form_header(doc: &mut Document) -> ReportResult<()> {
    doc.push(
        Paragraph::new("United States Environmental Protection Agency")
            .aligned(Alignment::Center)
            .styled(style::Style::new().bold().with_font_size(11)),
    );
    doc.push(
        Paragraph::new("Criminal Investigation Division")
            .aligned(Alignment::Center)
            .styled(style::Style::new().bold().with_font_size(11)),
    );
    doc.push(Break::new(0.3));
    doc.push(
        Paragraph::new("CHAIN OF CUSTODY")
            .aligned(Alignment::Center)
            .styled(style::Style::new().bold().with_font_size(14)),
    );
    doc.push(Break::new(0.5));
    Ok(())
}

fn add_case_header(
    doc: &mut Document,
    report: &ForensicReport,
    primary: &CocItem,
) -> ReportResult<()> {
    // Case Title | Office
    let mut row1 = TableLayout::new(vec![3, 1]);
    row1.set_cell_decorator(genpdf::elements::FrameCellDecorator::new(true, true, false));

    let case_name = report.case_info.case_name.as_deref().unwrap_or("");
    row1.row()
        .element(label_value("Case Title:", case_name))
        .element(label_value(
            "Office:",
            report.case_info.agency.as_deref().unwrap_or(""),
        ))
        .push()
        .map_err(|e| ReportError::Pdf(e.to_string()))?;
    doc.push(row1);

    // Case Number | COC#
    let mut row2 = TableLayout::new(vec![3, 1]);
    row2.set_cell_decorator(genpdf::elements::FrameCellDecorator::new(true, true, false));
    row2.row()
        .element(label_value("Case Number:", &report.case_info.case_number))
        .element(label_value("COC#:", &primary.coc_number))
        .push()
        .map_err(|e| ReportError::Pdf(e.to_string()))?;
    doc.push(row2);

    doc.push(Break::new(0.3));
    Ok(())
}

fn add_owner_section(doc: &mut Document, primary: &CocItem) -> ReportResult<()> {
    let mut table = TableLayout::new(vec![1, 1, 1]);
    table.set_cell_decorator(genpdf::elements::FrameCellDecorator::new(true, true, false));

    // Owner info — using submitted_by as owner name, received_location as address
    table
        .row()
        .element(label_value("Owner Name:", &primary.submitted_by))
        .element(label_value(
            "Address:",
            primary.received_location.as_deref().unwrap_or(""),
        ))
        .element(label_value("Phone:", ""))
        .push()
        .map_err(|e| ReportError::Pdf(e.to_string()))?;
    doc.push(table);

    doc.push(Break::new(0.3));
    Ok(())
}

fn add_source_section(doc: &mut Document, primary: &CocItem) -> ReportResult<()> {
    doc.push(
        Paragraph::new("Source (Check applicable):")
            .styled(style::Style::new().bold().with_font_size(9)),
    );

    let reason = primary.reason_submitted.as_deref().unwrap_or("");
    let source_options = [
        "Search Warrant",
        "Grand Jury Subpoena",
        "Consent Seizure",
        "Abandoned",
        "Digital/Electronic Capture",
        "Voluntary Submission",
        "Other",
    ];

    let mut layout = LinearLayout::vertical();
    // Render checkboxes in two rows of options
    let mut line1 = String::new();
    let mut line2 = String::new();
    for (i, opt) in source_options.iter().enumerate() {
        let checked = reason.to_lowercase().contains(&opt.to_lowercase());
        let mark = if checked { "[X]" } else { "[ ]" };
        let entry = format!("{} {}   ", mark, opt);
        if i < 4 {
            line1.push_str(&entry);
        } else {
            line2.push_str(&entry);
        }
    }

    layout.push(Paragraph::new(line1).styled(style::Style::new().with_font_size(9)));
    layout.push(Paragraph::new(line2).styled(style::Style::new().with_font_size(9)));
    doc.push(layout);

    // Other Contact
    doc.push(Break::new(0.2));
    let mut contact_table = TableLayout::new(vec![1, 1, 1]);
    contact_table.set_cell_decorator(genpdf::elements::FrameCellDecorator::new(true, true, false));
    contact_table
        .row()
        .element(label_value("Other Contact Name:", ""))
        .element(label_value("Relationship:", ""))
        .element(label_value("Phone:", ""))
        .push()
        .map_err(|e| ReportError::Pdf(e.to_string()))?;
    doc.push(contact_table);

    doc.push(Break::new(0.3));
    Ok(())
}

fn add_collected_by_section(doc: &mut Document, primary: &CocItem) -> ReportResult<()> {
    let mut table = TableLayout::new(vec![2, 2, 1]);
    table.set_cell_decorator(genpdf::elements::FrameCellDecorator::new(true, true, false));

    table
        .row()
        .element(label_value("Collected By (Print):", &primary.received_by))
        .element(label_value("Sign:", ""))
        .element(label_value("Date:", &primary.entered_custody_date))
        .push()
        .map_err(|e| ReportError::Pdf(e.to_string()))?;
    doc.push(table);

    // Storage location + date + remarks
    let mut storage = TableLayout::new(vec![2, 2]);
    storage.set_cell_decorator(genpdf::elements::FrameCellDecorator::new(true, true, false));
    storage
        .row()
        .element(label_value(
            "Storage Location:",
            primary.storage_location.as_deref().unwrap_or(""),
        ))
        .element(label_value("Date Entered:", &primary.entered_custody_date))
        .push()
        .map_err(|e| ReportError::Pdf(e.to_string()))?;
    doc.push(storage);

    // Remarks
    let mut remarks = TableLayout::new(vec![1]);
    remarks.set_cell_decorator(genpdf::elements::FrameCellDecorator::new(true, true, false));
    remarks
        .row()
        .element(label_value(
            "Remarks:",
            primary.notes.as_deref().unwrap_or(""),
        ))
        .push()
        .map_err(|e| ReportError::Pdf(e.to_string()))?;
    doc.push(remarks);

    doc.push(Break::new(0.3));
    Ok(())
}

fn add_relinquishment_section(
    doc: &mut Document,
    transfers: &[super::types::CocTransfer],
    offset: usize,
    max_rows: usize,
) -> ReportResult<()> {
    let end = (offset + max_rows).min(transfers.len());

    for i in 0..max_rows {
        let mut row = TableLayout::new(vec![2, 1, 1, 2, 1]);
        row.set_cell_decorator(genpdf::elements::FrameCellDecorator::new(true, true, false));

        if offset + i < end {
            let t = &transfers[offset + i];
            row.row()
                .element(label_value("Relinquished to (Print):", &t.received_by))
                .element(label_value("Sign:", ""))
                .element(label_value("Date:", &t.timestamp))
                .element(label_value(
                    "Storage Location:",
                    t.location.as_deref().unwrap_or(""),
                ))
                .element(label_value("Date Entered:", &t.timestamp))
                .push()
                .map_err(|e| ReportError::Pdf(e.to_string()))?;
        } else {
            // Empty row for blank form lines
            row.row()
                .element(label_value("Relinquished to (Print):", ""))
                .element(label_value("Sign:", ""))
                .element(label_value("Date:", ""))
                .element(label_value("Storage Location:", ""))
                .element(label_value("Date Entered:", ""))
                .push()
                .map_err(|e| ReportError::Pdf(e.to_string()))?;
        }

        doc.push(row);
    }

    doc.push(Break::new(0.3));
    Ok(())
}

fn add_relinquishment_rows_from_slice(
    doc: &mut Document,
    transfers: &[&super::types::CocTransfer],
) -> ReportResult<()> {
    for t in transfers {
        let mut row = TableLayout::new(vec![2, 1, 1, 2, 1]);
        row.set_cell_decorator(genpdf::elements::FrameCellDecorator::new(true, true, false));

        row.row()
            .element(label_value("Relinquished to (Print):", &t.received_by))
            .element(label_value("Sign:", ""))
            .element(label_value("Date:", &t.timestamp))
            .element(label_value(
                "Storage Location:",
                t.location.as_deref().unwrap_or(""),
            ))
            .element(label_value("Date Entered:", &t.timestamp))
            .push()
            .map_err(|e| ReportError::Pdf(e.to_string()))?;

        doc.push(row);
    }
    Ok(())
}

fn add_disposition_section(doc: &mut Document, primary: &CocItem) -> ReportResult<()> {
    doc.push(
        Paragraph::new("FINAL DISPOSITION").styled(style::Style::new().bold().with_font_size(10)),
    );

    let mut table = TableLayout::new(vec![1, 1, 1, 1]);
    table.set_cell_decorator(genpdf::elements::FrameCellDecorator::new(true, true, false));

    let disp = primary.disposition.as_deref().unwrap_or("in_custody");
    let disp_date = primary.disposition_date.as_deref().unwrap_or("");
    let disp_notes = primary.disposition_notes.as_deref().unwrap_or("");

    table
        .row()
        .element(label_value(
            "Final Disposition By (Print/Sign):",
            &primary.received_by,
        ))
        .element(label_value("Returned to (Sign/Date):", ""))
        .element(label_value(
            "Destruction Date:",
            if disp == "destroyed" { disp_date } else { "" },
        ))
        .element(label_value(
            "Other Disposition:",
            if disp == "released" || disp == "returned" {
                disp_notes
            } else {
                ""
            },
        ))
        .push()
        .map_err(|e| ReportError::Pdf(e.to_string()))?;
    doc.push(table);

    doc.push(Break::new(0.3));
    Ok(())
}

fn add_item_table(
    doc: &mut Document,
    items: &[CocItem],
    offset: usize,
    count: usize,
) -> ReportResult<()> {
    doc.push(
        Paragraph::new("ITEM / BOX NUMBER AND DESCRIPTION")
            .styled(style::Style::new().bold().with_font_size(10)),
    );
    doc.push(Break::new(0.2));

    let mut table = TableLayout::new(vec![1, 4]);
    table.set_cell_decorator(genpdf::elements::FrameCellDecorator::new(true, true, false));

    // Header
    table
        .row()
        .element(Text::new("Item/Box Number").styled(style::Style::new().bold().with_font_size(9)))
        .element(Text::new("Description").styled(style::Style::new().bold().with_font_size(9)))
        .push()
        .map_err(|e| ReportError::Pdf(e.to_string()))?;

    let end = (offset + count).min(items.len());
    for item in &items[offset..end] {
        // Build description string with details
        let mut desc_parts = vec![item.description.clone()];
        if let Some(ref make) = item.make {
            desc_parts.push(format!("Make: {}", make));
        }
        if let Some(ref model) = item.model {
            desc_parts.push(format!("Model: {}", model));
        }
        if let Some(ref sn) = item.serial_number {
            desc_parts.push(format!("S/N: {}", sn));
        }
        if let Some(ref cap) = item.capacity {
            desc_parts.push(format!("Capacity: {}", cap));
        }
        if !item.condition.is_empty() {
            desc_parts.push(format!("Condition: {}", item.condition));
        }

        // Include intake hashes
        for h in &item.intake_hashes {
            desc_parts.push(format!("{}: {}", h.algorithm, h.value));
        }

        let desc = desc_parts.join(" | ");

        table
            .row()
            .element(Text::new(&item.evidence_id).styled(style::Style::new().with_font_size(8)))
            .element(Text::new(desc).styled(style::Style::new().with_font_size(8)))
            .push()
            .map_err(|e| ReportError::Pdf(e.to_string()))?;
    }

    // Fill remaining rows as empty for the form appearance
    let filled = end - offset;
    let total_rows = if offset == 0 {
        PAGE1_ITEM_ROWS
    } else {
        CONTINUATION_ITEM_ROWS
    };
    for _ in filled..total_rows {
        table
            .row()
            .element(Text::new("").styled(style::Style::new().with_font_size(8)))
            .element(Text::new("").styled(style::Style::new().with_font_size(8)))
            .push()
            .map_err(|e| ReportError::Pdf(e.to_string()))?;
    }

    doc.push(table);
    Ok(())
}

fn add_continuation_header(
    doc: &mut Document,
    report: &ForensicReport,
    primary: &CocItem,
    section: &str,
) -> ReportResult<()> {
    doc.push(
        Paragraph::new("United States Environmental Protection Agency")
            .aligned(Alignment::Center)
            .styled(style::Style::new().bold().with_font_size(10)),
    );
    doc.push(
        Paragraph::new("Criminal Investigation Division")
            .aligned(Alignment::Center)
            .styled(style::Style::new().bold().with_font_size(10)),
    );
    doc.push(
        Paragraph::new(format!("CHAIN OF CUSTODY — Continuation ({})", section))
            .aligned(Alignment::Center)
            .styled(style::Style::new().bold().with_font_size(12)),
    );
    doc.push(Break::new(0.3));

    // Condensed case header
    let mut header = TableLayout::new(vec![2, 1, 1]);
    header.set_cell_decorator(genpdf::elements::FrameCellDecorator::new(true, true, false));
    header
        .row()
        .element(label_value(
            "Case Title:",
            report.case_info.case_name.as_deref().unwrap_or(""),
        ))
        .element(label_value("Case Number:", &report.case_info.case_number))
        .element(label_value("COC#:", &primary.coc_number))
        .push()
        .map_err(|e| ReportError::Pdf(e.to_string()))?;
    doc.push(header);

    doc.push(Break::new(0.3));
    Ok(())
}

fn add_form_footer(doc: &mut Document, page_num: usize) -> ReportResult<()> {
    doc.push(Break::new(0.5));
    doc.push(
        Paragraph::new(format!(
            "OCEFT Form 7-01  Original - With Item  Copy-Evidence Log  Copy – eCase File  Rev_03/2017  Page {} of __",
            page_num
        ))
        .aligned(Alignment::Center)
        .styled(style::Style::new().with_font_size(7)),
    );
    Ok(())
}

// =============================================================================
// Helpers
// =============================================================================

/// Create a paragraph with a bold label followed by the value
fn label_value(label: &str, value: &str) -> StyledElement<Paragraph> {
    Paragraph::new(format!("{} {}", label, value)).styled(style::Style::new().with_font_size(9))
}
