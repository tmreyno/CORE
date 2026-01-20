// =============================================================================
// CORE-FFX - Forensic File Explorer
// Binary Analyzer - PE/ELF/Mach-O analysis for forensic investigation
// =============================================================================

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use goblin::Object;

use super::error::{DocumentError, DocumentResult};

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
    let data = fs::read(path)?;
    let file_size = data.len() as u64;
    
    let obj = Object::parse(&data)
        .map_err(|e| DocumentError::Parse(format!("Failed to parse binary: {}", e)))?;
    
    match obj {
        Object::PE(pe) => analyze_pe(pe, path, file_size),
        Object::Elf(elf) => analyze_elf(elf, path, file_size),
        Object::Mach(mach) => analyze_mach(mach, path, file_size),
        _ => Err(DocumentError::UnsupportedFormat("Not a recognized binary format".to_string())),
    }
}

fn analyze_pe(pe: goblin::pe::PE, path: &Path, file_size: u64) -> DocumentResult<BinaryInfo> {
    let is_64bit = pe.is_64;
    let format = if is_64bit { BinaryFormat::PE64 } else { BinaryFormat::PE32 };
    
    let architecture = match pe.header.coff_header.machine {
        0x8664 => "x86_64".to_string(),
        0x14c => "i386".to_string(),
        0xaa64 => "ARM64".to_string(),
        m => format!("0x{:04x}", m),
    };
    
    // Imports
    let imports: Vec<ImportInfo> = pe.imports
        .iter()
        .map(|imp| ImportInfo {
            library: imp.dll.to_string(),
            functions: vec![imp.name.to_string()],
            function_count: 1,
        })
        .collect();
    
    // Exports
    let exports: Vec<ExportInfo> = pe.exports
        .iter()
        .filter_map(|exp| {
            exp.name.map(|name| ExportInfo {
                name: name.to_string(),
                ordinal: None,
                address: exp.rva as u64,
            })
        })
        .collect();
    
    // Sections
    let sections: Vec<SectionInfo> = pe.sections
        .iter()
        .map(|sec| {
            let name = String::from_utf8_lossy(&sec.name).trim_end_matches('\0').to_string();
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
    
    Ok(BinaryInfo {
        path: path.to_string_lossy().to_string(),
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
        is_stripped: false,
        has_code_signing: false,
    })
}

fn analyze_elf(elf: goblin::elf::Elf, path: &Path, file_size: u64) -> DocumentResult<BinaryInfo> {
    let is_64bit = elf.is_64;
    let format = if is_64bit { BinaryFormat::ELF64 } else { BinaryFormat::ELF32 };
    
    let architecture = match elf.header.e_machine {
        0x3E => "x86_64".to_string(),
        0x03 => "i386".to_string(),
        0xB7 => "ARM64".to_string(),
        0x28 => "ARM".to_string(),
        m => format!("0x{:04x}", m),
    };
    
    // Imports (dynamic symbols that are undefined)
    let imports: Vec<ImportInfo> = elf.libraries
        .iter()
        .map(|lib| ImportInfo {
            library: lib.to_string(),
            functions: Vec::new(),
            function_count: 0,
        })
        .collect();
    
    // Exports (dynamic symbols that are defined)
    let exports: Vec<ExportInfo> = elf.dynsyms
        .iter()
        .filter(|sym| sym.st_value != 0 && !sym.is_import())
        .filter_map(|sym| {
            elf.dynstrtab.get_at(sym.st_name).map(|name| ExportInfo {
                name: name.to_string(),
                ordinal: None,
                address: sym.st_value,
            })
        })
        .collect();
    
    // Sections
    let sections: Vec<SectionInfo> = elf.section_headers
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
        .collect();
    
    Ok(BinaryInfo {
        path: path.to_string_lossy().to_string(),
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
        has_debug_info: elf.section_headers.iter().any(|s| {
            elf.shdr_strtab.get_at(s.sh_name)
                .map(|n| n.starts_with(".debug"))
                .unwrap_or(false)
        }),
        is_stripped: elf.syms.is_empty(),
        has_code_signing: false,
    })
}

fn analyze_mach(mach: goblin::mach::Mach, path: &Path, file_size: u64) -> DocumentResult<BinaryInfo> {
    match mach {
        goblin::mach::Mach::Binary(macho) => analyze_single_mach(macho, path, file_size),
        goblin::mach::Mach::Fat(fat) => {
            // For fat binaries, analyze the first arch
            if let Some(_arch) = fat.iter_arches().flatten().next() {
                Ok(BinaryInfo {
                    path: path.to_string_lossy().to_string(),
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
                    macho_cpu_type: Some(format!("Fat ({} architectures)", fat.narches)),
                    macho_filetype: None,
                    has_debug_info: false,
                    is_stripped: false,
                    has_code_signing: false,
                })
            } else {
                Err(DocumentError::Parse("Empty fat binary".to_string()))
            }
        }
    }
}

fn analyze_single_mach(macho: goblin::mach::MachO, path: &Path, file_size: u64) -> DocumentResult<BinaryInfo> {
    // Check if 64-bit by looking at magic number
    let is_64bit = matches!(macho.header.magic, 0xFEEDFACF | 0xCFFAEDFE);
    let format = if is_64bit { BinaryFormat::MachO64 } else { BinaryFormat::MachO32 };
    
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
    let imports: Vec<ImportInfo> = macho.libs
        .iter()
        .map(|lib| ImportInfo {
            library: lib.to_string(),
            functions: Vec::new(),
            function_count: 0,
        })
        .collect();
    
    // Exports
    let exports: Vec<ExportInfo> = macho.exports()
        .map_err(|e| DocumentError::Parse(format!("Failed to read exports: {}", e)))?
        .iter()
        .map(|exp| ExportInfo {
            name: exp.name.clone(),
            ordinal: None,
            address: exp.offset,
        })
        .collect();
    
    // Sections
    let sections: Vec<SectionInfo> = macho.segments
        .iter()
        .flat_map(|seg| seg.sections().ok().unwrap_or_default())
        .map(|(sec, _)| SectionInfo {
            name: format!("{},{}", sec.segname().unwrap_or("?"), sec.name().unwrap_or("?")),
            virtual_address: sec.addr,
            virtual_size: sec.size,
            raw_size: sec.size,
            characteristics: format!("0x{:08x}", sec.flags),
        })
        .collect();
    
    Ok(BinaryInfo {
        path: path.to_string_lossy().to_string(),
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
        has_debug_info: false,
        is_stripped: false,
        has_code_signing: false,
    })
}

/// Quick format detection without full parsing
pub fn detect_binary_format(path: impl AsRef<Path>) -> DocumentResult<BinaryFormat> {
    let data = fs::read(path.as_ref())?;
    if data.len() < 4 {
        return Ok(BinaryFormat::Unknown);
    }
    
    match Object::parse(&data) {
        Ok(Object::PE(pe)) => Ok(if pe.is_64 { BinaryFormat::PE64 } else { BinaryFormat::PE32 }),
        Ok(Object::Elf(elf)) => Ok(if elf.is_64 { BinaryFormat::ELF64 } else { BinaryFormat::ELF32 }),
        Ok(Object::Mach(goblin::mach::Mach::Binary(m))) => {
            Ok(if matches!(m.header.magic, 0xFEEDFACF | 0xCFFAEDFE) { BinaryFormat::MachO64 } else { BinaryFormat::MachO32 })
        }
        Ok(Object::Mach(goblin::mach::Mach::Fat(_))) => Ok(BinaryFormat::MachOFat),
        _ => Ok(BinaryFormat::Unknown),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_binary_format_enum() {
        let format = BinaryFormat::PE64;
        assert!(matches!(format, BinaryFormat::PE64));
    }
}
