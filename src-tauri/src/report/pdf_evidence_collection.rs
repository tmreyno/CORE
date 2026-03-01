// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! PDF renderer for EPA CID Computer Forensics Laboratory Evidence Collection Form
//!
//! Generates a PDF that reproduces the layout of the EPA CID Evidence Collection
//! Form (v.06-02-2021). Each collected item gets its own form page.
//!
//! Form sections:
//!   1. Initial Collection (Team Lead initials, CoC#, Image#)
//!   2. Actual Date/Time (24hr)
//!   3. System Date/Time accuracy
//!   4. Device Type Information (circle applicable)
//!   5. Brand, Model, Serial Number
//!   6. Location Found Data (Bldg, Room, Subloc, ITM)
//!   7. Total Devices / Primary User
//!   8. Forensic Image Data (Case#, Site, IMG#, Format, Method)
//!   9. Tool Used for Imaging, Image Size
//!  10. Hard Drive Info (Mfg/Mdl, S/N)
//!   + Notes section

use genpdf::{
    elements::{Break, LinearLayout, Paragraph, StyledElement, TableLayout, Text},
    fonts, style, Alignment, Document, Element,
};
use std::path::Path;

use super::error::{ReportError, ReportResult};
use super::types::{CollectedItem, EvidenceCollectionData, ForensicReport};

/// Device types available on the form (the "circle" options)
const DEVICE_TYPES: &[&str] = &[
    "Laptop",
    "Phone",
    "DVR/NVR",
    "Desktop",
    "Tablet",
    "External HDD",
    "Server",
    "Media Card",
    "Internal Drive",
    "NAS",
    "USB Flash",
    "Other",
];

/// Interface types available on the form
const INTERFACE_TYPES: &[&str] = &["IDE", "SATA", "SAS", "USB", "ThunderBolt", "Other"];

/// Image format options on the form
const IMAGE_FORMATS: &[&str] = &["AD1", "E01", "UFD", "DD", "ZIP-TAR"];

/// Image method options on the form
const IMAGE_METHODS: &[&str] = &[
    "Advanced Logical",
    "Physical",
    "VHD-VMDK",
    "Logical Files-Folders",
    "Logical Partition",
    "Native Files",
    "Original Item",
    "Other",
];

/// Generate the EPA CID Evidence Collection Form PDF
pub fn generate_evidence_collection(
    report: &ForensicReport,
    font_family: fonts::FontFamily<fonts::FontData>,
    output_path: impl AsRef<Path>,
) -> ReportResult<()> {
    let ev_data = report.evidence_collection.as_ref().ok_or_else(|| {
        ReportError::Pdf("No Evidence Collection data found in report".to_string())
    })?;

    let mut doc = Document::new(font_family);
    doc.set_title("EPA CID Evidence Collection Form");
    doc.set_minimal_conformance();

    if ev_data.collected_items.is_empty() {
        // Render one blank form
        add_collection_form_page(&mut doc, report, ev_data, None, 1)?;
    } else {
        // One form page per collected item
        for (i, item) in ev_data.collected_items.iter().enumerate() {
            if i > 0 {
                doc.push(Break::new(4.0)); // page break approximation
            }
            add_collection_form_page(&mut doc, report, ev_data, Some(item), i + 1)?;
        }
    }

    doc.render_to_file(output_path)
        .map_err(|e| ReportError::Pdf(e.to_string()))?;

    Ok(())
}

// =============================================================================
// Full form page for one collected item
// =============================================================================

