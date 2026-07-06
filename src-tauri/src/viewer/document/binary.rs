// =============================================================================
// CORE-FFX - Forensic File Explorer
// Binary Analyzer - PE/ELF/Mach-O analysis for forensic investigation
// =============================================================================

use goblin::Object;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::{self, File};
use std::io::{Read, Take};
use std::path::Path;

use super::error::{DocumentError, DocumentResult};

const MAX_BINARY_ANALYSIS_BYTES: u64 = 512 * 1024 * 1024;
const MAX_BINARY_DETECT_PREFIX_BYTES: usize = 4096;
const MAX_BINARY_IMPORT_LIBRARIES: usize = 512;
const MAX_BINARY_IMPORT_FUNCTIONS_PER_LIBRARY: usize = 2048;
const MAX_BINARY_EXPORTS: usize = 4096;
const MAX_BINARY_SECTIONS: usize = 2048;
const MAX_BINARY_STRINGS: usize = 2048;
const MIN_BINARY_STRING_CHARS: usize = 4;
const MAX_BINARY_STRING_CHARS: usize = 256;
const MAX_PE_VERSION_INFO_FIELDS: usize = 32;
const MAX_PE_VERSION_INFO_VALUE_CHARS: usize = 512;

const PE_VERSION_INFO_KEYS: &[&str] = &[
    "CompanyName",
    "FileDescription",
    "FileVersion",
    "InternalName",
    "OriginalFilename",
    "ProductName",
    "ProductVersion",
    "LegalCopyright",
    "LegalTrademarks",
    "PrivateBuild",
    "SpecialBuild",
    "Comments",
];

#[derive(Debug, Default)]
struct ImportAccumulator {
    function_count: usize,
    functions: Vec<String>,
}

fn ensure_binary_analysis_size(size: u64) -> DocumentResult<()> {
    if size > MAX_BINARY_ANALYSIS_BYTES {
        return Err(binary_analysis_too_large_error(
            size,
            MAX_BINARY_ANALYSIS_BYTES,
        ));
    }
    Ok(())
}

fn binary_analysis_too_large_error(size: u64, max_bytes: u64) -> DocumentError {
    DocumentError::Parse(format!(
        "Binary file too large for full analysis ({:.1} MiB, max {} MiB)",
        size as f64 / (1024.0 * 1024.0),
        max_bytes / (1024 * 1024)
    ))
}

fn read_binary_prefix(path: &Path, max_bytes: usize) -> DocumentResult<Vec<u8>> {
    let mut file = File::open(path)?;
    let total_size = file.metadata()?.len();
    let to_read = total_size.min(max_bytes as u64) as usize;
    let mut data = vec![0u8; to_read];
    file.read_exact(&mut data)?;
    Ok(data)
}

fn read_binary_analysis_with_limit<R: Read>(reader: R, max_bytes: u64) -> DocumentResult<Vec<u8>> {
    let read_limit = max_bytes
        .checked_add(1)
        .ok_or_else(|| DocumentError::Parse("Binary analysis read limit overflow".to_string()))?;
    let mut limited: Take<R> = reader.take(read_limit);
    let mut data = Vec::new();
    limited.read_to_end(&mut data)?;
    if data.len() as u64 > max_bytes {
        return Err(binary_analysis_too_large_error(
            data.len() as u64,
            max_bytes,
        ));
    }
    Ok(data)
}

/// Binary format detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BinaryFormat {
    PE32,
    PE64,
    ELF32,
    ELF64,
    MachO32,
    MachO64,
    MachOFat,
    Unknown,
}

/// Import information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportInfo {
    pub library: String,
    pub functions: Vec<String>,
    pub function_count: usize,
}

/// Export information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportInfo {
    pub name: String,
    pub ordinal: Option<u32>,
    pub address: u64,
}

/// Section information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionInfo {
    pub name: String,
    pub virtual_address: u64,
    pub virtual_size: u64,
    pub raw_size: u64,
    pub characteristics: String,
    pub characteristics_detail: Vec<String>,
    pub entropy: Option<f64>,
}

/// Linux kernel module information extracted from ELF `.ko` module strings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LinuxModuleInfo {
    pub detected: bool,
    pub names: Vec<String>,
    pub versions: Vec<String>,
    pub vermagic: Vec<String>,
    pub licenses: Vec<String>,
    pub authors: Vec<String>,
    pub descriptions: Vec<String>,
    pub aliases: Vec<String>,
    pub dependencies: Vec<String>,
    pub firmware: Vec<String>,
    pub signers: Vec<String>,
    pub signatures: Vec<String>,
}

/// Binary analysis result (read-only)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryInfo {
    pub path: String,
    pub format: BinaryFormat,
    pub architecture: String,
    pub is_64bit: bool,
    pub entry_point: Option<u64>,
    pub imports: Vec<ImportInfo>,
    pub exports: Vec<ExportInfo>,
    pub sections: Vec<SectionInfo>,
    pub strings: Vec<String>,
    pub file_size: u64,
    // PE specific
    pub pe_timestamp: Option<u32>,
    pub pe_checksum: Option<u32>,
    pub pe_subsystem: Option<String>,
    pub pe_linker_version: Option<String>,
    pub pe_os_version: Option<String>,
    pub pe_image_version: Option<String>,
    pub pe_subsystem_version: Option<String>,
    pub pe_image_base: Option<u64>,
    pub pe_section_alignment: Option<u32>,
    pub pe_file_alignment: Option<u32>,
    pub pe_size_of_image: Option<u32>,
    pub pe_size_of_headers: Option<u32>,
    pub pe_dll_characteristics: Option<String>,
    pub pe_dll_characteristics_detail: Vec<String>,
    pub pe_certificate_table_size: Option<u32>,
    pub pe_is_driver: bool,
    pub pe_driver_type: Option<String>,
    pub pe_driver_indicators: Vec<String>,
    pub pe_version_info: BTreeMap<String, String>,
    // Mach-O specific
    pub macho_cpu_type: Option<String>,
    pub macho_filetype: Option<String>,
    // Linux kernel module specific
    pub linux_module_info: Option<LinuxModuleInfo>,
    // Security indicators
    pub has_debug_info: bool,
    pub is_stripped: bool,
    pub has_code_signing: bool,
}

/// Analyze a binary file
pub fn analyze_binary(path: impl AsRef<Path>) -> DocumentResult<BinaryInfo> {
    let path = path.as_ref();
    ensure_binary_analysis_size(fs::metadata(path)?.len())?;
    let data = read_binary_analysis_with_limit(File::open(path)?, MAX_BINARY_ANALYSIS_BYTES)?;
    analyze_binary_bytes(path.to_string_lossy(), &data)
}

/// Analyze binary bytes read from any evidence source.
pub fn analyze_binary_bytes(
    source_id: impl Into<String>,
    data: &[u8],
) -> DocumentResult<BinaryInfo> {
    let source_id = source_id.into();
    ensure_binary_analysis_size(data.len() as u64)?;
    let file_size = data.len() as u64;
    let strings = extract_binary_strings(data);
    let obj = Object::parse(data)
        .map_err(|e| DocumentError::Parse(format!("Failed to parse binary: {}", e)))?;

    match obj {
        Object::PE(pe) => analyze_pe(pe, &source_id, data, file_size, strings),
        Object::Elf(elf) => analyze_elf(elf, &source_id, data, file_size, strings),
        Object::Mach(mach) => analyze_mach(mach, &source_id, data, file_size, strings),
        _ => Err(DocumentError::UnsupportedFormat(
            "Not a recognized binary format".to_string(),
        )),
    }
}

