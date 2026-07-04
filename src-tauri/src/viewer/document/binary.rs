// =============================================================================
// CORE-FFX - Forensic File Explorer
// Binary Analyzer - PE/ELF/Mach-O analysis for forensic investigation
// =============================================================================

use goblin::Object;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
    pub file_size: u64,
    // PE specific
    pub pe_timestamp: Option<u32>,
    pub pe_checksum: Option<u32>,
    pub pe_subsystem: Option<String>,
    // Mach-O specific
    pub macho_cpu_type: Option<String>,
    pub macho_filetype: Option<String>,
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
    let obj = Object::parse(data)
        .map_err(|e| DocumentError::Parse(format!("Failed to parse binary: {}", e)))?;

    match obj {
        Object::PE(pe) => analyze_pe(pe, &source_id, file_size),
        Object::Elf(elf) => analyze_elf(elf, &source_id, file_size),
        Object::Mach(mach) => analyze_mach(mach, &source_id, data, file_size),
        _ => Err(DocumentError::UnsupportedFormat(
            "Not a recognized binary format".to_string(),
        )),
    }
}

fn analyze_pe(pe: goblin::pe::PE, source_id: &str, file_size: u64) -> DocumentResult<BinaryInfo> {
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
            }
        })
        .collect();

    // Optional header info
    let (timestamp, checksum, subsystem) = if let Some(opt) = pe.header.optional_header {
        let sub = match opt.windows_fields.subsystem {
            1 => "Native",
            2 => "GUI",
            3 => "Console",
            _ => "Unknown",
        };
        (
            Some(pe.header.coff_header.time_date_stamp),
            Some(opt.windows_fields.check_sum),
            Some(sub.to_string()),
        )
    } else {
        (Some(pe.header.coff_header.time_date_stamp), None, None)
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

    Ok(BinaryInfo {
        path: source_id.to_string(),
        format,
        architecture,
        is_64bit,
        entry_point: Some(pe.entry as u64),
        imports,
        exports,
        sections,
        file_size,
        pe_timestamp: timestamp,
        pe_checksum: checksum,
        pe_subsystem: subsystem,
        macho_cpu_type: None,
        macho_filetype: None,
        has_debug_info: pe.debug_data.is_some(),
        is_stripped,
        has_code_signing,
    })
}

fn analyze_elf(
    elf: goblin::elf::Elf,
    source_id: &str,
    file_size: u64,
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

    Ok(BinaryInfo {
        path: source_id.to_string(),
        format,
        architecture,
        is_64bit,
        entry_point: Some(elf.entry),
        imports,
        exports,
        sections,
        file_size,
        pe_timestamp: None,
        pe_checksum: None,
        pe_subsystem: None,
        macho_cpu_type: None,
        macho_filetype: None,
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
) -> DocumentResult<BinaryInfo> {
    match mach {
        goblin::mach::Mach::Binary(macho) => analyze_single_mach(macho, source_id, file_size),
        goblin::mach::Mach::Fat(fat) => {
            let narches = fat.narches;

            // Try to parse and analyze the first architecture fully
            if let Some(arch) = fat.iter_arches().flatten().next() {
                if let Some(slice) = checked_u64_slice(data, arch.offset as u64, arch.size as u64) {
                    if let Ok(Object::Mach(goblin::mach::Mach::Binary(inner))) =
                        Object::parse(slice)
                    {
                        let mut info = analyze_single_mach(inner, source_id, file_size)?;
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
                file_size,
                pe_timestamp: None,
                pe_checksum: None,
                pe_subsystem: None,
                macho_cpu_type: Some(format!("Fat ({} architectures)", narches)),
                macho_filetype: None,
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

fn analyze_single_mach(
    macho: goblin::mach::MachO,
    source_id: &str,
    file_size: u64,
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
        file_size,
        pe_timestamp: None,
        pe_checksum: None,
        pe_subsystem: None,
        macho_cpu_type: Some(cpu_type),
        macho_filetype: Some(filetype.to_string()),
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