fn add_collection_form_page(
    doc: &mut Document,
    report: &ForensicReport,
    ev_data: &EvidenceCollectionData,
    item: Option<&CollectedItem>,
    page_num: usize,
) -> ReportResult<()> {
    // Header
    add_form_header(doc)?;

    // Section 1: Initial Collection
    add_initial_collection_section(doc, ev_data, item, page_num)?;

    // Section 2-3: Date/Time + System Accuracy
    add_datetime_section(doc, ev_data, item)?;

    // Section 4: Device Type Information
    add_device_type_section(doc, item)?;

    // Section 5: Brand, Model, Serial Number
    add_device_identity_section(doc, item)?;

    // Section 6: Location Found Data
    add_location_section(doc, item)?;

    // Section 7: Total Devices / Primary User
    add_devices_user_section(doc, ev_data)?;

    // Section 8: Forensic Image Data
    add_forensic_image_section(doc, report, item, page_num)?;

    // Section 9: Tool Used, Image Size
    add_tool_section(doc)?;

    // Section 10: Hard Drive Info
    add_hard_drive_section(doc, item)?;

    // Notes
    add_notes_section(doc, item, ev_data)?;

    // Footer
    add_form_footer(doc, page_num)?;

    Ok(())
}

// =============================================================================
// Form sections
// =============================================================================

fn add_form_header(doc: &mut Document) -> ReportResult<()> {
    doc.push(
        Paragraph::new("EPA CID Computer Forensics Laboratory")
            .aligned(Alignment::Center)
            .styled(style::Style::new().bold().with_font_size(12)),
    );
    doc.push(
        Paragraph::new("Evidence Collection Form")
            .aligned(Alignment::Center)
            .styled(style::Style::new().bold().with_font_size(14)),
    );
    doc.push(Break::new(0.5));
    Ok(())
}

fn add_initial_collection_section(
    doc: &mut Document,
    ev_data: &EvidenceCollectionData,
    item: Option<&CollectedItem>,
    page_num: usize,
) -> ReportResult<()> {
    let mut table = TableLayout::new(vec![2, 2, 2]);
    table.set_cell_decorator(genpdf::elements::FrameCellDecorator::new(true, true, false));

    let img_num = item.map(|i| i.item_number.as_str()).unwrap_or("");

    // Use per-item officer if available, otherwise fall back to header
    let officer = item
        .and_then(|i| i.item_collecting_officer.as_deref())
        .filter(|s| !s.is_empty())
        .unwrap_or(&ev_data.collecting_officer);

    // Extract initials from collecting officer name
    let initials: String = officer
        .split_whitespace()
        .filter_map(|w| w.chars().next())
        .collect();

    table
        .row()
        .element(label_value("Team Lead Initials:", &initials))
        .element(label_value("CoC#:", ""))
        .element(label_value(&format!("Img# {}", page_num), img_num))
        .push()
        .map_err(|e| ReportError::Pdf(e.to_string()))?;

    doc.push(table);
    doc.push(Break::new(0.2));
    Ok(())
}

fn add_datetime_section(
    doc: &mut Document,
    ev_data: &EvidenceCollectionData,
    item: Option<&CollectedItem>,
) -> ReportResult<()> {
    // Use per-item datetime if available, otherwise fall back to header
    let collection_dt = item
        .and_then(|i| i.item_collection_datetime.as_deref())
        .filter(|s| !s.is_empty())
        .unwrap_or(&ev_data.collection_date);
    let system_dt = item
        .and_then(|i| i.item_system_datetime.as_deref())
        .filter(|s| !s.is_empty())
        .or(ev_data.system_date_time.as_deref())
        .unwrap_or("");

    let mut table = TableLayout::new(vec![2, 2, 2]);
    table.set_cell_decorator(genpdf::elements::FrameCellDecorator::new(true, true, false));

    table
        .row()
        .element(label_value("Actual Date:", collection_dt))
        .element(label_value("Actual Time (24hr):", ""))
        .element(label_value("System Accurate: [ ] Yes  [ ] No", ""))
        .push()
        .map_err(|e| ReportError::Pdf(e.to_string()))?;

    doc.push(table);

    // System Date/Time row
    let mut sys_table = TableLayout::new(vec![2, 2]);
    sys_table.set_cell_decorator(genpdf::elements::FrameCellDecorator::new(true, true, false));
    sys_table
        .row()
        .element(label_value("System Date:", system_dt))
        .element(label_value("System Time:", ""))
        .push()
        .map_err(|e| ReportError::Pdf(e.to_string()))?;
    doc.push(sys_table);

    doc.push(Break::new(0.2));
    Ok(())
}