fn analyze_pe(
    pe: goblin::pe::PE,
    source_id: &str,
    data: &[u8],
    file_size: u64,
    strings: Vec<String>,
) -> DocumentResult<BinaryInfo> {
    let is_64bit = pe.is_64;
    let format = if is_64bit {
        BinaryFormat::PE64
    } else {
        BinaryFormat::PE32
    };

    let architecture = match pe.header.coff_header.machine {
        0x8664 => "x86_64".to_string(),
        0x14c => "i386".to_string(),
        0xaa64 => "ARM64".to_string(),
        m => format!("0x{:04x}", m),
    };

    // Imports - group by DLL
    let mut import_map: HashMap<String, ImportAccumulator> = HashMap::new();
    for imp in &pe.imports {
        push_limited_import(&mut import_map, imp.dll, &imp.name);
    }
    let imports = import_infos_from_accumulators(import_map);

    // Exports
    let exports: Vec<ExportInfo> = pe
        .exports
        .iter()
        .take(MAX_BINARY_EXPORTS)
        .filter_map(|exp| {
            exp.name.map(|name| ExportInfo {
                name: name.to_string(),
                ordinal: None,
                address: exp.rva as u64,
            })
        })
        .collect();

    // Sections
    let sections: Vec<SectionInfo> = pe
        .sections
        .iter()
        .take(MAX_BINARY_SECTIONS)
        .map(|sec| {
            let name = String::from_utf8_lossy(&sec.name)
                .trim_end_matches('\0')
                .to_string();
            SectionInfo {
                name,
                virtual_address: sec.virtual_address as u64,
                virtual_size: sec.virtual_size as u64,
                raw_size: sec.size_of_raw_data as u64,
                characteristics: format!("0x{:08x}", sec.characteristics),
                characteristics_detail: decode_pe_section_characteristics(sec.characteristics),
                entropy: section_entropy(
                    data,
                    sec.pointer_to_raw_data as u64,
                    sec.size_of_raw_data as u64,
                ),
            }
        })
        .collect();

    // Optional header info
    let (
        timestamp,
        checksum,
        subsystem,
        pe_linker_version,
        pe_os_version,
        pe_image_version,
        pe_subsystem_version,
        pe_image_base,
        pe_section_alignment,
        pe_file_alignment,
        pe_size_of_image,
        pe_size_of_headers,
        pe_dll_characteristics,
        pe_dll_characteristics_detail,
        pe_certificate_table_size,
    ) = if let Some(opt) = pe.header.optional_header {
        let sub = match opt.windows_fields.subsystem {
            1 => "Native",
            2 => "GUI",
            3 => "Console",
            _ => "Unknown",
        };
        let certificate_table = opt.data_directories.get_certificate_table();
        (
            Some(pe.header.coff_header.time_date_stamp),
            Some(opt.windows_fields.check_sum),
            Some(sub.to_string()),
            Some(format!(
                "{}.{}",
                opt.standard_fields.major_linker_version, opt.standard_fields.minor_linker_version
            )),
            Some(format!(
                "{}.{}",
                opt.windows_fields.major_operating_system_version,
                opt.windows_fields.minor_operating_system_version
            )),
            Some(format!(
                "{}.{}",
                opt.windows_fields.major_image_version, opt.windows_fields.minor_image_version
            )),
            Some(format!(
                "{}.{}",
                opt.windows_fields.major_subsystem_version,
                opt.windows_fields.minor_subsystem_version
            )),
            Some(opt.windows_fields.image_base),
            Some(opt.windows_fields.section_alignment),
            Some(opt.windows_fields.file_alignment),
            Some(opt.windows_fields.size_of_image),
            Some(opt.windows_fields.size_of_headers),
            Some(format!("0x{:04x}", opt.windows_fields.dll_characteristics)),
            decode_pe_dll_characteristics(opt.windows_fields.dll_characteristics),
            certificate_table.map(|directory| directory.size),
        )
    } else {
        (
            Some(pe.header.coff_header.time_date_stamp),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Vec::new(),
            None,
        )
    };

    // Security indicators
    let is_stripped = pe.header.coff_header.pointer_to_symbol_table == 0
        && pe.header.coff_header.number_of_symbol_table == 0;
    let has_code_signing = pe
        .header
        .optional_header
        .map(|opt| {
            // Certificate Table is data directory index 4
            opt.data_directories.get_certificate_table().is_some()
        })
        .unwrap_or(false);
    let (pe_is_driver, pe_driver_type, pe_driver_indicators) =
        classify_pe_driver(source_id, subsystem.as_deref(), &imports, &exports);
    let pe_version_info = extract_pe_version_info_strings(data);

    Ok(BinaryInfo {
        path: source_id.to_string(),
        format,
        architecture,
        is_64bit,
        entry_point: Some(pe.entry as u64),
        imports,
        exports,
        sections,
        strings,
        file_size,
        pe_timestamp: timestamp,
        pe_checksum: checksum,
        pe_subsystem: subsystem,
        pe_linker_version,
        pe_os_version,
        pe_image_version,
        pe_subsystem_version,
        pe_image_base,
        pe_section_alignment,
        pe_file_alignment,
        pe_size_of_image,
        pe_size_of_headers,
        pe_dll_characteristics,
        pe_dll_characteristics_detail,
        pe_certificate_table_size,
        pe_is_driver,
        pe_driver_type,
        pe_driver_indicators,
        pe_version_info,
        macho_cpu_type: None,
        macho_filetype: None,
        linux_module_info: None,
        has_debug_info: pe.debug_data.is_some(),
        is_stripped,
        has_code_signing,
    })
}

fn analyze_elf(
    elf: goblin::elf::Elf,
    source_id: &str,
    data: &[u8],
    file_size: u64,
    strings: Vec<String>,
) -> DocumentResult<BinaryInfo> {
    let is_64bit = elf.is_64;
    let format = if is_64bit {
        BinaryFormat::ELF64
    } else {
        BinaryFormat::ELF32
    };

    let architecture = match elf.header.e_machine {
        0x3E => "x86_64".to_string(),
        0x03 => "i386".to_string(),
        0xB7 => "ARM64".to_string(),
        0x28 => "ARM".to_string(),
        m => format!("0x{:04x}", m),
    };

    // Imports (dynamic symbols that are undefined)
    let imports: Vec<ImportInfo> = elf
        .libraries
        .iter()
        .take(MAX_BINARY_IMPORT_LIBRARIES)
        .map(|lib| ImportInfo {
            library: lib.to_string(),
            functions: Vec::new(),
            function_count: 0,
        })
        .collect();

    // Exports (dynamic symbols that are defined)
    let exports: Vec<ExportInfo> = elf
        .dynsyms
        .iter()
        .filter(|sym| sym.st_value != 0 && !sym.is_import())
        .filter_map(|sym| {
            elf.dynstrtab.get_at(sym.st_name).map(|name| ExportInfo {
                name: name.to_string(),
                ordinal: None,
                address: sym.st_value,
            })
        })
        .take(MAX_BINARY_EXPORTS)
        .collect();

    // Sections
    let sections: Vec<SectionInfo> = elf
        .section_headers
        .iter()
        .filter_map(|sec| {
            elf.shdr_strtab.get_at(sec.sh_name).map(|name| SectionInfo {
                name: name.to_string(),
                virtual_address: sec.sh_addr,
                virtual_size: sec.sh_size,
                raw_size: sec.sh_size,
                characteristics: format!("0x{:08x}", sec.sh_flags),
                characteristics_detail: decode_elf_section_flags(sec.sh_flags),
                entropy: section_entropy(data, sec.sh_offset, sec.sh_size),
            })
        })
        .take(MAX_BINARY_SECTIONS)
        .collect();

    // Security and debug indicators
    let has_debug_info = elf.section_headers.iter().any(|s| {
        elf.shdr_strtab
            .get_at(s.sh_name)
            .map(|n| n.starts_with(".debug"))
            .unwrap_or(false)
    });
    let has_code_signing = elf.section_headers.iter().any(|s| {
        elf.shdr_strtab
            .get_at(s.sh_name)
            .map(|n| n == ".note.gnu.build-id" || n == ".note.package")
            .unwrap_or(false)
    });
    let linux_module_info = linux_kernel_module_info(source_id, &strings);

    Ok(BinaryInfo {
        path: source_id.to_string(),
        format,
        architecture,
        is_64bit,
        entry_point: Some(elf.entry),
        imports,
        exports,
        sections,
        strings,
        file_size,
        pe_timestamp: None,
        pe_checksum: None,
        pe_subsystem: None,
        pe_linker_version: None,
        pe_os_version: None,
        pe_image_version: None,
        pe_subsystem_version: None,
        pe_image_base: None,
        pe_section_alignment: None,
        pe_file_alignment: None,
        pe_size_of_image: None,
        pe_size_of_headers: None,
        pe_dll_characteristics: None,
        pe_dll_characteristics_detail: Vec::new(),
        pe_certificate_table_size: None,
        pe_is_driver: false,
        pe_driver_type: None,
        pe_driver_indicators: Vec::new(),
        pe_version_info: BTreeMap::new(),
        macho_cpu_type: None,
        macho_filetype: None,
        linux_module_info,
        has_debug_info,
        is_stripped: elf.syms.is_empty(),
        has_code_signing,
    })
}

