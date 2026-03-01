// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Test E01 random access reading
//!
//! This example demonstrates reading bytes from an E01 disk image
//! at arbitrary offsets - the foundation for filesystem parsing.
//!
//! Usage: cargo run --example test_e01_read -- /path/to/image.E01

use ffx_check_lib::ewf::EwfHandle;

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <e01_path>", args[0]);
        eprintln!("\nExample: {} /path/to/disk.E01", args[0]);
        std::process::exit(1);
    }

    let path = &args[1];
    println!("Opening E01: {}\n", path);

    let mut handle = EwfHandle::open(path)?;

    // Get media info
    let media_size = handle.get_media_size();
    let chunk_size = handle.get_chunk_size();
    let volume = handle.get_volume_info();

    println!("=== E01 Media Information ===");
    println!(
        "Media size:       {} bytes ({:.2} GB)",
        media_size,
        media_size as f64 / 1_073_741_824.0
    );
    println!("Sector size:      {} bytes", volume.bytes_per_sector);
    println!("Sector count:     {}", volume.sector_count);
    println!("Chunk size:       {} bytes", chunk_size);
    println!("Chunks per image: {}", handle.get_chunk_count());
    println!();

    // Read first 512 bytes (MBR/Boot sector)
    println!("=== Reading Master Boot Record (first 512 bytes) ===");
    let mbr = handle.read_at(0, 512)?;

    // Check for MBR signature (0x55AA at offset 510)
    if mbr.len() >= 512 && mbr[510] == 0x55 && mbr[511] == 0xAA {
        println!("✓ Valid MBR signature found (0x55AA)");

        // Parse partition table (offset 446, 4 entries of 16 bytes each)
        println!("\n=== Partition Table ===");
        for i in 0..4 {
            let offset = 446 + (i * 16);
            let boot_flag = mbr[offset];
            let partition_type = mbr[offset + 4];
            let start_lba = u32::from_le_bytes([
                mbr[offset + 8],
                mbr[offset + 9],
                mbr[offset + 10],
                mbr[offset + 11],
            ]);
            let size_sectors = u32::from_le_bytes([
                mbr[offset + 12],
                mbr[offset + 13],
                mbr[offset + 14],
                mbr[offset + 15],
            ]);

            if partition_type != 0 {
                let type_name = match partition_type {
                    0x01 => "FAT12",
                    0x04 | 0x06 | 0x0E => "FAT16",
                    0x07 => "NTFS/exFAT",
                    0x0B | 0x0C => "FAT32",
                    0x0F | 0x05 => "Extended",
                    0x82 => "Linux swap",
                    0x83 => "Linux",
                    0xAF => "HFS+",
                    0xEE => "GPT Protective MBR",
                    0xEF => "EFI System",
                    _ => "Unknown",
                };

                let size_bytes = size_sectors as u64 * 512;
                println!(
                    "Partition {}: Type=0x{:02X} ({}) Boot={} Start=LBA {} Size={:.2} GB",
                    i + 1,
                    partition_type,
                    type_name,
                    if boot_flag == 0x80 { "Yes" } else { "No" },
                    start_lba,
                    size_bytes as f64 / 1_073_741_824.0
                );

                // Try to read the partition's boot sector
                if partition_type == 0x07 || partition_type == 0x0B || partition_type == 0x0C {
                    let partition_offset = start_lba as u64 * 512;
                    println!(
                        "  → Reading partition boot sector at offset {}...",
                        partition_offset
                    );

                    if let Ok(boot_sector) = handle.read_at(partition_offset, 512) {
                        // Check for NTFS
                        if boot_sector.len() >= 8 && boot_sector[3..7] == *b"NTFS" {
                            println!("  → NTFS filesystem detected!");

                            // Parse NTFS BPB
                            let bytes_per_sector =
                                u16::from_le_bytes([boot_sector[11], boot_sector[12]]);
                            let sectors_per_cluster = boot_sector[13];
                            let total_sectors = u64::from_le_bytes([
                                boot_sector[40],
                                boot_sector[41],
                                boot_sector[42],
                                boot_sector[43],
                                boot_sector[44],
                                boot_sector[45],
                                boot_sector[46],
                                boot_sector[47],
                            ]);

                            println!("  → Bytes/sector: {}", bytes_per_sector);
                            println!("  → Sectors/cluster: {}", sectors_per_cluster);
                            println!("  → Total sectors: {}", total_sectors);

                            // Read $MFT location
                            let mft_cluster = u64::from_le_bytes([
                                boot_sector[48],
                                boot_sector[49],
                                boot_sector[50],
                                boot_sector[51],
                                boot_sector[52],
                                boot_sector[53],
                                boot_sector[54],
                                boot_sector[55],
                            ]);
                            let mft_offset = partition_offset
                                + (mft_cluster
                                    * sectors_per_cluster as u64
                                    * bytes_per_sector as u64);
                            println!(
                                "  → $MFT at cluster {} (offset {})",
                                mft_cluster, mft_offset
                            );

                            // Read first MFT entry ($MFT file record)
                            if let Ok(mft_entry) = handle.read_at(mft_offset, 1024) {
                                if mft_entry.len() >= 4 && mft_entry[0..4] == *b"FILE" {
                                    println!("  → ✓ Valid $MFT entry found (FILE signature)");
                                }
                            }
                        }

                        // Check for FAT32
                        if boot_sector.len() >= 90 && boot_sector[82..87] == *b"FAT32" {
                            println!("  → FAT32 filesystem detected!");
                        }
                    }
                }
            }
        }
    } else if mbr.len() >= 8 && mbr[0..8] == *b"EFI PART" {
        println!("Note: GPT header found (not MBR)");
    } else {
        println!("Note: No standard MBR signature found");
        println!("First 16 bytes: {:02X?}", &mbr[..16.min(mbr.len())]);
    }

    // Show hex dump of first 64 bytes
    println!("\n=== First 64 bytes (hex dump) ===");
    for row in 0..4 {
        let offset = row * 16;
        print!("{:08X}  ", offset);
        for col in 0..16 {
            print!("{:02X} ", mbr[offset + col]);
        }
        print!(" |");
        for col in 0..16 {
            let byte = mbr[offset + col];
            if byte.is_ascii_graphic() || byte == b' ' {
                print!("{}", byte as char);
            } else {
                print!(".");
            }
        }
        println!("|");
    }

    println!("\n✓ E01 random access test complete!");
    Ok(())
}
