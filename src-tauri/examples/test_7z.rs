// Test 7z archive reading (including split archives)
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use sevenz_rust::{SevenZReader, Password};

/// A reader that concatenates multiple files (for split archives)
struct MultiFileReader {
    files: Vec<(String, u64)>,  // (path, size)
    current_file: Option<BufReader<File>>,
    current_index: usize,
    current_pos_in_file: u64,
    total_pos: u64,
    total_size: u64,
}

impl MultiFileReader {
    fn new(paths: Vec<String>) -> std::io::Result<Self> {
        let mut files = Vec::new();
        let mut total_size = 0u64;
        
        for path in &paths {
            let meta = std::fs::metadata(path)?;
            let size = meta.len();
            files.push((path.clone(), size));
            total_size += size;
        }
        
        let first_file = if !files.is_empty() {
            Some(BufReader::new(File::open(&files[0].0)?))
        } else {
            None
        };
        
        Ok(Self {
            files,
            current_file: first_file,
            current_index: 0,
            current_pos_in_file: 0,
            total_pos: 0,
            total_size,
        })
    }
    
    fn total_size(&self) -> u64 {
        self.total_size
    }
}

impl Read for MultiFileReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        loop {
            if let Some(ref mut file) = self.current_file {
                let n = file.read(buf)?;
                if n > 0 {
                    self.current_pos_in_file += n as u64;
                    self.total_pos += n as u64;
                    return Ok(n);
                }
            }
            
            // Move to next file
            self.current_index += 1;
            if self.current_index >= self.files.len() {
                return Ok(0); // EOF
            }
            
            self.current_file = Some(BufReader::new(File::open(&self.files[self.current_index].0)?));
            self.current_pos_in_file = 0;
        }
    }
}

impl Seek for MultiFileReader {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let new_pos = match pos {
            SeekFrom::Start(p) => p,
            SeekFrom::End(p) => (self.total_size as i64 + p) as u64,
            SeekFrom::Current(p) => (self.total_pos as i64 + p) as u64,
        };
        
        // Find which file contains this position
        let mut cumulative = 0u64;
        for (i, (path, size)) in self.files.iter().enumerate() {
            if new_pos < cumulative + size {
                // Position is in this file
                if i != self.current_index {
                    self.current_file = Some(BufReader::new(File::open(path)?));
                    self.current_index = i;
                }
                let pos_in_file = new_pos - cumulative;
                self.current_file.as_mut().unwrap().seek(SeekFrom::Start(pos_in_file))?;
                self.current_pos_in_file = pos_in_file;
                self.total_pos = new_pos;
                return Ok(new_pos);
            }
            cumulative += size;
        }
        
        // Position is at or past the end
        if let Some((path, _)) = self.files.last() {
            self.current_index = self.files.len() - 1;
            self.current_file = Some(BufReader::new(File::open(path)?));
            self.current_file.as_mut().unwrap().seek(SeekFrom::End(0))?;
        }
        self.total_pos = self.total_size;
        Ok(self.total_size)
    }
}

fn find_split_archive_parts(first_part: &str) -> Vec<String> {
    let mut parts = Vec::new();
    
    // Check if it's a .7z.001 pattern
    if let Some(base) = first_part.strip_suffix(".001") {
        let mut num = 1;
        loop {
            let part_path = format!("{}.{:03}", base, num);
            if std::path::Path::new(&part_path).exists() {
                parts.push(part_path);
                num += 1;
            } else {
                break;
            }
        }
    } else {
        parts.push(first_part.to_string());
    }
    
    parts
}