fn analyze_mach(
    mach: goblin::mach::Mach,
    source_id: &str,
    data: &[u8],
    file_size: u64,
    strings: Vec<String>,
) -> DocumentResult<BinaryInfo> {
    match mach {
        goblin::mach::Mach::Binary(macho) => {
            analyze_single_mach(macho, source_id, data, file_size, strings)
        }
        goblin::mach::Mach::Fat(fat) => {
            let narches = fat.narches;

            // Try to parse and analyze the first architecture fully
            if let Some(arch) = fat.iter_arches().flatten().next() {
                if let Some(slice) = checked_u64_slice(data, arch.offset as u64, arch.size as u64) {
                    if let Ok(Object::Mach(goblin::mach::Mach::Binary(inner))) =
                        Object::parse(slice)
                    {
                        let mut info =
                            analyze_single_mach(inner, source_id, slice, file_size, strings)?;
                        info.format = BinaryFormat::MachOFat;
                        info.architecture = format!(
                            "{} (Universal, {} architectures)",
                            info.architecture, narches
                        );
                        info.macho_cpu_type = Some(format!(
                            "{} (Fat, {} architectures)",
                            info.macho_cpu_type.unwrap_or_default(),
                            narches
                        ));
                        return Ok(info);
                    }
                }
            }

            // Fallback if we can't parse inner binary
            Ok(BinaryInfo {
                path: source_id.to_string(),
                format: BinaryFormat::MachOFat,
                architecture: "Universal".to_string(),
                is_64bit: true,
                entry_point: None,
                imports: Vec::new(),
                exports: Vec::new(),
                sections: Vec::new(),
                strings,
                file_size,
                pe_timestamp: None,
                pe_checksum: None,
                pe_subsystem: None,
                pe_linker_version: None,
                pe_os_version: None,
                pe_image_version: None,
                pe_subsystem_version: None,
                pe_image_base: None,
                pe_section_alignment: None,
                pe_file_alignment: None,
                pe_size_of_image: None,
                pe_size_of_headers: None,
                pe_dll_characteristics: None,
                pe_dll_characteristics_detail: Vec::new(),
                pe_certificate_table_size: None,
                pe_is_driver: false,
                pe_driver_type: None,
                pe_driver_indicators: Vec::new(),
                pe_version_info: BTreeMap::new(),
                macho_cpu_type: Some(format!("Fat ({} architectures)", narches)),
                macho_filetype: None,
                linux_module_info: None,
                has_debug_info: false,
                is_stripped: false,
                has_code_signing: false,
            })
        }
    }
}

fn checked_u64_slice(data: &[u8], offset: u64, size: u64) -> Option<&[u8]> {
    let start = usize::try_from(offset).ok()?;
    let len = usize::try_from(size).ok()?;
    let end = start.checked_add(len)?;
    data.get(start..end)
}

fn section_entropy(data: &[u8], offset: u64, size: u64) -> Option<f64> {
    let bytes = checked_u64_slice(data, offset, size)?;
    if bytes.is_empty() {
        return None;
    }

    let mut counts = [0usize; 256];
    for byte in bytes {
        counts[*byte as usize] += 1;
    }

    let len = bytes.len() as f64;
    let entropy = counts
        .iter()
        .filter(|count| **count > 0)
        .map(|count| {
            let probability = *count as f64 / len;
            -probability * probability.log2()
        })
        .sum::<f64>();

    Some((entropy * 1000.0).round() / 1000.0)
}

fn decode_pe_section_characteristics(characteristics: u32) -> Vec<String> {
    let mappings = [
        (0x0000_0020, "contains-code"),
        (0x0000_0040, "initialized-data"),
        (0x0000_0080, "uninitialized-data"),
        (0x0200_0000, "discardable"),
        (0x0400_0000, "not-cacheable"),
        (0x0800_0000, "not-pageable"),
        (0x1000_0000, "shared"),
        (0x2000_0000, "executable"),
        (0x4000_0000, "readable"),
        (0x8000_0000, "writable"),
    ];

    mappings
        .iter()
        .filter(|(mask, _)| characteristics & mask != 0)
        .map(|(_, label)| (*label).to_string())
        .collect()
}

fn decode_pe_dll_characteristics(characteristics: u16) -> Vec<String> {
    let mappings = [
        (0x0020, "high-entropy-va"),
        (0x0040, "dynamic-base"),
        (0x0080, "force-integrity"),
        (0x0100, "nx-compatible"),
        (0x0200, "no-isolation"),
        (0x0400, "no-seh"),
        (0x0800, "no-bind"),
        (0x1000, "appcontainer"),
        (0x2000, "wdm-driver"),
        (0x4000, "control-flow-guard"),
        (0x8000, "terminal-server-aware"),
    ];

    mappings
        .iter()
        .filter(|(mask, _)| characteristics & mask != 0)
        .map(|(_, label)| (*label).to_string())
        .collect()
}

fn decode_elf_section_flags(flags: u64) -> Vec<String> {
    let mappings = [
        (0x1, "writable"),
        (0x2, "allocated"),
        (0x4, "executable"),
        (0x10, "mergeable"),
        (0x20, "strings"),
        (0x40, "info-link"),
        (0x80, "link-order"),
        (0x100, "os-nonconforming"),
        (0x200, "group"),
        (0x400, "thread-local"),
        (0x800, "compressed"),
        (0x1000, "gnu-retain"),
        (0x2000_0000, "gnu-mbind"),
    ];

    mappings
        .iter()
        .filter(|(mask, _)| flags & mask != 0)
        .map(|(_, label)| (*label).to_string())
        .collect()
}

fn decode_macho_section_flags(flags: u32) -> Vec<String> {
    let mut values = Vec::new();
    match flags & 0x0000_00ff {
        0x0 => values.push("regular".to_string()),
        0x1 => values.push("zero-fill".to_string()),
        0x2 => values.push("cstring-literals".to_string()),
        0x3 => values.push("4-byte-literals".to_string()),
        0x4 => values.push("8-byte-literals".to_string()),
        0x5 => values.push("literal-pointers".to_string()),
        0x6 => values.push("non-lazy-symbol-pointers".to_string()),
        0x7 => values.push("lazy-symbol-pointers".to_string()),
        0x8 => values.push("symbol-stubs".to_string()),
        0x9 => values.push("mod-init-func-pointers".to_string()),
        0xa => values.push("mod-term-func-pointers".to_string()),
        0xb => values.push("coalesced".to_string()),
        0xc => values.push("gb-zero-fill".to_string()),
        0xd => values.push("interposing".to_string()),
        0xe => values.push("16-byte-literals".to_string()),
        0xf => values.push("dtrace-dof".to_string()),
        0x10 => values.push("lazy-dylib-symbol-pointers".to_string()),
        0x11 => values.push("thread-local-regular".to_string()),
        0x12 => values.push("thread-local-zero-fill".to_string()),
        0x13 => values.push("thread-local-variables".to_string()),
        0x14 => values.push("thread-local-variable-pointers".to_string()),
        0x15 => values.push("thread-local-init-function-pointers".to_string()),
        _ => {}
    }

    let attributes = [
        (0x8000_0000, "pure-instructions"),
        (0x4000_0000, "no-toc"),
        (0x2000_0000, "strip-static-symbols"),
        (0x1000_0000, "no-dead-strip"),
        (0x0800_0000, "live-support"),
        (0x0400_0000, "self-modifying-code"),
        (0x0200_0000, "debug"),
        (0x0000_0400, "some-instructions"),
        (0x0000_0200, "external-relocations"),
        (0x0000_0100, "local-relocations"),
    ];
    values.extend(
        attributes
            .iter()
            .filter(|(mask, _)| flags & mask != 0)
            .map(|(_, label)| (*label).to_string()),
    );
    values
}

