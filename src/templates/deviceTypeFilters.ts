// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Device-type-based option filter maps.
 *
 * Each map defines which option values are relevant for each device_type value.
 * Used by `getOptions()` in `useFormTemplate.ts` when a field has `options_filter`.
 *
 * "other" and "n_a" are always included automatically — do NOT add them here.
 * Group headers (disabled _group_* items) are included automatically when at
 * least one of their children is in the filtered set.
 */

import type { InlineOption } from "./types";

// =============================================================================
// FILTER MAP TYPE
// =============================================================================

/** Mapping from a field value to allowed option values */
export type FilterMap = Record<string, string[]>;

// =============================================================================
// STORAGE INTERFACE FILTER (by device_type)
// =============================================================================

const deviceToStorageInterface: FilterMap = {
  // ── Most Common ──
  laptop:           ["nvme_m2", "sata_m2", "sata", "usb_3", "usb_c", "emmc"],
  desktop_computer: ["nvme_m2", "sata", "sata_m2", "pcie", "usb_3", "usb_c", "ide_pata"],
  mobile_phone:     ["usb_c", "lightning", "wifi_adb", "bluetooth", "ufs", "emmc"],
  tablet:           ["usb_c", "lightning", "wifi_adb", "bluetooth", "ufs", "emmc"],
  external_hdd:     ["usb_3", "usb_c", "usb_2", "esata", "firewire"],
  external_ssd:     ["usb_3", "usb_c", "usb_2", "esata"],
  usb_flash_drive:  ["usb_3", "usb_2", "usb_c"],
  server:           ["nvme_m2", "sata", "scsi_sas", "pcie", "raid", "fc_san", "network_nas"],

  // ── Mobile & Wearable ──
  wearable:         ["bluetooth", "wifi_adb", "usb_c", "usb_2"],
  gps_device:       ["usb_2", "usb_3", "sd_card"],
  sat_phone:        ["usb_2", "bluetooth"],
  pager:            ["usb_2"],

  // ── Storage Media ──
  memory_card:      ["sd_card"],
  internal_hdd:     ["sata", "ide_pata", "scsi_sas"],
  internal_ssd:     ["nvme_m2", "sata_m2", "sata"],
  nvme_drive:       ["nvme_m2", "pcie"],
  optical_disc:     ["sata", "ide_pata", "usb_3", "usb_2"],
  tape_media:       ["scsi_sas", "usb_3"],
  floppy_disk:      ["ide_pata", "usb_2"],

  // ── Network & Surveillance ──
  network_device:   ["network_nas"],
  nvr_dvr:          ["sata", "network_nas", "usb_3"],
  ip_camera:        ["network_nas", "sd_card", "wifi_adb"],
  access_point:     ["network_nas"],
  firewall:         ["network_nas", "sata"],

  // ── Specialty Devices ──
  gaming_console:   ["sata", "nvme_m2", "usb_3"],
  drone:            ["sd_card", "usb_c", "usb_3"],
  camera:           ["sd_card", "usb_3", "usb_2"],
  vehicle_infotainment: ["usb_3", "sd_card", "emmc"],
  pos_terminal:     ["sata", "emmc", "sd_card", "usb_3"],
  printer_mfp:      ["network_nas", "usb_3", "sd_card"],
  iot_device:       ["emmc", "wifi_adb", "bluetooth", "sd_card", "usb_3"],
  smart_speaker:    ["wifi_adb", "bluetooth", "emmc"],
  medical_device:   ["usb_3", "sd_card", "network_nas", "sata"],

  // ── Virtual & Cloud ──
  virtual_machine:       ["n_a"],
  cloud_account:         ["n_a"],
  email_account:         ["n_a"],
  social_media_account:  ["n_a"],
};

// =============================================================================
// FORM FACTOR FILTER (by device_type)
// =============================================================================