fn main() {
    let path = "/Users/terryreynolds/1827-1001 Case With Data /3.Exports.Results/1.Export.For.Review/25-053 08282-1003 (NESHAP - ABC Kid City)/Non-PACP.7z.001";
    println!("Testing segmented 7z file: {}", path);
    
    let parts = find_split_archive_parts(path);
    println!("Found {} archive parts:", parts.len());
    for (i, p) in parts.iter().enumerate() {
        let size = std::fs::metadata(p).map(|m| m.len()).unwrap_or(0);
        println!("  Part {}: {} ({:.2} GB)", i + 1, p.split('/').next_back().unwrap_or(p), size as f64 / 1_073_741_824.0);
    }
    
    match MultiFileReader::new(parts) {
        Ok(mut reader) => {
            let total_size = reader.total_size();
            println!("\nTotal archive size: {:.2} GB", total_size as f64 / 1_073_741_824.0);
            
            println!("\nOpening 7z archive (this may take a moment for large archives)...");
            match SevenZReader::new(&mut reader, total_size, Password::empty()) {
                Ok(sz_reader) => {
                    println!("✅ Segmented 7z archive opened successfully!");
                    
                    let files = &sz_reader.archive().files;
                    let total = files.len();
                    let dirs = files.iter().filter(|e| e.is_directory()).count();
                    let file_count = total - dirs;
                    
                    println!("\nArchive contains: {} files, {} directories", file_count, dirs);
                    
                    // Show directory structure summary
                    println!("\n=== TOP-LEVEL STRUCTURE ===");
                    let mut top_level: std::collections::HashSet<String> = std::collections::HashSet::new();
                    for entry in files.iter() {
                        let name = entry.name();
                        let parts: Vec<&str> = name.split('/').collect();
                        if parts.len() >= 2 {
                            top_level.insert(format!("{}/{}", parts[0], parts[1]));
                        } else if !parts.is_empty() {
                            top_level.insert(parts[0].to_string());
                        }
                    }
                    let mut top_vec: Vec<_> = top_level.into_iter().collect();
                    top_vec.sort();
                    for item in &top_vec {
                        println!("  📁 {}/", item);
                    }
                    
                    // Show some actual files with sizes
                    println!("\n=== SAMPLE FILES (first 100 non-directory entries) ===");
                    let mut file_shown = 0;
                    for entry in files.iter() {
                        if !entry.is_directory() {
                            let name = entry.name();
                            let size = entry.size();
                            let size_str = if size > 1_000_000_000 {
                                format!("{:.2} GB", size as f64 / 1_073_741_824.0)
                            } else if size > 1_000_000 {
                                format!("{:.2} MB", size as f64 / 1_048_576.0)
                            } else if size > 1000 {
                                format!("{:.1} KB", size as f64 / 1024.0)
                            } else {
                                format!("{} B", size)
                            };
                            println!("  📄 {} ({})", name, size_str);
                            file_shown += 1;
                            if file_shown >= 100 {
                                println!("  ... ({} more files)", file_count - 100);
                                break;
                            }
                        }
                    }
                    
                    // Show file type breakdown
                    println!("\n=== FILE TYPES ===");
                    let mut ext_counts: std::collections::HashMap<String, (usize, u64)> = std::collections::HashMap::new();
                    for entry in files.iter() {
                        if !entry.is_directory() {
                            let name = entry.name();
                            let ext = std::path::Path::new(name)
                                .extension()
                                .map(|e| e.to_string_lossy().to_lowercase())
                                .unwrap_or_else(|| "(no ext)".to_string());
                            let e = ext_counts.entry(ext).or_insert((0, 0));
                            e.0 += 1;
                            e.1 += entry.size();
                        }
                    }
                    let mut ext_vec: Vec<_> = ext_counts.into_iter().collect();
                    ext_vec.sort_by(|a, b| b.1.0.cmp(&a.1.0)); // Sort by count
                    for (ext, (count, total_size)) in ext_vec.iter().take(20) {
                        let size_str = if *total_size > 1_000_000_000 {
                            format!("{:.2} GB", *total_size as f64 / 1_073_741_824.0)
                        } else if *total_size > 1_000_000 {
                            format!("{:.2} MB", *total_size as f64 / 1_048_576.0)
                        } else {
                            format!("{:.1} KB", *total_size as f64 / 1024.0)
                        };
                        println!("  .{:<10} {:>5} files  ({} total)", ext, count, size_str);
                    }
                    if ext_vec.len() > 20 {
                        println!("  ... and {} more file types", ext_vec.len() - 20);
                    }
                    
                    println!("\n✅ Segmented 7z support working correctly!");
                }
                Err(e) => {
                    eprintln!("❌ Failed to open 7z archive: {:?}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to open file parts: {:?}", e);
        }
    }
}