fn push_limited_import(
    import_map: &mut HashMap<String, ImportAccumulator>,
    library: &str,
    function: &str,
) {
    if !import_map.contains_key(library) && import_map.len() >= MAX_BINARY_IMPORT_LIBRARIES {
        return;
    }

    let entry = import_map.entry(library.to_string()).or_default();
    entry.function_count = entry.function_count.saturating_add(1);
    if entry.functions.len() < MAX_BINARY_IMPORT_FUNCTIONS_PER_LIBRARY {
        entry.functions.push(function.to_string());
    }
}

fn import_infos_from_accumulators(
    import_map: HashMap<String, ImportAccumulator>,
) -> Vec<ImportInfo> {
    let mut imports: Vec<ImportInfo> = import_map
        .into_iter()
        .map(|(library, accumulator)| ImportInfo {
            library,
            functions: accumulator.functions,
            function_count: accumulator.function_count,
        })
        .collect();
    imports.sort_by(|left, right| left.library.cmp(&right.library));
    imports
}

fn classify_pe_driver(
    source_id: &str,
    subsystem: Option<&str>,
    imports: &[ImportInfo],
    exports: &[ExportInfo],
) -> (bool, Option<String>, Vec<String>) {
    let mut indicators = Vec::new();
    let has_sys_extension = source_id
        .to_ascii_lowercase()
        .rsplit(['/', '\\', ':'])
        .next()
        .is_some_and(|name| name.ends_with(".sys") || name.ends_with(".drv"));
    if has_sys_extension {
        indicators.push("driver file extension".to_string());
    }
    if subsystem == Some("Native") {
        indicators.push("PE native subsystem".to_string());
    }

    let imports_ntoskrnl = imports_library(imports, "ntoskrnl.exe");
    if imports_ntoskrnl {
        indicators.push("imports ntoskrnl.exe".to_string());
    }
    if imports_library(imports, "hal.dll") {
        indicators.push("imports hal.dll".to_string());
    }
    if exports_function(exports, "DriverEntry") || imports_function(imports, "DriverEntry") {
        indicators.push("DriverEntry entry point".to_string());
    }
    if imports_library(imports, "fltmgr.sys") || imports_function(imports, "FltRegisterFilter") {
        indicators.push("file-system filter driver APIs".to_string());
    }
    if imports_library(imports, "ndis.sys") || imports_function(imports, "NdisRegister") {
        indicators.push("network driver APIs".to_string());
    }
    if imports_library(imports, "wdf01000.sys") || imports_function(imports, "WdfDriverCreate") {
        indicators.push("KMDF driver framework APIs".to_string());
    }
    if imports_library(imports, "storport.sys")
        || imports_library(imports, "scsiport.sys")
        || imports_library(imports, "classpnp.sys")
        || imports_function(imports, "StorPortInitialize")
        || imports_function(imports, "ScsiPortInitialize")
    {
        indicators.push("storage driver APIs".to_string());
    }
    if imports_function_prefix(imports, "FsRtl")
        || imports_function(imports, "IoRegisterFileSystem")
    {
        indicators.push("file-system driver APIs".to_string());
    }
    if imports_function(imports, "PsSetCreateProcessNotifyRoutine")
        || imports_function(imports, "PsSetCreateThreadNotifyRoutine")
        || imports_function(imports, "PsSetLoadImageNotifyRoutine")
        || imports_function(imports, "ObRegisterCallbacks")
        || imports_function(imports, "CmRegisterCallback")
    {
        indicators.push("security callback driver APIs".to_string());
    }
    if imports_library(imports, "usbd.sys")
        || imports_library(imports, "usbport.sys")
        || imports_function(imports, "WdfUsbTargetDeviceCreate")
    {
        indicators.push("USB driver APIs".to_string());
    }
    if imports_library(imports, "hidclass.sys") || imports_function_prefix(imports, "HidP_") {
        indicators.push("HID driver APIs".to_string());
    }
    if imports_library(imports, "dxgkrnl.sys")
        || imports_library(imports, "dxgmms1.sys")
        || imports_library(imports, "dxgmms2.sys")
        || imports_function_prefix(imports, "Dxgk")
    {
        indicators.push("display driver APIs".to_string());
    }

    indicators.sort();
    indicators.dedup();

    let is_driver = has_sys_extension
        || imports_ntoskrnl
        || imports_library(imports, "hal.dll")
        || exports_function(exports, "DriverEntry")
        || imports_function(imports, "DriverEntry")
        || imports_library(imports, "fltmgr.sys")
        || imports_library(imports, "ndis.sys")
        || imports_library(imports, "wdf01000.sys")
        || imports_function(imports, "FltRegisterFilter")
        || imports_function(imports, "WdfDriverCreate")
        || imports_library(imports, "storport.sys")
        || imports_library(imports, "scsiport.sys")
        || imports_library(imports, "classpnp.sys")
        || imports_function(imports, "StorPortInitialize")
        || imports_function(imports, "ScsiPortInitialize")
        || imports_function_prefix(imports, "FsRtl")
        || imports_function(imports, "IoRegisterFileSystem")
        || imports_function(imports, "PsSetCreateProcessNotifyRoutine")
        || imports_function(imports, "PsSetCreateThreadNotifyRoutine")
        || imports_function(imports, "PsSetLoadImageNotifyRoutine")
        || imports_function(imports, "ObRegisterCallbacks")
        || imports_function(imports, "CmRegisterCallback")
        || imports_library(imports, "usbd.sys")
        || imports_library(imports, "usbport.sys")
        || imports_function(imports, "WdfUsbTargetDeviceCreate")
        || imports_library(imports, "hidclass.sys")
        || imports_function_prefix(imports, "HidP_")
        || imports_library(imports, "dxgkrnl.sys")
        || imports_library(imports, "dxgmms1.sys")
        || imports_library(imports, "dxgmms2.sys")
        || imports_function_prefix(imports, "Dxgk");
    let driver_type = if !is_driver {
        None
    } else if imports_library(imports, "fltmgr.sys")
        || imports_function(imports, "FltRegisterFilter")
    {
        Some("File system minifilter driver".to_string())
    } else if imports_library(imports, "storport.sys")
        || imports_library(imports, "scsiport.sys")
        || imports_library(imports, "classpnp.sys")
        || imports_function(imports, "StorPortInitialize")
        || imports_function(imports, "ScsiPortInitialize")
    {
        Some("Storage driver".to_string())
    } else if imports_library(imports, "ndis.sys") || imports_function(imports, "NdisRegister") {
        Some("Network driver".to_string())
    } else if imports_function(imports, "PsSetCreateProcessNotifyRoutine")
        || imports_function(imports, "PsSetCreateThreadNotifyRoutine")
        || imports_function(imports, "PsSetLoadImageNotifyRoutine")
        || imports_function(imports, "ObRegisterCallbacks")
        || imports_function(imports, "CmRegisterCallback")
    {
        Some("Security callback driver".to_string())
    } else if imports_library(imports, "usbd.sys")
        || imports_library(imports, "usbport.sys")
        || imports_function(imports, "WdfUsbTargetDeviceCreate")
    {
        Some("USB driver".to_string())
    } else if imports_library(imports, "hidclass.sys") || imports_function_prefix(imports, "HidP_")
    {
        Some("HID driver".to_string())
    } else if imports_library(imports, "dxgkrnl.sys")
        || imports_library(imports, "dxgmms1.sys")
        || imports_library(imports, "dxgmms2.sys")
        || imports_function_prefix(imports, "Dxgk")
    {
        Some("Display driver".to_string())
    } else if imports_function_prefix(imports, "FsRtl")
        || imports_function(imports, "IoRegisterFileSystem")
    {
        Some("File system driver".to_string())
    } else if imports_library(imports, "wdf01000.sys")
        || imports_function(imports, "WdfDriverCreate")
    {
        Some("Kernel-Mode Driver Framework driver".to_string())
    } else {
        Some("Windows kernel driver".to_string())
    };

    (is_driver, driver_type, indicators)
}