fn add_device_type_section(doc: &mut Document, item: Option<&CollectedItem>) -> ReportResult<()> {
    doc.push(
        Paragraph::new("Device Type Information (circle applicable):")
            .styled(style::Style::new().bold().with_font_size(9)),
    );

    // Use device_type (new) with fallback to item_type (legacy)
    let device_type = item
        .map(|i| {
            if !i.device_type.is_empty() {
                i.device_type.as_str()
            } else {
                i.item_type.as_str()
            }
        })
        .unwrap_or("");

    // Device types - render in a grid-like layout (4 per row)
    let mut layout = LinearLayout::vertical();
    for chunk in DEVICE_TYPES.chunks(4) {
        let line: String = chunk
            .iter()
            .map(|dt| {
                let selected = device_type.to_lowercase().contains(&dt.to_lowercase());
                if selected {
                    format!("[{}]  ", dt)
                } else {
                    format!(" {}   ", dt)
                }
            })
            .collect();
        layout.push(Paragraph::new(line).styled(style::Style::new().with_font_size(9)));
    }
    doc.push(layout);

    // Interface types
    doc.push(Break::new(0.1));
    let iface = item
        .and_then(|i| i.storage_interface.as_deref())
        .unwrap_or("");
    let if_line: String = INTERFACE_TYPES
        .iter()
        .map(|it| {
            let selected = iface.to_lowercase().contains(&it.to_lowercase());
            if selected {
                format!("[{}]  ", it)
            } else {
                format!(" {}  ", it)
            }
        })
        .collect();
    doc.push(
        Paragraph::new(format!("Interface: {}", if_line))
            .styled(style::Style::new().with_font_size(9)),
    );

    doc.push(Break::new(0.2));
    Ok(())
}

fn add_device_identity_section(
    doc: &mut Document,
    item: Option<&CollectedItem>,
) -> ReportResult<()> {
    let mut table = TableLayout::new(vec![1, 1, 1]);
    table.set_cell_decorator(genpdf::elements::FrameCellDecorator::new(true, true, false));

    let (brand, model, sn) = match item {
        Some(i) => (
            i.brand.as_deref().or(i.make.as_deref()).unwrap_or(""),
            i.model.as_deref().unwrap_or(""),
            i.serial_number.as_deref().unwrap_or(""),
        ),
        None => ("", "", ""),
    };

    table
        .row()
        .element(label_value("Brand:", brand))
        .element(label_value("Model:", model))
        .element(label_value("S/N:", sn))
        .push()
        .map_err(|e| ReportError::Pdf(e.to_string()))?;

    doc.push(table);
    doc.push(Break::new(0.2));
    Ok(())
}

fn add_location_section(doc: &mut Document, item: Option<&CollectedItem>) -> ReportResult<()> {
    doc.push(
        Paragraph::new("Location Found Data:").styled(style::Style::new().bold().with_font_size(9)),
    );

    let mut table = TableLayout::new(vec![1, 1, 1, 1]);
    table.set_cell_decorator(genpdf::elements::FrameCellDecorator::new(true, true, false));

    // Use structured fields (new) with fallback to found_location (legacy)
    let (bldg, room, subloc) = match item {
        Some(i) if i.building.is_some() || i.room.is_some() || i.location_other.is_some() => (
            i.building.as_deref().unwrap_or(""),
            i.room.as_deref().unwrap_or(""),
            i.location_other.as_deref().unwrap_or(""),
        ),
        Some(i) if !i.found_location.is_empty() => {
            // Legacy: parse "Bldg / Room / Subloc" format
            let parts: Vec<&str> = i.found_location.split('/').map(|s| s.trim()).collect();
            (
                *parts.first().unwrap_or(&i.found_location.as_str()),
                *parts.get(1).unwrap_or(&""),
                *parts.get(2).unwrap_or(&""),
            )
        }
        _ => ("", "", ""),
    };

    let item_num = item.map(|i| i.item_number.as_str()).unwrap_or("");

    table
        .row()
        .element(label_value("Bldg:", bldg))
        .element(label_value("Room:", room))
        .element(label_value("Subloc:", subloc))
        .element(label_value("ITM:", item_num))
        .push()
        .map_err(|e| ReportError::Pdf(e.to_string()))?;

    doc.push(table);
    doc.push(Break::new(0.2));
    Ok(())
}