const deviceToFormFactor: FilterMap = {
  // ── Most Common ──
  laptop:           ["laptop_clamshell", "convertible_2in1"],
  desktop_computer: ["tower", "mini_tower", "sff", "mini_pc", "all_in_one"],
  mobile_phone:     ["smartphone"],
  tablet:           ["tablet_form", "convertible_2in1"],
  external_hdd:     ["portable_enclosure", "3_5_inch", "2_5_inch"],
  external_ssd:     ["portable_enclosure", "2_5_inch"],
  usb_flash_drive:  ["usb_stick"],
  server:           ["tower", "rack_1u", "rack_2u", "rack_4u", "blade_server"],

  // ── Mobile & Wearable ──
  wearable:         ["watch_band"],
  gps_device:       ["handheld"],
  sat_phone:        ["handheld"],
  pager:            ["handheld"],

  // ── Storage Media ──
  memory_card:      ["full_size_sd", "micro_sd", "compact_flash"],
  internal_hdd:     ["3_5_inch", "2_5_inch"],
  internal_ssd:     ["2_5_inch", "m2_2280", "m2_2230", "msata"],
  nvme_drive:       ["m2_2280", "m2_2230"],
  optical_disc:     ["disc"],
  tape_media:       ["tape_cartridge"],
  floppy_disk:      ["floppy_3_5"],

  // ── Network & Surveillance ──
  network_device:   ["rack_1u", "set_top_box", "embedded_board"],
  nvr_dvr:          ["rack_1u", "set_top_box"],
  ip_camera:        ["body_mount", "dashboard_mount", "embedded_board"],
  access_point:     ["embedded_board"],
  firewall:         ["rack_1u", "set_top_box"],

  // ── Specialty Devices ──
  gaming_console:   ["set_top_box", "handheld"],
  drone:            ["quadcopter"],
  camera:           ["handheld", "body_mount", "dashboard_mount"],
  vehicle_infotainment: ["dashboard_mount"],
  pos_terminal:     ["set_top_box", "embedded_board"],
  printer_mfp:      ["set_top_box"],
  iot_device:       ["embedded_board", "set_top_box"],
  smart_speaker:    ["set_top_box"],
  medical_device:   ["handheld", "embedded_board", "set_top_box"],

  // ── Virtual & Cloud ──
  virtual_machine:       ["n_a"],
  cloud_account:         ["n_a"],
  email_account:         ["n_a"],
  social_media_account:  ["n_a"],
};

// =============================================================================
// ACQUISITION METHOD FILTER (by device_type)
// =============================================================================

/** Computer-class acquisition methods */
const COMPUTER_METHODS = [
  "physical", "logical_partition", "logical_file_folder", "file_system",
  "native_file", "targeted_collection",
  "ftk_physical", "ftk_logical", "ftk_contents", "ftk_memory",
  "axiom_acquire", "axiom_quick",
  "macquisition", "paladin", "caine", "encase_acquire",
];

/** Mobile-class acquisition methods */
const MOBILE_METHODS = [
  "physical", "logical_file_folder", "file_system", "consent_download",
  "ufed_physical", "ufed_file_system", "ufed_logical", "ufed_cloud", "ufed_chinex", "ufed_graykey",
  "axiom_acquire", "axiom_quick", "axiom_cloud",
  "graykey_full", "graykey_partial", "graykey_logical",
  "inseyets_physical", "inseyets_file_system", "inseyets_logical",
  "xry_physical", "xry_logical",
  "oxygen_physical", "oxygen_logical",
  "mobiledit_extraction",
  "chip_off", "jtag", "isp", "bootloader",
];

/** Storage media acquisition methods */
const STORAGE_METHODS = [
  "physical", "logical_partition", "logical_file_folder", "file_system", "native_file",
  "ftk_physical", "ftk_logical", "ftk_contents",
  "encase_acquire",
];

/** Network/surveillance device methods */
const NETWORK_METHODS = [
  "logical_file_folder", "file_system", "native_file", "targeted_collection",
  "ftk_logical", "ftk_contents",
];

/** Cloud/virtual methods */
const CLOUD_METHODS = [
  "logical_file_folder", "native_file", "targeted_collection", "consent_download",
  "axiom_cloud", "ufed_cloud",
];