fn imports_library(imports: &[ImportInfo], library: &str) -> bool {
    imports
        .iter()
        .any(|import| import.library.eq_ignore_ascii_case(library))
}

fn imports_function(imports: &[ImportInfo], function: &str) -> bool {
    imports.iter().any(|import| {
        import
            .functions
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(function))
    })
}

fn imports_function_prefix(imports: &[ImportInfo], prefix: &str) -> bool {
    imports.iter().any(|import| {
        import.functions.iter().any(|candidate| {
            candidate
                .get(..prefix.len())
                .is_some_and(|head| head.eq_ignore_ascii_case(prefix))
        })
    })
}

fn exports_function(exports: &[ExportInfo], function: &str) -> bool {
    exports
        .iter()
        .any(|export| export.name.eq_ignore_ascii_case(function))
}

fn extract_pe_version_info_strings(data: &[u8]) -> BTreeMap<String, String> {
    let mut version_info = BTreeMap::new();
    for key in PE_VERSION_INFO_KEYS {
        if version_info.len() >= MAX_PE_VERSION_INFO_FIELDS {
            break;
        }
        if let Some(value) = find_utf16le_version_value(data, key) {
            version_info.insert((*key).to_string(), value);
        }
    }
    version_info
}

fn find_utf16le_version_value(data: &[u8], key: &str) -> Option<String> {
    let key_pattern = utf16le_nul_terminated_pattern(key);
    let key_offset = find_subslice(data, &key_pattern)?;
    let value_search_start = key_offset.checked_add(key_pattern.len())?;
    let mut candidate_offsets = Vec::new();
    for skipped in (0..=32).step_by(2) {
        if let Some(offset) = value_search_start.checked_add(skipped) {
            candidate_offsets.push(offset);
            candidate_offsets.push(align_up(offset, 4)?);
        }
    }

    candidate_offsets.sort_unstable();
    candidate_offsets.dedup();
    candidate_offsets
        .into_iter()
        .filter_map(|offset| read_utf16le_string_at(data, offset))
        .find(|value| looks_like_version_resource_value(value))
}

fn utf16le_nul_terminated_pattern(value: &str) -> Vec<u8> {
    value
        .encode_utf16()
        .chain(std::iter::once(0))
        .flat_map(u16::to_le_bytes)
        .collect()
}

fn find_subslice(data: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || needle.len() > data.len() {
        return None;
    }
    data.windows(needle.len())
        .position(|window| window == needle)
}

fn align_up(value: usize, alignment: usize) -> Option<usize> {
    let mask = alignment.checked_sub(1)?;
    value.checked_add(mask).map(|candidate| candidate & !mask)
}

fn read_utf16le_string_at(data: &[u8], offset: usize) -> Option<String> {
    if offset >= data.len() || !offset.is_multiple_of(2) {
        return None;
    }

    let mut units = Vec::new();
    for chunk in data[offset..].chunks_exact(2) {
        let unit = u16::from_le_bytes([chunk[0], chunk[1]]);
        if unit == 0 {
            break;
        }
        units.push(unit);
        if units.len() >= MAX_PE_VERSION_INFO_VALUE_CHARS {
            break;
        }
    }

    if units.is_empty() {
        return None;
    }
    String::from_utf16(&units)
        .ok()
        .map(|value| value.trim().to_string())
}

fn looks_like_version_resource_value(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|ch| !ch.is_control() || matches!(ch, '\t' | '\n' | '\r'))
        && value.chars().any(|ch| ch.is_ascii_alphanumeric())
}

fn extract_binary_strings(data: &[u8]) -> Vec<String> {
    let mut strings = Vec::new();
    let mut seen = HashSet::new();
    collect_ascii_strings(data, &mut strings, &mut seen);
    if strings.len() < MAX_BINARY_STRINGS {
        collect_utf16le_strings(data, &mut strings, &mut seen);
    }
    strings
}

fn collect_ascii_strings(data: &[u8], strings: &mut Vec<String>, seen: &mut HashSet<String>) {
    let mut start: Option<usize> = None;

    for (index, byte) in data.iter().enumerate() {
        if is_printable_ascii_byte(*byte) {
            start.get_or_insert(index);
            continue;
        }

        if let Some(run_start) = start.take() {
            push_ascii_run(data, run_start, index, strings, seen);
            if strings.len() >= MAX_BINARY_STRINGS {
                return;
            }
        }
    }

    if let Some(run_start) = start {
        push_ascii_run(data, run_start, data.len(), strings, seen);
    }
}

fn push_ascii_run(
    data: &[u8],
    start: usize,
    end: usize,
    strings: &mut Vec<String>,
    seen: &mut HashSet<String>,
) {
    let run_len = end.saturating_sub(start);
    if run_len < MIN_BINARY_STRING_CHARS {
        return;
    }
    let value: String = data[start..end]
        .iter()
        .take(MAX_BINARY_STRING_CHARS)
        .map(|byte| *byte as char)
        .collect();
    push_limited_string(strings, seen, value);
}

fn collect_utf16le_strings(data: &[u8], strings: &mut Vec<String>, seen: &mut HashSet<String>) {
    let mut offset = 0usize;

    while offset + 1 < data.len() && strings.len() < MAX_BINARY_STRINGS {
        if offset > 0 && data[offset - 1] != 0 {
            offset += 1;
            continue;
        }

        let mut cursor = offset;
        let mut units = Vec::new();

        while cursor + 1 < data.len() {
            let unit = u16::from_le_bytes([data[cursor], data[cursor + 1]]);
            if !is_printable_utf16_unit(unit) {
                break;
            }
            if units.len() < MAX_BINARY_STRING_CHARS {
                units.push(unit);
            }
            cursor += 2;
        }

        if units.len() >= MIN_BINARY_STRING_CHARS {
            if let Ok(value) = String::from_utf16(&units) {
                push_limited_string(strings, seen, value);
            }
            offset = cursor.saturating_add(2);
        } else {
            offset += 1;
        }
    }
}

fn is_printable_ascii_byte(byte: u8) -> bool {
    matches!(byte, 0x20..=0x7e)
}

fn is_printable_utf16_unit(unit: u16) -> bool {
    matches!(unit, 0x20..=0x7e)
}

fn push_limited_string(strings: &mut Vec<String>, seen: &mut HashSet<String>, value: String) {
    if strings.len() >= MAX_BINARY_STRINGS {
        return;
    }

    let trimmed = value.trim();
    if trimmed.chars().count() < MIN_BINARY_STRING_CHARS
        || !trimmed.chars().any(|ch| ch.is_alphanumeric())
    {
        return;
    }

    let normalized: String = trimmed.chars().take(MAX_BINARY_STRING_CHARS).collect();
    if seen.insert(normalized.clone()) {
        strings.push(normalized);
    }
}

fn linux_kernel_module_info(source_id: &str, strings: &[String]) -> Option<LinuxModuleInfo> {
    if !source_id
        .replace('\\', "/")
        .to_ascii_lowercase()
        .ends_with(".ko")
    {
        return None;
    }

    let mut info = LinuxModuleInfo::default();
    for value in strings {
        let Some((key, value)) = split_linux_module_info(value) else {
            continue;
        };
        match key {
            "name" => push_unique_module_value(&mut info.names, value),
            "version" => push_unique_module_value(&mut info.versions, value),
            "vermagic" => push_unique_module_value(&mut info.vermagic, value),
            "license" => push_unique_module_value(&mut info.licenses, value),
            "author" => push_unique_module_value(&mut info.authors, value),
            "description" => push_unique_module_value(&mut info.descriptions, value),
            "alias" => push_unique_module_value(&mut info.aliases, value),
            "depends" => {
                for dependency in value.split(',') {
                    let dependency = dependency.trim();
                    if !dependency.is_empty() {
                        push_unique_module_value(&mut info.dependencies, dependency.to_string());
                    }
                }
            }
            "firmware" => push_unique_module_value(&mut info.firmware, value),
            "signer" => push_unique_module_value(&mut info.signers, value),
            "sig_key" | "sig_hashalgo" => {
                push_unique_module_value(&mut info.signatures, format!("{key}={value}"))
            }
            _ => {}
        }
    }

    info.detected = linux_module_info_has_values(&info);
    info.detected.then_some(info)
}