fn add_devices_user_section(
    doc: &mut Document,
    ev_data: &EvidenceCollectionData,
) -> ReportResult<()> {
    let mut table = TableLayout::new(vec![1, 2, 1]);
    table.set_cell_decorator(genpdf::elements::FrameCellDecorator::new(true, true, false));

    let total = ev_data.collected_items.len().to_string();
    table
        .row()
        .element(label_value("Total Devices:", &total))
        .element(label_value("Primary User/Use:", ""))
        .element(label_value("Cap:", ""))
        .push()
        .map_err(|e| ReportError::Pdf(e.to_string()))?;

    doc.push(table);
    doc.push(Break::new(0.2));
    Ok(())
}

fn add_forensic_image_section(
    doc: &mut Document,
    report: &ForensicReport,
    item: Option<&CollectedItem>,
    img_num: usize,
) -> ReportResult<()> {
    doc.push(
        Paragraph::new("Forensic Image Data:").styled(style::Style::new().bold().with_font_size(9)),
    );

    // Row 1: Case#, Site, Last 6 of Container S/N, IMG#
    let mut row1 = TableLayout::new(vec![2, 1, 2, 1]);
    row1.set_cell_decorator(genpdf::elements::FrameCellDecorator::new(true, true, false));

    let sn = item.and_then(|i| i.serial_number.as_deref()).unwrap_or("");
    let last6 = if sn.len() >= 6 {
        &sn[sn.len() - 6..]
    } else {
        sn
    };

    row1.row()
        .element(label_value("Case#:", &report.case_info.case_number))
        .element(label_value("Site:", ""))
        .element(label_value("Last 6 of Container S/N:", last6))
        .element(label_value("IMG#:", &img_num.to_string()))
        .push()
        .map_err(|e| ReportError::Pdf(e.to_string()))?;
    doc.push(row1);

    // Row 2: Image Format (checkboxes) + Image Method (checkboxes)
    let selected_format = item.and_then(|i| i.image_format.as_deref()).unwrap_or("");
    let format_line: String = IMAGE_FORMATS
        .iter()
        .map(|f| {
            let selected = selected_format.to_lowercase().contains(&f.to_lowercase());
            if selected {
                format!("[{}]  ", f)
            } else {
                format!(" {}  ", f)
            }
        })
        .collect();
    doc.push(
        Paragraph::new(format!("Image Format: {}", format_line))
            .styled(style::Style::new().with_font_size(9)),
    );

    let selected_method = item
        .and_then(|i| i.acquisition_method.as_deref())
        .unwrap_or("");
    let method_line: String = IMAGE_METHODS
        .iter()
        .map(|m| {
            let selected = selected_method.to_lowercase().contains(&m.to_lowercase());
            if selected {
                format!("[{}]  ", m)
            } else {
                format!(" {}  ", m)
            }
        })
        .collect();
    doc.push(
        Paragraph::new(format!("Image Method: {}", method_line))
            .styled(style::Style::new().with_font_size(8)),
    );

    doc.push(Break::new(0.2));
    Ok(())
}

fn add_tool_section(doc: &mut Document) -> ReportResult<()> {
    let mut table = TableLayout::new(vec![2, 1]);
    table.set_cell_decorator(genpdf::elements::FrameCellDecorator::new(true, true, false));

    table
        .row()
        .element(label_value("Tool Used for Imaging:", ""))
        .element(label_value("Image Size:", ""))
        .push()
        .map_err(|e| ReportError::Pdf(e.to_string()))?;

    doc.push(table);
    doc.push(Break::new(0.2));
    Ok(())
}