const deviceToAcquisitionMethod: FilterMap = {
  // ── Most Common ──
  laptop:           COMPUTER_METHODS,
  desktop_computer: COMPUTER_METHODS,
  mobile_phone:     MOBILE_METHODS,
  tablet:           MOBILE_METHODS,
  external_hdd:     STORAGE_METHODS,
  external_ssd:     STORAGE_METHODS,
  usb_flash_drive:  STORAGE_METHODS,
  server:           COMPUTER_METHODS,

  // ── Mobile & Wearable ──
  wearable:         [...MOBILE_METHODS],
  gps_device:       ["physical", "logical_file_folder", "native_file", "ftk_physical", "ftk_logical", "chip_off"],
  sat_phone:        [...MOBILE_METHODS],
  pager:            ["physical", "logical_file_folder", "chip_off", "jtag"],

  // ── Storage Media ──
  memory_card:      STORAGE_METHODS,
  internal_hdd:     STORAGE_METHODS,
  internal_ssd:     STORAGE_METHODS,
  nvme_drive:       STORAGE_METHODS,
  optical_disc:     ["physical", "logical_file_folder", "native_file", "ftk_physical", "ftk_logical"],
  tape_media:       ["physical", "logical_file_folder", "native_file", "ftk_physical"],
  floppy_disk:      ["physical", "logical_file_folder", "native_file", "ftk_physical"],

  // ── Network & Surveillance ──
  network_device:   NETWORK_METHODS,
  nvr_dvr:          [...NETWORK_METHODS, "physical", "ftk_physical", "encase_acquire"],
  ip_camera:        NETWORK_METHODS,
  access_point:     NETWORK_METHODS,
  firewall:         NETWORK_METHODS,

  // ── Specialty Devices ──
  gaming_console:   ["physical", "logical_file_folder", "file_system", "ftk_physical", "ftk_logical", "chip_off"],
  drone:            ["logical_file_folder", "native_file", "file_system", "ftk_logical", "ftk_contents", "chip_off"],
  camera:           ["logical_file_folder", "native_file", "file_system", "ftk_logical", "ftk_contents"],
  vehicle_infotainment: ["physical", "logical_file_folder", "file_system", "axiom_vehicle", "chip_off", "jtag"],
  pos_terminal:     ["physical", "logical_file_folder", "file_system", "ftk_physical", "ftk_logical", "chip_off"],
  printer_mfp:      NETWORK_METHODS,
  iot_device:       ["physical", "logical_file_folder", "file_system", "chip_off", "jtag", "isp"],
  smart_speaker:    ["physical", "logical_file_folder", "file_system", "chip_off", "jtag"],
  medical_device:   ["physical", "logical_file_folder", "file_system", "native_file", "ftk_physical", "ftk_logical"],

  // ── Virtual & Cloud ──
  virtual_machine:       CLOUD_METHODS,
  cloud_account:         CLOUD_METHODS,
  email_account:         CLOUD_METHODS,
  social_media_account:  CLOUD_METHODS,
};

// =============================================================================
// FILTER MAP REGISTRY
// =============================================================================

/** All available filter maps, keyed by filter_map ID */
const FILTER_MAPS: Record<string, FilterMap> = {
  device_to_storage_interface: deviceToStorageInterface,
  device_to_form_factor: deviceToFormFactor,
  device_to_acquisition_method: deviceToAcquisitionMethod,
};

/**
 * Look up a filter map by its ID.
 * Returns undefined if the map doesn't exist.
 */
export function getFilterMap(filterMapId: string): FilterMap | undefined {
  return FILTER_MAPS[filterMapId];
}

/**
 * Filter an options array based on a set of allowed values.
 *
 * - Options whose value is in `allowed` are kept.
 * - "other" and "n_a" are always kept.
 * - Group headers (disabled _group_* items) are kept only if at least
 *   one of their child options is in the filtered set.
 * - If `allowed` is empty, returns all options (unfiltered).
 */
export function filterOptions(options: InlineOption[], allowed: string[]): InlineOption[] {
  if (allowed.length === 0) return options;

  const allowedSet = new Set(allowed);
  // "other" and "n_a" are always available
  allowedSet.add("other");
  allowedSet.add("n_a");

  const result: InlineOption[] = [];
  let i = 0;

  while (i < options.length) {
    const opt = options[i];

    // Group header — include only if it has at least one visible child
    if (opt.disabled && opt.value.startsWith("_group_")) {
      const groupHeader = opt;
      const groupChildren: InlineOption[] = [];
      let j = i + 1;
      while (j < options.length && !(options[j].disabled && options[j].value.startsWith("_group_"))) {
        if (allowedSet.has(options[j].value)) {
          groupChildren.push(options[j]);
        }
        j++;
      }
      if (groupChildren.length > 0) {
        result.push(groupHeader);
        result.push(...groupChildren);
      }
      i = j;
    } else if (allowedSet.has(opt.value)) {
      result.push(opt);
      i++;
    } else {
      i++;
    }
  }

  return result;
}