fn split_linux_module_info(value: &str) -> Option<(&str, String)> {
    let (key, value) = value.split_once('=')?;
    let key = key.trim();
    if !matches!(
        key,
        "name"
            | "version"
            | "vermagic"
            | "license"
            | "author"
            | "description"
            | "alias"
            | "depends"
            | "firmware"
            | "signer"
            | "sig_key"
            | "sig_hashalgo"
    ) {
        return None;
    }

    let value = value.trim();
    if value.is_empty() {
        return None;
    }

    Some((key, truncate_linux_module_value(value)))
}

fn truncate_linux_module_value(value: &str) -> String {
    const MAX_LINUX_MODULE_VALUE_CHARS: usize = 180;
    value.chars().take(MAX_LINUX_MODULE_VALUE_CHARS).collect()
}

fn push_unique_module_value(values: &mut Vec<String>, value: String) {
    const MAX_LINUX_MODULE_VALUES: usize = 64;
    if values.len() >= MAX_LINUX_MODULE_VALUES || values.iter().any(|existing| existing == &value) {
        return;
    }
    values.push(value);
}

fn linux_module_info_has_values(info: &LinuxModuleInfo) -> bool {
    !info.names.is_empty()
        || !info.versions.is_empty()
        || !info.vermagic.is_empty()
        || !info.licenses.is_empty()
        || !info.authors.is_empty()
        || !info.descriptions.is_empty()
        || !info.aliases.is_empty()
        || !info.dependencies.is_empty()
        || !info.firmware.is_empty()
        || !info.signers.is_empty()
        || !info.signatures.is_empty()
}

fn analyze_single_mach(
    macho: goblin::mach::MachO,
    source_id: &str,
    data: &[u8],
    file_size: u64,
    strings: Vec<String>,
) -> DocumentResult<BinaryInfo> {
    // Check if 64-bit by looking at magic number
    let is_64bit = matches!(macho.header.magic, 0xFEEDFACF | 0xCFFAEDFE);
    let format = if is_64bit {
        BinaryFormat::MachO64
    } else {
        BinaryFormat::MachO32
    };

    let cpu_type = match macho.header.cputype {
        0x01000007 => "x86_64".to_string(),
        0x0100000C => "ARM64".to_string(),
        0x07 => "i386".to_string(),
        0x0C => "ARM".to_string(),
        c => format!("0x{:08x}", c),
    };

    let filetype = match macho.header.filetype {
        1 => "Object",
        2 => "Executable",
        3 => "Fixed VM Library",
        4 => "Core",
        5 => "Preload",
        6 => "Dylib",
        7 => "Dylinker",
        8 => "Bundle",
        _ => "Unknown",
    };

    // Imports
    let imports: Vec<ImportInfo> = macho
        .libs
        .iter()
        .take(MAX_BINARY_IMPORT_LIBRARIES)
        .map(|lib| ImportInfo {
            library: lib.to_string(),
            functions: Vec::new(),
            function_count: 0,
        })
        .collect();

    // Exports
    let exports: Vec<ExportInfo> = macho
        .exports()
        .map_err(|e| DocumentError::Parse(format!("Failed to read exports: {}", e)))?
        .iter()
        .take(MAX_BINARY_EXPORTS)
        .map(|exp| ExportInfo {
            name: exp.name.clone(),
            ordinal: None,
            address: exp.offset,
        })
        .collect();

    // Sections
    let sections: Vec<SectionInfo> = macho
        .segments
        .iter()
        .flat_map(|seg| seg.sections().ok().unwrap_or_default())
        .take(MAX_BINARY_SECTIONS)
        .map(|(sec, _)| SectionInfo {
            name: format!(
                "{},{}",
                sec.segname().unwrap_or("?"),
                sec.name().unwrap_or("?")
            ),
            virtual_address: sec.addr,
            virtual_size: sec.size,
            raw_size: sec.size,
            characteristics: format!("0x{:08x}", sec.flags),
            characteristics_detail: decode_macho_section_flags(sec.flags),
            entropy: section_entropy(data, sec.offset as u64, sec.size),
        })
        .collect();

    // Security indicators
    let has_debug_info = macho
        .segments
        .iter()
        .any(|seg| seg.name().ok().map(|n| n == "__DWARF").unwrap_or(false));
    let has_code_signing = macho.load_commands.iter().any(|lc| {
        // LC_CODE_SIGNATURE = 0x1D
        lc.command.cmd() == 0x1D
    });
    let is_stripped = macho.symbols().next().is_none();

    Ok(BinaryInfo {
        path: source_id.to_string(),
        format,
        architecture: cpu_type.clone(),
        is_64bit,
        entry_point: Some(macho.entry),
        imports,
        exports,
        sections,
        strings,
        file_size,
        pe_timestamp: None,
        pe_checksum: None,
        pe_subsystem: None,
        pe_linker_version: None,
        pe_os_version: None,
        pe_image_version: None,
        pe_subsystem_version: None,
        pe_image_base: None,
        pe_section_alignment: None,
        pe_file_alignment: None,
        pe_size_of_image: None,
        pe_size_of_headers: None,
        pe_dll_characteristics: None,
        pe_dll_characteristics_detail: Vec::new(),
        pe_certificate_table_size: None,
        pe_is_driver: false,
        pe_driver_type: None,
        pe_driver_indicators: Vec::new(),
        pe_version_info: BTreeMap::new(),
        macho_cpu_type: Some(cpu_type),
        macho_filetype: Some(filetype.to_string()),
        linux_module_info: None,
        has_debug_info,
        is_stripped,
        has_code_signing,
    })
}

/// Quick format detection without full parsing
pub fn detect_binary_format(path: impl AsRef<Path>) -> DocumentResult<BinaryFormat> {
    let data = read_binary_prefix(path.as_ref(), MAX_BINARY_DETECT_PREFIX_BYTES)?;
    Ok(detect_binary_format_bytes(&data))
}

/// Quick format detection from a bounded header/prefix.
pub fn detect_binary_format_bytes(data: &[u8]) -> BinaryFormat {
    if data.len() < 4 {
        return BinaryFormat::Unknown;
    }

    if data.starts_with(b"\x7fELF") {
        return match data.get(4) {
            Some(1) => BinaryFormat::ELF32,
            Some(2) => BinaryFormat::ELF64,
            _ => BinaryFormat::Unknown,
        };
    }

    match data.get(0..4) {
        Some([0xFE, 0xED, 0xFA, 0xCE]) | Some([0xCE, 0xFA, 0xED, 0xFE]) => {
            return BinaryFormat::MachO32
        }
        Some([0xFE, 0xED, 0xFA, 0xCF]) | Some([0xCF, 0xFA, 0xED, 0xFE]) => {
            return BinaryFormat::MachO64
        }
        Some([0xCA, 0xFE, 0xBA, 0xBE])
        | Some([0xBE, 0xBA, 0xFE, 0xCA])
        | Some([0xCA, 0xFE, 0xBA, 0xBF])
        | Some([0xBF, 0xBA, 0xFE, 0xCA]) => return BinaryFormat::MachOFat,
        _ => {}
    }

    if data.starts_with(b"MZ") {
        return detect_pe_format_from_prefix(data);
    }

    BinaryFormat::Unknown
}