fn add_hard_drive_section(doc: &mut Document, item: Option<&CollectedItem>) -> ReportResult<()> {
    doc.push(
        Paragraph::new("Hard Drive Info:").styled(style::Style::new().bold().with_font_size(9)),
    );

    let mut table = TableLayout::new(vec![2, 2]);
    table.set_cell_decorator(genpdf::elements::FrameCellDecorator::new(true, true, false));

    let (mfg_mdl, sn) = match item {
        Some(i) => {
            let mfg = i.make.as_deref().unwrap_or("");
            let mdl = i.model.as_deref().unwrap_or("");
            let combined = if mfg.is_empty() && mdl.is_empty() {
                String::new()
            } else {
                format!("{} {}", mfg, mdl).trim().to_string()
            };
            (
                combined,
                i.serial_number.as_deref().unwrap_or("").to_string(),
            )
        }
        None => (String::new(), String::new()),
    };

    table
        .row()
        .element(label_value("Mfg/Mdl:", &mfg_mdl))
        .element(label_value("S/N:", &sn))
        .push()
        .map_err(|e| ReportError::Pdf(e.to_string()))?;

    doc.push(table);
    doc.push(Break::new(0.2));
    Ok(())
}

fn add_notes_section(
    doc: &mut Document,
    item: Option<&CollectedItem>,
    ev_data: &EvidenceCollectionData,
) -> ReportResult<()> {
    doc.push(Paragraph::new("Notes:").styled(style::Style::new().bold().with_font_size(9)));

    let mut notes_parts = Vec::new();
    if let Some(ref doc_notes) = ev_data.documentation_notes {
        if !doc_notes.is_empty() {
            notes_parts.push(doc_notes.clone());
        }
    }
    if let Some(ref conditions) = ev_data.conditions {
        if !conditions.is_empty() {
            notes_parts.push(format!("Conditions: {}", conditions));
        }
    }
    if let Some(i) = item {
        if let Some(ref n) = i.notes {
            if !n.is_empty() {
                notes_parts.push(n.clone());
            }
        }
        if !i.packaging.is_empty() {
            notes_parts.push(format!("Packaging: {}", i.packaging));
        }
        if let Some(ref sn) = i.storage_notes {
            if !sn.is_empty() {
                notes_parts.push(format!("Storage Notes: {}", sn));
            }
        }
        if let Some(ref imei) = i.imei {
            if !imei.is_empty() {
                notes_parts.push(format!("IMEI: {}", imei));
            }
        }
        if let Some(ref other_id) = i.other_identifiers {
            if !other_id.is_empty() {
                notes_parts.push(format!("Other IDs: {}", other_id));
            }
        }
        if !i.photo_refs.is_empty() {
            notes_parts.push(format!("Photo Refs: {}", i.photo_refs.join(", ")));
        }
    }

    let notes_text = if notes_parts.is_empty() {
        " ".to_string()
    } else {
        notes_parts.join("\n")
    };

    let mut notes_box = TableLayout::new(vec![1]);
    notes_box.set_cell_decorator(genpdf::elements::FrameCellDecorator::new(true, true, false));
    notes_box
        .row()
        .element(Text::new(notes_text).styled(style::Style::new().with_font_size(8)))
        .push()
        .map_err(|e| ReportError::Pdf(e.to_string()))?;
    // Add empty rows for writing space
    for _ in 0..3 {
        notes_box
            .row()
            .element(Text::new(" ").styled(style::Style::new().with_font_size(8)))
            .push()
            .map_err(|e| ReportError::Pdf(e.to_string()))?;
    }

    doc.push(notes_box);
    doc.push(Break::new(0.2));
    Ok(())
}

fn add_form_footer(doc: &mut Document, page_num: usize) -> ReportResult<()> {
    doc.push(Break::new(0.5));

    // Authorization info
    doc.push(
        Paragraph::new(format!(
            "Collecting Officer: ________________    Date: ________    Page {} of __",
            page_num
        ))
        .styled(style::Style::new().with_font_size(8)),
    );

    doc.push(Break::new(0.3));
    doc.push(
        Paragraph::new(
            "EPA CID Computer Forensics Laboratory Evidence Collection Form  v.06-02-2021",
        )
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
