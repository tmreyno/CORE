// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Evidence collection form types for the EPA CID Computer Forensics Laboratory.

use serde::{Deserialize, Serialize};

/// EPA CID Computer Forensics Laboratory Evidence Collection Form data
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct EvidenceCollectionData {
    /// Collection date
    pub collection_date: String,
    /// System date/time if different from actual collection time
    pub system_date_time: Option<String>,
    /// Collection location (legacy — prefer per-item building/room fields)
    pub collection_location: String,
    /// Collecting officer/examiner
    pub collecting_officer: String,
    /// Authorization (warrant/consent)
    pub authorization: String,
    /// Authorization date
    pub authorization_date: Option<String>,
    /// Authorizing authority (judge name)
    pub authorizing_authority: Option<String>,
    /// Witnesses present
    #[serde(default)]
    pub witnesses: Vec<String>,
    /// Items collected with collection-specific details
    #[serde(default)]
    pub collected_items: Vec<CollectedItem>,
    /// Photography/documentation notes
    pub documentation_notes: Option<String>,
    /// Environmental conditions
    pub conditions: Option<String>,
}

/// Individual collected item for Evidence Collection Form
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CollectedItem {
    /// Internal UI identifier
    pub id: String,
    /// Item number (sequential)
    pub item_number: String,
    /// Description
    pub description: String,

    // --- Per-Item Collection Info ---
    /// Collection date/time for this specific item (may differ from header)
    pub item_collection_datetime: Option<String>,
    /// Device system clock date/time at moment of collection
    pub item_system_datetime: Option<String>,
    /// Collecting officer for this item (overrides header if different)
    pub item_collecting_officer: Option<String>,
    /// Authorization for this specific item (overrides header if different)
    pub item_authorization: Option<String>,

    // --- Device Identification ---
    /// Device type (desktop_computer, laptop, mobile_phone, tablet, server, etc.)
    pub device_type: String,
    /// Custom device type if "Other" selected
    pub device_type_other: Option<String>,
    /// Storage interface type (sata, usb, ide, nvme_m2, raid, etc.)
    pub storage_interface: Option<String>,
    /// Custom storage interface if "Other" selected
    pub storage_interface_other: Option<String>,
    /// Brand / Manufacturer
    pub brand: Option<String>,
    /// Make
    pub make: Option<String>,
    /// Model
    pub model: Option<String>,
    /// Color
    pub color: Option<String>,
    /// Serial number
    pub serial_number: Option<String>,
    /// IMEI number (mobile devices)
    pub imei: Option<String>,
    /// Other device identifiers (asset tag, MAC address, etc.)
    pub other_identifiers: Option<String>,

    // --- Location ---
    /// Building where evidence was located
    pub building: Option<String>,
    /// Room where evidence was located
    pub room: Option<String>,
    /// Other location details
    pub location_other: Option<String>,
    /// Location found/collected from (legacy combined field)
    pub found_location: String,

    // --- Forensic Image ---
    /// Forensic image/container format (ad1, e01, l01, dd, etc.)
    pub image_format: Option<String>,
    /// Custom image format if "Other" selected
    pub image_format_other: Option<String>,
    /// Acquisition method (logical_file_folder, physical, logical_partition, etc.)
    pub acquisition_method: Option<String>,
    /// Custom acquisition method if "Other" selected
    pub acquisition_method_other: Option<String>,

    // --- Condition & Packaging ---
    /// Evidence type (legacy — prefer device_type)
    pub item_type: String,
    /// Condition at collection
    pub condition: String,
    /// How it was packaged
    pub packaging: String,

    // --- Additional Info ---
    /// Other HDD/SSD/USB/NVMe information
    pub storage_notes: Option<String>,
    /// Notes — passwords, BitLocker, encryption, phone details, etc.
    pub notes: Option<String>,
    /// Photo reference numbers
    #[serde(default)]
    pub photo_refs: Vec<String>,
}