fn detect_pe_format_from_prefix(data: &[u8]) -> BinaryFormat {
    let pe_offset = match data.get(0x3c..0x40) {
        Some(bytes) => u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize,
        None => return BinaryFormat::Unknown,
    };
    let optional_magic_offset = match pe_offset.checked_add(24) {
        Some(offset) => offset,
        None => return BinaryFormat::Unknown,
    };

    if data.get(pe_offset..pe_offset.saturating_add(4)) != Some(&b"PE\0\0"[..]) {
        return BinaryFormat::Unknown;
    }

    match data.get(optional_magic_offset..optional_magic_offset.saturating_add(2)) {
        Some([0x0b, 0x01]) => BinaryFormat::PE32,
        Some([0x0b, 0x02]) => BinaryFormat::PE64,
        _ => BinaryFormat::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::io::Write;

    #[test]
    fn test_binary_format_enum() {
        let format = BinaryFormat::PE64;
        assert!(matches!(format, BinaryFormat::PE64));
    }

    #[test]
    fn test_checked_u64_slice_valid_range() {
        let data = b"hello";
        assert_eq!(checked_u64_slice(data, 1, 3), Some(&b"ell"[..]));
    }

    #[test]
    fn test_checked_u64_slice_rejects_overflow_range() {
        let data = b"hello";
        assert!(checked_u64_slice(data, 4, u64::MAX).is_none());
    }

    #[test]
    fn section_entropy_scores_raw_section_bytes() {
        let mut data = vec![0u8; 16];
        for value in 0u8..=255 {
            data.push(value);
        }

        assert_eq!(section_entropy(&data, 0, 16), Some(0.0));
        assert_eq!(section_entropy(&data, 16, 256), Some(8.0));
        assert_eq!(section_entropy(&data, 16, 0), None);
        assert_eq!(section_entropy(&data, 10_000, 4), None);
    }

    #[test]
    fn section_flag_decoders_label_common_executable_and_writable_traits() {
        assert_eq!(
            decode_pe_section_characteristics(0x6000_0020),
            vec!["contains-code", "executable", "readable"]
        );
        assert_eq!(
            decode_pe_section_characteristics(0xc000_0040),
            vec!["initialized-data", "readable", "writable"]
        );
        assert_eq!(
            decode_pe_dll_characteristics(0x2140),
            vec!["dynamic-base", "nx-compatible", "wdm-driver"]
        );
        assert_eq!(
            decode_elf_section_flags(0x7),
            vec!["writable", "allocated", "executable"]
        );
        assert_eq!(
            decode_macho_section_flags(0x8000_0400),
            vec!["regular", "pure-instructions", "some-instructions"]
        );
    }

    #[test]
    fn import_accumulator_caps_libraries_and_functions() {
        let mut imports = HashMap::new();
        for index in 0..(MAX_BINARY_IMPORT_FUNCTIONS_PER_LIBRARY + 5) {
            push_limited_import(&mut imports, "KERNEL32.dll", &format!("Function{index}"));
        }
        for index in 0..(MAX_BINARY_IMPORT_LIBRARIES + 5) {
            push_limited_import(&mut imports, &format!("lib{index}.dll"), "OnlyFunction");
        }

        let infos = import_infos_from_accumulators(imports);
        assert_eq!(infos.len(), MAX_BINARY_IMPORT_LIBRARIES);

        let kernel32 = infos
            .iter()
            .find(|info| info.library == "KERNEL32.dll")
            .expect("retained first import library");
        assert_eq!(
            kernel32.function_count,
            MAX_BINARY_IMPORT_FUNCTIONS_PER_LIBRARY + 5
        );
        assert_eq!(
            kernel32.functions.len(),
            MAX_BINARY_IMPORT_FUNCTIONS_PER_LIBRARY
        );
    }

    #[test]
    fn classify_pe_driver_identifies_sys_kernel_driver() {
        let imports = vec![ImportInfo {
            library: "ntoskrnl.exe".to_string(),
            functions: vec!["IoCreateDevice".to_string()],
            function_count: 1,
        }];
        let exports = vec![ExportInfo {
            name: "DriverEntry".to_string(),
            ordinal: None,
            address: 0x1000,
        }];

        let (is_driver, driver_type, indicators) = classify_pe_driver(
            "evidence.ad1:/Windows/System32/drivers/example.sys",
            Some("Native"),
            &imports,
            &exports,
        );

        assert!(is_driver);
        assert_eq!(driver_type.as_deref(), Some("Windows kernel driver"));
        assert!(indicators.contains(&"driver file extension".to_string()));
        assert!(indicators.contains(&"imports ntoskrnl.exe".to_string()));
        assert!(indicators.contains(&"DriverEntry entry point".to_string()));
    }

    #[test]
    fn classify_pe_driver_identifies_minifilter_driver() {
        let imports = vec![ImportInfo {
            library: "fltmgr.sys".to_string(),
            functions: vec!["FltRegisterFilter".to_string()],
            function_count: 1,
        }];

        let (is_driver, driver_type, indicators) = classify_pe_driver(
            "C:\\Windows\\System32\\drivers\\filter.sys",
            Some("Native"),
            &imports,
            &[],
        );

        assert!(is_driver);
        assert_eq!(
            driver_type.as_deref(),
            Some("File system minifilter driver")
        );
        assert!(indicators.contains(&"file-system filter driver APIs".to_string()));
    }

    #[test]
    fn classify_pe_driver_identifies_storage_driver() {
        let imports = vec![ImportInfo {
            library: "storport.sys".to_string(),
            functions: vec!["StorPortInitialize".to_string()],
            function_count: 1,
        }];

        let (is_driver, driver_type, indicators) = classify_pe_driver(
            "C:\\Windows\\System32\\drivers\\storflt.sys",
            Some("Native"),
            &imports,
            &[],
        );

        assert!(is_driver);
        assert_eq!(driver_type.as_deref(), Some("Storage driver"));
        assert!(indicators.contains(&"storage driver APIs".to_string()));
    }

    #[test]
    fn classify_pe_driver_identifies_security_callback_driver() {
        let imports = vec![ImportInfo {
            library: "ntoskrnl.exe".to_string(),
            functions: vec![
                "PsSetCreateProcessNotifyRoutine".to_string(),
                "ObRegisterCallbacks".to_string(),
            ],
            function_count: 2,
        }];

        let (is_driver, driver_type, indicators) = classify_pe_driver(
            "C:\\Windows\\System32\\drivers\\watcher.sys",
            Some("Native"),
            &imports,
            &[],
        );

        assert!(is_driver);
        assert_eq!(driver_type.as_deref(), Some("Security callback driver"));
        assert!(indicators.contains(&"security callback driver APIs".to_string()));
    }

    #[test]
    fn classify_pe_driver_identifies_usb_hid_and_display_drivers() {
        let usb_imports = vec![ImportInfo {
            library: "usbd.sys".to_string(),
            functions: vec!["WdfUsbTargetDeviceCreate".to_string()],
            function_count: 1,
        }];
        let hid_imports = vec![ImportInfo {
            library: "hidclass.sys".to_string(),
            functions: vec!["HidP_GetCaps".to_string()],
            function_count: 1,
        }];
        let display_imports = vec![ImportInfo {
            library: "dxgkrnl.sys".to_string(),
            functions: vec!["DxgkInitialize".to_string()],
            function_count: 1,
        }];

        assert_eq!(
            classify_pe_driver("usb.sys", Some("Native"), &usb_imports, &[])
                .1
                .as_deref(),
            Some("USB driver")
        );
        assert_eq!(
            classify_pe_driver("hid.sys", Some("Native"), &hid_imports, &[])
                .1
                .as_deref(),
            Some("HID driver")
        );
        assert_eq!(
            classify_pe_driver("display.sys", Some("Native"), &display_imports, &[])
                .1
                .as_deref(),
            Some("Display driver")
        );
    }

    #[test]
    fn classify_pe_driver_leaves_user_mode_exe_unclassified() {
        let imports = vec![ImportInfo {
            library: "KERNEL32.dll".to_string(),
            functions: vec!["CreateFileW".to_string()],
            function_count: 1,
        }];

        let (is_driver, driver_type, indicators) =
            classify_pe_driver("C:\\Tools\\viewer.exe", Some("Console"), &imports, &[]);

        assert!(!is_driver);
        assert!(driver_type.is_none());
        assert!(indicators.is_empty());
    }

    #[test]
    fn extract_pe_version_info_strings_reads_driver_identity_fields() {
        let mut data = Vec::new();
        append_utf16le_version_pair(&mut data, "CompanyName", "Contoso Driver Labs");
        append_utf16le_version_pair(&mut data, "FileDescription", "Contoso Storage Filter");
        append_utf16le_version_pair(&mut data, "FileVersion", "1.2.3.4");
        append_utf16le_version_pair(&mut data, "OriginalFilename", "contosoflt.sys");

        let version_info = extract_pe_version_info_strings(&data);

        assert_eq!(
            version_info.get("CompanyName").map(String::as_str),
            Some("Contoso Driver Labs")
        );
        assert_eq!(
            version_info.get("FileDescription").map(String::as_str),
            Some("Contoso Storage Filter")
        );
        assert_eq!(
            version_info.get("FileVersion").map(String::as_str),
            Some("1.2.3.4")
        );
        assert_eq!(
            version_info.get("OriginalFilename").map(String::as_str),
            Some("contosoflt.sys")
        );
    }

    #[test]
    fn extract_binary_strings_reads_ascii_and_utf16le_values() {
        let mut data = Vec::new();
        data.extend_from_slice(
            b"\0\0\\Registry\\Machine\\System\\CurrentControlSet\\Services\\contosoflt\0",
        );
        append_utf16le_nul_terminated(&mut data, "\\Device\\ContosoFilter");
        data.extend_from_slice(b"\0\0https://drivers.example.test/update\0");

        let strings = extract_binary_strings(&data);

        assert!(strings.contains(
            &"\\Registry\\Machine\\System\\CurrentControlSet\\Services\\contosoflt".to_string()
        ));
        assert!(strings.contains(&"\\Device\\ContosoFilter".to_string()));
        assert!(strings.contains(&"https://drivers.example.test/update".to_string()));
    }

    #[test]
    fn extract_binary_strings_deduplicates_and_limits_values() {
        let mut data = Vec::new();
        data.extend_from_slice(b"RepeatedDriverString\0RepeatedDriverString\0");
        for index in 0..(MAX_BINARY_STRINGS + 25) {
            data.extend_from_slice(format!("UniqueDriverString{index:04}\0").as_bytes());
        }

        let strings = extract_binary_strings(&data);

        assert_eq!(
            strings
                .iter()
                .filter(|value| value.as_str() == "RepeatedDriverString")
                .count(),
            1
        );
        assert_eq!(strings.len(), MAX_BINARY_STRINGS);
    }

    #[test]
    fn detect_binary_format_bytes_identifies_pe64_from_prefix() {
        let data = minimal_pe_header(0x20b);

        assert!(matches!(
            detect_binary_format_bytes(&data),
            BinaryFormat::PE64
        ));
    }

    #[test]
    fn detect_binary_format_reads_sparse_file_prefix_only() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(&minimal_pe_header(0x10b)).unwrap();
        tmp.as_file_mut()
            .set_len(MAX_BINARY_ANALYSIS_BYTES + 1)
            .unwrap();

        let format = detect_binary_format(tmp.path()).unwrap();

        assert!(matches!(format, BinaryFormat::PE32));
    }

    #[test]
    fn analyze_binary_rejects_oversized_local_file_before_reading() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(&minimal_elf64_header()).unwrap();
        tmp.as_file_mut()
            .set_len(MAX_BINARY_ANALYSIS_BYTES + 1)
            .unwrap();

        let err = analyze_binary(tmp.path()).unwrap_err();

        assert!(err.to_string().contains("too large for full analysis"));
    }

    #[test]
    fn read_binary_analysis_with_limit_accepts_exact_limit() {
        let data = read_binary_analysis_with_limit(Cursor::new(b"abcd"), 4).unwrap();

        assert_eq!(data, b"abcd");
    }

    #[test]
    fn read_binary_analysis_with_limit_rejects_reader_past_limit() {
        let err = read_binary_analysis_with_limit(Cursor::new(b"abcde"), 4).unwrap_err();

        assert!(err.to_string().contains("too large for full analysis"));
    }

    #[test]
    fn analyze_binary_bytes_reads_elf_source_metadata() {
        let data = minimal_elf64_header();
        let info = analyze_binary_bytes("container.ad1:/bin/tool", &data).unwrap();

        assert_eq!(info.path, "container.ad1:/bin/tool");
        assert!(matches!(info.format, BinaryFormat::ELF64));
        assert_eq!(info.architecture, "x86_64");
        assert!(info.is_64bit);
        assert_eq!(info.entry_point, Some(0x400000));
        assert_eq!(info.file_size, data.len() as u64);
        assert!(info.strings.is_empty());
        assert!(info.linux_module_info.is_none());
    }

    #[test]
    fn analyze_binary_bytes_extracts_linux_kernel_module_metadata() {
        let mut data = minimal_elf64_header();
        append_ascii_nul_terminated(&mut data, "name=coretap");
        append_ascii_nul_terminated(&mut data, "version=1.2.3");
        append_ascii_nul_terminated(&mut data, "vermagic=6.8.0 SMP mod_unload");
        append_ascii_nul_terminated(&mut data, "license=GPL");
        append_ascii_nul_terminated(&mut data, "depends=cfg80211,rfkill");
        append_ascii_nul_terminated(&mut data, "signer=CORE Lab");
        append_ascii_nul_terminated(&mut data, "sig_hashalgo=sha256");

        let info =
            analyze_binary_bytes("e01:/case/linux.E01:/lib/modules/coretap.ko", &data).unwrap();
        let module = info.linux_module_info.unwrap();

        assert!(module.detected);
        assert_eq!(module.names, vec!["coretap"]);
        assert_eq!(module.versions, vec!["1.2.3"]);
        assert_eq!(module.vermagic, vec!["6.8.0 SMP mod_unload"]);
        assert_eq!(module.licenses, vec!["GPL"]);
        assert_eq!(module.dependencies, vec!["cfg80211", "rfkill"]);
        assert_eq!(module.signers, vec!["CORE Lab"]);
        assert_eq!(module.signatures, vec!["sig_hashalgo=sha256"]);
    }

    fn minimal_pe_header(optional_magic: u16) -> Vec<u8> {
        let pe_offset = 0x80usize;
        let mut data = vec![0u8; pe_offset + 26];
        data[0..2].copy_from_slice(b"MZ");
        data[0x3c..0x40].copy_from_slice(&(pe_offset as u32).to_le_bytes());
        data[pe_offset..pe_offset + 4].copy_from_slice(b"PE\0\0");
        data[pe_offset + 24..pe_offset + 26].copy_from_slice(&optional_magic.to_le_bytes());
        data
    }

    fn append_utf16le_version_pair(data: &mut Vec<u8>, key: &str, value: &str) {
        data.extend_from_slice(&[0u8; 6]);
        append_utf16le_nul_terminated(data, key);
        while !data.len().is_multiple_of(4) {
            data.push(0);
        }
        append_utf16le_nul_terminated(data, value);
    }

    fn append_utf16le_nul_terminated(data: &mut Vec<u8>, value: &str) {
        for unit in value.encode_utf16().chain(std::iter::once(0)) {
            data.extend_from_slice(&unit.to_le_bytes());
        }
    }

    fn append_ascii_nul_terminated(data: &mut Vec<u8>, value: &str) {
        data.extend_from_slice(value.as_bytes());
        data.push(0);
    }

    fn minimal_elf64_header() -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&[0x7f, b'E', b'L', b'F']);
        data.extend_from_slice(&[2, 1, 1, 0]); // 64-bit, little-endian, ELF v1
        data.extend_from_slice(&[0; 8]);
        data.extend_from_slice(&2u16.to_le_bytes()); // executable
        data.extend_from_slice(&0x3eu16.to_le_bytes()); // x86_64
        data.extend_from_slice(&1u32.to_le_bytes());
        data.extend_from_slice(&0x400000u64.to_le_bytes());
        data.extend_from_slice(&0u64.to_le_bytes()); // no program headers
        data.extend_from_slice(&0u64.to_le_bytes()); // no section headers
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&64u16.to_le_bytes());
        data.extend_from_slice(&56u16.to_le_bytes());
        data.extend_from_slice(&0u16.to_le_bytes());
        data.extend_from_slice(&64u16.to_le_bytes());
        data.extend_from_slice(&0u16.to_le_bytes());
        data.extend_from_slice(&0u16.to_le_bytes());
        data
    }
}
