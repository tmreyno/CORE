//! High-level archive operations
//!
//! Safe Rust API for 7z archive creation, extraction, and inspection with full
//! encryption support.

use crate::error::{Error, Result};
use crate::ffi;
use std::ffi::{CStr, CString};
use std::path::Path;
use std::ptr;

/// Compression level for archive operations
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CompressionLevel {
    /// No compression (store only)
    Store,
    /// Fastest compression
    Fastest,
    /// Fast compression
    Fast,
    /// Normal compression (balanced)
    Normal,
    /// Maximum compression
    Maximum,
    /// Ultra compression (slowest, best ratio)
    Ultra,
}

impl From<CompressionLevel> for ffi::SevenZipCompressionLevel {
    fn from(level: CompressionLevel) -> Self {
        match level {
            CompressionLevel::Store => ffi::SevenZipCompressionLevel::SEVENZIP_LEVEL_STORE,
            CompressionLevel::Fastest => ffi::SevenZipCompressionLevel::SEVENZIP_LEVEL_FASTEST,
            CompressionLevel::Fast => ffi::SevenZipCompressionLevel::SEVENZIP_LEVEL_FAST,
            CompressionLevel::Normal => ffi::SevenZipCompressionLevel::SEVENZIP_LEVEL_NORMAL,
            CompressionLevel::Maximum => ffi::SevenZipCompressionLevel::SEVENZIP_LEVEL_MAXIMUM,
            CompressionLevel::Ultra => ffi::SevenZipCompressionLevel::SEVENZIP_LEVEL_ULTRA,
        }
    }
}

/// Archive entry information
#[derive(Debug, Clone)]
pub struct ArchiveEntry {
    /// File name (UTF-8)
    pub name: String,
    /// Uncompressed size in bytes
    pub size: u64,
    /// Compressed size in bytes
    pub packed_size: u64,
    /// Unix timestamp of last modification
    pub modified_time: u64,
    /// File attributes
    pub attributes: u32,
    /// True if this is a directory
    pub is_directory: bool,
}

impl ArchiveEntry {
    /// Get compression ratio as a percentage (0-100)
    pub fn compression_ratio(&self) -> f64 {
        if self.size == 0 {
            0.0
        } else {
            (1.0 - (self.packed_size as f64 / self.size as f64)) * 100.0
        }
    }
}

/// Progress callback closure type
pub type ProgressCallback = Box<dyn FnMut(u64, u64) + Send>;

/// Byte-level progress callback closure type  
/// Parameters: (bytes_processed, bytes_total, current_file_bytes, current_file_total, current_file_name)
pub type BytesProgressCallback = Box<dyn FnMut(u64, u64, u64, u64, &str) + Send>;

/// Calculate Shannon entropy for data compressibility detection
/// Returns value between 0.0 (very compressible) and 1.0 (incompressible)
fn calculate_entropy(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    
    let mut counts = [0u32; 256];
    for &byte in data {
        counts[byte as usize] += 1;
    }
    
    let len = data.len() as f64;
    let mut entropy = 0.0;
    
    for &count in &counts {
        if count > 0 {
            let p = count as f64 / len;
            entropy -= p * p.log2();
        }
    }
    
    // Normalize to 0-1 range (max entropy for byte is 8)
    entropy / 8.0
}

/// Analyze file to determine if compression is worthwhile
/// Returns (entropy, recommended_compression_level)
pub fn analyze_file_compressibility(file_path: &Path) -> std::io::Result<(f64, CompressionLevel)> {
    use std::fs::File;
    use std::io::Read;
    
    let metadata = std::fs::metadata(file_path)?;
    let file_size = metadata.len();
    
    // Sample size: 64KB or 5% of file, whichever is smaller
    let sample_size = ((file_size / 20).min(65536).max(4096)) as usize;
    
    let mut file = File::open(file_path)?;
    let mut buffer = vec![0u8; sample_size];
    let bytes_read = file.read(&mut buffer)?;
    buffer.truncate(bytes_read);
    
    let entropy = calculate_entropy(&buffer);
    
    // Determine compression level based on entropy
    let recommended_level = match entropy {
        e if e > 0.95 => CompressionLevel::Store,    // Already compressed/encrypted
        e if e > 0.85 => CompressionLevel::Fastest,  // Low compression potential
        e if e > 0.70 => CompressionLevel::Fast,     // Moderate compression
        e if e > 0.50 => CompressionLevel::Normal,   // Good compression potential
        _ => CompressionLevel::Maximum,              // High compression potential
    };
    
    Ok((entropy, recommended_level))
}

/// Calculate optimal thread count based on total data size
/// Returns recommended thread count considering overhead vs benefit
pub fn calculate_optimal_threads(total_bytes: u64) -> usize {
    let available_cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    
    // Thresholds determined from benchmark data:
    // - <1MB: Single thread fastest (no threading overhead)
    // - 1-10MB: 2 threads provides minimal benefit
    // - 10-50MB: 4 threads good balance
    // - >50MB: Scale with cores
    
    match total_bytes {
        0..=1_048_576 => 1,                          // <1MB
        1_048_577..=10_485_760 => 2,                 // 1-10MB
        10_485_761..=52_428_800 => {                 // 10-50MB
            available_cores.min(4)
        },
        52_428_801..=524_288_000 => {                // 50-500MB
            available_cores.min(8)
        },
        _ => available_cores.min(16),                // >500MB
    }
}

/// Get total size of all files to be compressed
fn calculate_total_size(file_paths: &[&str]) -> std::io::Result<u64> {
    let mut total = 0u64;
    for path in file_paths {
        let metadata = std::fs::metadata(path)?;
        if metadata.is_file() {
            total += metadata.len();
        }
    }
    Ok(total)
}

/// Advanced compression options
#[derive(Debug, Clone)]
pub struct CompressOptions {
    /// Number of threads to use (0 = auto-detect)
    pub num_threads: usize,
    /// Dictionary size in bytes (0 = auto)
    pub dict_size: u64,
    /// Create solid archive (better compression)
    pub solid: bool,
    /// Optional password for encryption
    pub password: Option<String>,
    /// Auto-detect and skip compression for incompressible data
    pub auto_detect_incompressible: bool,
}

impl Default for CompressOptions {
    fn default() -> Self {
        Self {
            num_threads: 0, // auto-detect
            dict_size: 0,   // auto
            solid: true,
            password: None,
            auto_detect_incompressible: false, // Conservative default
        }
    }
}

impl CompressOptions {
    /// Create options with auto-tuned thread count based on file sizes
    pub fn auto_tuned(file_paths: &[&str]) -> std::io::Result<Self> {
        let total_size = calculate_total_size(file_paths)?;
        let optimal_threads = calculate_optimal_threads(total_size);
        
        Ok(Self {
            num_threads: optimal_threads,
            dict_size: 0,
            solid: true,
            password: None,
            auto_detect_incompressible: true, // Enable by default for smart mode
        })
    }
    
    /// Enable auto-detection with method chaining
    pub fn with_auto_detect(mut self, enable: bool) -> Self {
        self.auto_detect_incompressible = enable;
        self
    }
    
    /// Set thread count with method chaining
    pub fn with_threads(mut self, threads: usize) -> Self {
        self.num_threads = threads;
        self
    }
    
    /// Set password with method chaining
    pub fn with_password(mut self, password: String) -> Self {
        self.password = Some(password);
        self
    }
}

/// Streaming compression options for large files and split archives
#[derive(Debug, Clone)]
pub struct StreamOptions {
    /// Number of threads to use (0 = auto-detect)
    pub num_threads: usize,
    /// Dictionary size in bytes (0 = auto)
    pub dict_size: u64,
    /// Create solid archive
    pub solid: bool,
    /// Optional password for encryption
    pub password: Option<String>,
    /// Split archive size in bytes (0 = no split, e.g., 4GB = 4_294_967_296)
    pub split_size: u64,
    /// Chunk size for streaming (0 = auto)
    pub chunk_size: u64,
    /// Temporary directory (None = system default)
    pub temp_dir: Option<String>,
    /// Delete temporary files on error
    pub delete_temp_on_error: bool,
}

impl Default for StreamOptions {
    fn default() -> Self {
        Self {
            num_threads: 0,
            dict_size: 0,
            solid: true,
            password: None,
            split_size: 0,
            chunk_size: 0,
            temp_dir: None,
            delete_temp_on_error: true,
        }
    }
}

/// Main 7z archive interface
pub struct SevenZip {
    _initialized: bool,
}

impl SevenZip {
    /// Create a new SevenZip instance
    ///
    /// Initializes the underlying 7z library.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use seven_zip::SevenZip;
    ///
    /// let sz = SevenZip::new()?;
    /// # Ok::<(), seven_zip::Error>(())
    /// ```
    pub fn new() -> Result<Self> {
        unsafe {
            let result = ffi::sevenzip_init();
            if result != ffi::SevenZipErrorCode::SEVENZIP_OK {
                return Err(Error::from_code(result));
            }
        }
        Ok(Self { _initialized: true })
    }

    /// Extract a 7z archive
    ///
    /// # Arguments
    ///
    /// * `archive_path` - Path to the archive file
    /// * `output_dir` - Directory to extract to
    ///
    /// # Example
    ///
    /// ```no_run
    /// use seven_zip::SevenZip;
    ///
    /// let sz = SevenZip::new()?;
    /// sz.extract("archive.7z", "output")?;
    /// # Ok::<(), seven_zip::Error>(())
    /// ```
    pub fn extract(&self, archive_path: impl AsRef<Path>, output_dir: impl AsRef<Path>) -> Result<()> {
        self.extract_with_password(archive_path, output_dir, None, None)
    }

    /// Extract a 7z archive with password and progress callback
    ///
    /// # Arguments
    ///
    /// * `archive_path` - Path to the archive file
    /// * `output_dir` - Directory to extract to
    /// * `password` - Optional password for encrypted archives
    /// * `progress` - Optional progress callback
    ///
    /// # Example
    ///
    /// ```no_run
    /// use seven_zip::SevenZip;
    ///
    /// let sz = SevenZip::new()?;
    /// sz.extract_with_password(
    ///     "archive.7z",
    ///     "output",
    ///     Some("password"),
    ///     Some(Box::new(|completed, total| {
    ///         println!("Progress: {}/{} bytes", completed, total);
    ///     }))
    /// )?;
    /// # Ok::<(), seven_zip::Error>(())
    /// ```
    pub fn extract_with_password(
        &self,
        archive_path: impl AsRef<Path>,
        output_dir: impl AsRef<Path>,
        password: Option<&str>,
        progress: Option<ProgressCallback>,
    ) -> Result<()> {
        let archive_path_c = path_to_cstring(archive_path.as_ref())?;
        let output_dir_c = path_to_cstring(output_dir.as_ref())?;
        let password_c = password.map(|p| CString::new(p)).transpose()?;

        let (callback, user_data) = if let Some(cb) = progress {
            // Convert Box<dyn FnMut> into raw pointer that can cross FFI boundary
            let boxed = Box::new(cb);
            let raw = Box::into_raw(boxed);
            (
                Some(progress_callback_wrapper as unsafe extern "C" fn(u64, u64, *mut std::os::raw::c_void)),
                raw as *mut std::os::raw::c_void,
            )
        } else {
            (None, ptr::null_mut())
        };

        unsafe {
            let result = ffi::sevenzip_extract(
                archive_path_c.as_ptr(),
                output_dir_c.as_ptr(),
                password_c.as_ref().map_or(ptr::null(), |p| p.as_ptr()),
                callback,
                user_data,
            );

            // Clean up the callback if it was allocated
            if !user_data.is_null() {
                let _boxed = Box::from_raw(user_data as *mut ProgressCallback);
                // Drops automatically
            }

            if result != ffi::SevenZipErrorCode::SEVENZIP_OK {
                return Err(Error::from_code(result));
            }
        }

        Ok(())
    }

    /// Extract specific files from an archive
    ///
    /// # Arguments
    ///
    /// * `archive_path` - Path to the archive file
    /// * `output_dir` - Directory to extract to
    /// * `files` - List of files to extract
    /// * `password` - Optional password
    ///
    /// # Example
    ///
    /// ```no_run
    /// use seven_zip::SevenZip;
    ///
    /// let sz = SevenZip::new()?;
    /// sz.extract_files(
    ///     "archive.7z",
    ///     "output",
    ///     &["file1.txt", "dir/file2.txt"],
    ///     None
    /// )?;
    /// # Ok::<(), seven_zip::Error>(())
    /// ```
    pub fn extract_files(
        &self,
        archive_path: impl AsRef<Path>,
        output_dir: impl AsRef<Path>,
        files: &[&str],
        password: Option<&str>,
    ) -> Result<()> {
        let archive_path_c = path_to_cstring(archive_path.as_ref())?;
        let output_dir_c = path_to_cstring(output_dir.as_ref())?;
        let password_c = password.map(|p| CString::new(p)).transpose()?;

        // Convert file list to C string array
        let files_c: Vec<CString> = files
            .iter()
            .map(|&f| CString::new(f))
            .collect::<std::result::Result<_, _>>()?;
        let mut files_ptrs: Vec<*const i8> = files_c.iter().map(|s| s.as_ptr()).collect();
        files_ptrs.push(ptr::null()); // NULL-terminate

        unsafe {
            let result = ffi::sevenzip_extract_files(
                archive_path_c.as_ptr(),
                output_dir_c.as_ptr(),
                files_ptrs.as_ptr(),
                password_c.as_ref().map_or(ptr::null(), |p| p.as_ptr()),
                None,
                ptr::null_mut(),
            );

            if result != ffi::SevenZipErrorCode::SEVENZIP_OK {
                return Err(Error::from_code(result));
            }
        }

        Ok(())
    }

    /// List contents of an archive
    ///
    /// # Arguments
    ///
    /// * `archive_path` - Path to the archive file
    /// * `password` - Optional password for encrypted archives
    ///
    /// # Returns
    ///
    /// Vec of ArchiveEntry with information about each file
    ///
    /// # Example
    ///
    /// ```no_run
    /// use seven_zip::SevenZip;
    ///
    /// let sz = SevenZip::new()?;
    /// let entries = sz.list("archive.7z", None)?;
    /// for entry in entries {
    ///     println!("{}: {} bytes ({}% compressed)",
    ///         entry.name, entry.size, entry.compression_ratio());
    /// }
    /// # Ok::<(), seven_zip::Error>(())
    /// ```
    pub fn list(&self, archive_path: impl AsRef<Path>, password: Option<&str>) -> Result<Vec<ArchiveEntry>> {
        let archive_path_c = path_to_cstring(archive_path.as_ref())?;
        let password_c = password.map(|p| CString::new(p)).transpose()?;

        let mut list_ptr: *mut ffi::SevenZipList = ptr::null_mut();

        unsafe {
            let result = ffi::sevenzip_list(
                archive_path_c.as_ptr(),
                password_c.as_ref().map_or(ptr::null(), |p| p.as_ptr()),
                &mut list_ptr as *mut *mut ffi::SevenZipList,
            );

            if result != ffi::SevenZipErrorCode::SEVENZIP_OK {
                return Err(Error::from_code(result));
            }

            if list_ptr.is_null() {
                return Ok(Vec::new());
            }

            let list = &*list_ptr;
            let mut entries = Vec::with_capacity(list.count);

            for i in 0..list.count {
                let entry = &*list.entries.add(i);
                let name = CStr::from_ptr(entry.name).to_string_lossy().into_owned();

                entries.push(ArchiveEntry {
                    name,
                    size: entry.size,
                    packed_size: entry.packed_size,
                    modified_time: entry.modified_time,
                    attributes: entry.attributes,
                    is_directory: entry.is_directory != 0,
                });
            }

            ffi::sevenzip_free_list(list_ptr);
            Ok(entries)
        }
    }

    /// Create a standard 7z archive
    ///
    /// **WARNING**: This function loads entire files into memory before compression.
    /// For files larger than ~1GB, use [`create_archive_streaming`](Self::create_archive_streaming)
    /// instead to avoid memory exhaustion.
    ///
    /// # Arguments
    ///
    /// * `archive_path` - Output archive path
    /// * `input_paths` - Files/directories to compress
    /// * `level` - Compression level
    /// * `options` - Optional compression options
    ///
    /// # Memory Warning
    ///
    /// This function is NOT suitable for large files (82GB evidence directories, etc.).
    /// It will attempt to load entire files into RAM and may cause system instability.
    /// Use `create_archive_streaming()` for large files.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use seven_zip::{SevenZip, CompressionLevel, CompressOptions};
    ///
    /// let sz = SevenZip::new()?;
    /// let mut opts = CompressOptions::default();
    /// opts.num_threads = 4;
    /// opts.password = Some("secret".to_string());
    ///
    /// sz.create_archive(
    ///     "archive.7z",
    ///     &["file1.txt", "file2.txt", "directory"],
    ///     CompressionLevel::Normal,
    ///     Some(&opts)
    /// )?;
    /// # Ok::<(), seven_zip::Error>(())
    /// ```
    pub fn create_archive(
        &self,
        archive_path: impl AsRef<Path>,
        input_paths: &[impl AsRef<Path>],
        level: CompressionLevel,
        options: Option<&CompressOptions>,
    ) -> Result<()> {
        // Smart defaults: auto-tune if no options provided
        let mut opts = options.cloned().unwrap_or_default();
        
        // Check total size and warn if it's large
        let mut total_size: u64 = 0;
        for path in input_paths {
            if let Ok(metadata) = std::fs::metadata(path.as_ref()) {
                if metadata.is_dir() {
                    // Estimate directory size (walk would be expensive, just warn)
                    eprintln!("WARNING: create_archive() loads files into memory.");
                    eprintln!("For large directories, use create_archive_streaming() instead.");
                } else {
                    total_size += metadata.len();
                }
            }
        }
        
        // Warn if total size exceeds 1GB
        const MAX_SAFE_SIZE: u64 = 1024 * 1024 * 1024; // 1GB
        if total_size > MAX_SAFE_SIZE {
            eprintln!("WARNING: Total input size is {:.2} GB", total_size as f64 / 1e9);
            eprintln!("This may exhaust system memory. Consider using create_archive_streaming().");
        }
        
        // Auto-tune threads if not explicitly set (num_threads == 0)
        if opts.num_threads == 0 && total_size > 0 {
            opts.num_threads = calculate_optimal_threads(total_size);
        }
        
        // Auto-detect incompressible data if enabled and single file
        let effective_level = if opts.auto_detect_incompressible && input_paths.len() == 1 {
            let path = input_paths[0].as_ref();
            if let Ok(metadata) = std::fs::metadata(path) {
                if metadata.is_file() {
                    match analyze_file_compressibility(path) {
                        Ok((entropy, recommended)) if entropy > 0.95 => {
                            eprintln!("Info: Data appears incompressible (entropy: {:.2}), using Store mode", entropy);
                            CompressionLevel::Store
                        },
                        Ok((entropy, _)) if entropy > 0.85 => {
                            eprintln!("Info: Low compression potential detected (entropy: {:.2})", entropy);
                            level
                        }
                        _ => level,
                    }
                } else {
                    level
                }
            } else {
                level
            }
        } else {
            level
        };
        
        let archive_path_c = path_to_cstring(archive_path.as_ref())?;
        
        // Convert input paths to C strings
        let input_paths_c: Vec<CString> = input_paths
            .iter()
            .map(|p| path_to_cstring(p.as_ref()))
            .collect::<Result<_>>()?;
        let mut input_ptrs: Vec<*const i8> = input_paths_c.iter().map(|s| s.as_ptr()).collect();
        input_ptrs.push(ptr::null()); // NULL-terminate

        // Convert options to C struct
        let password_c = opts.password.as_ref().map(|p| CString::new(p.as_str())).transpose()?;
        let c_opts = ffi::SevenZipCompressOptions {
            num_threads: opts.num_threads as i32,
            dict_size: opts.dict_size,
            solid: if opts.solid { 1 } else { 0 },
            password: password_c.as_ref().map_or(ptr::null(), |p| p.as_ptr()),
        };
        let opts_ptr = Box::new(c_opts);

        unsafe {
            let result = ffi::sevenzip_create_7z(
                archive_path_c.as_ptr(),
                input_ptrs.as_ptr(),
                effective_level.into(),
                Box::as_ref(&opts_ptr) as *const ffi::SevenZipCompressOptions,
                None,
                ptr::null_mut(),
            );

            if result != ffi::SevenZipErrorCode::SEVENZIP_OK {
                return Err(Error::from_code(result));
            }
        }

        Ok(())
    }

    /// Create encrypted archive with recommended settings
    /// 
    /// Encryption has virtually zero performance overhead (<1%)
    /// and provides strong AES-256 security.
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// use seven_zip::SevenZip;
    /// 
    /// let sz = SevenZip::new()?;
    /// sz.create_encrypted_archive(
    ///     "secure.7z",
    ///     &["sensitive.txt", "private.doc"],
    ///     "MyStrongPassword123!",
    ///     seven_zip::CompressionLevel::Normal,
    /// )?;
    /// # Ok::<(), seven_zip::Error>(())
    /// ```
    pub fn create_encrypted_archive(
        &self,
        archive_path: impl AsRef<Path>,
        input_paths: &[impl AsRef<Path>],
        password: &str,
        level: CompressionLevel,
    ) -> Result<()> {
        let file_path_strs: Vec<String> = input_paths
            .iter()
            .map(|p| p.as_ref().to_string_lossy().to_string())
            .collect();
        let file_paths_refs: Vec<&str> = file_path_strs.iter().map(|s| s.as_str()).collect();
        
        let opts = CompressOptions::auto_tuned(&file_paths_refs)
            .unwrap_or_default()
            .with_password(password.to_string());
        
        self.create_archive(archive_path, input_paths, level, Some(&opts))
    }

    /// Create archive with smart defaults (auto-tuned threads, incompressible detection)
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// use seven_zip::SevenZip;
    /// 
    /// let sz = SevenZip::new()?;
    /// sz.create_smart_archive(
    ///     "backup.7z",
    ///     &["file1.txt", "file2.bin"],
    ///     seven_zip::CompressionLevel::Normal,
    /// )?;
    /// # Ok::<(), seven_zip::Error>(())
    /// ```
    pub fn create_smart_archive(
        &self,
        archive_path: impl AsRef<Path>,
        input_paths: &[impl AsRef<Path>],
        level: CompressionLevel,
    ) -> Result<()> {
        let file_path_strs: Vec<String> = input_paths
            .iter()
            .map(|p| p.as_ref().to_string_lossy().to_string())
            .collect();
        let file_paths_refs: Vec<&str> = file_path_strs.iter().map(|s| s.as_str()).collect();
        
        let opts = CompressOptions::auto_tuned(&file_paths_refs).unwrap_or_default();
        self.create_archive(archive_path, input_paths, level, Some(&opts))
    }

    /// Test archive integrity
    ///
    /// Validates CRCs and decompression without writing files.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use seven_zip::SevenZip;
    ///
    /// let sz = SevenZip::new()?;
    /// sz.test_archive("archive.7z", None)?;
    /// println!("Archive is valid!");
    /// # Ok::<(), seven_zip::Error>(())
    /// ```
    pub fn test_archive(&self, archive_path: impl AsRef<Path>, password: Option<&str>) -> Result<()> {
        let archive_path_c = path_to_cstring(archive_path.as_ref())?;
        let password_c = password.map(|p| CString::new(p)).transpose()?;

        unsafe {
            let result = ffi::sevenzip_test_archive(
                archive_path_c.as_ptr(),
                password_c.as_ref().map_or(ptr::null(), |p| p.as_ptr()),
                None,
                ptr::null_mut(),
            );

            if result != ffi::SevenZipErrorCode::SEVENZIP_OK {
                return Err(Error::from_code(result));
            }
        }

        Ok(())
    }

    /// Create a 7z archive with streaming compression (supports large files and split archives)
    ///
    /// This method is optimized for large files and supports creating split/multi-volume archives.
    /// Files are processed in chunks to avoid loading entire files into RAM.
    ///
    /// # Arguments
    ///
    /// * `archive_path` - Base path for the archive (e.g., "archive.7z")
    ///                    For split archives, creates archive.7z.001, archive.7z.002, etc.
    /// * `input_paths` - Files/directories to compress
    /// * `level` - Compression level
    /// * `options` - Streaming options (split size, chunk size, etc.)
    /// * `progress` - Optional byte-level progress callback
    ///
    /// # Example
    ///
    /// ```no_run
    /// use seven_zip::{SevenZip, CompressionLevel, StreamOptions};
    ///
    /// let sz = SevenZip::new()?;
    /// let mut opts = StreamOptions::default();
    /// opts.split_size = 4_294_967_296; // 4GB segments
    /// opts.chunk_size = 67_108_864;     // 64MB chunks
    /// opts.num_threads = 8;
    ///
    /// sz.create_archive_streaming(
    ///     "large_archive.7z",
    ///     &["/path/to/large/file.img"],
    ///     CompressionLevel::Normal,
    ///     Some(&opts),
    ///     Some(Box::new(|processed, total, file_bytes, file_total, filename| {
    ///         println!("Processing {}: {}/{} bytes", filename, file_bytes, file_total);
    ///         println!("Total: {}/{} bytes", processed, total);
    ///     }))
    /// )?;
    /// # Ok::<(), seven_zip::Error>(())
    /// ```
    pub fn create_archive_streaming(
        &self,
        archive_path: impl AsRef<Path>,
        input_paths: &[impl AsRef<Path>],
        level: CompressionLevel,
        options: Option<&StreamOptions>,
        progress: Option<BytesProgressCallback>,
    ) -> Result<()> {
        let archive_path_c = path_to_cstring(archive_path.as_ref())?;
        
        // Convert input paths to C strings
        let input_paths_c: Vec<CString> = input_paths
            .iter()
            .map(|p| path_to_cstring(p.as_ref()))
            .collect::<Result<_>>()?;
        let mut input_ptrs: Vec<*const i8> = input_paths_c.iter().map(|s| s.as_ptr()).collect();
        input_ptrs.push(ptr::null()); // NULL-terminate

        // Convert options to C struct
        let (opts_ptr, _password_c, _temp_dir_c) = if let Some(opts) = options {
            let password_c = opts.password.as_ref().map(|p| CString::new(p.as_str())).transpose()?;
            let temp_dir_c = opts.temp_dir.as_ref().map(|p| CString::new(p.as_str())).transpose()?;
            let c_opts = ffi::SevenZipStreamOptions {
                num_threads: opts.num_threads as i32,
                dict_size: opts.dict_size,
                solid: if opts.solid { 1 } else { 0 },
                password: password_c.as_ref().map_or(ptr::null(), |p| p.as_ptr()),
                split_size: opts.split_size,
                chunk_size: opts.chunk_size,
                temp_dir: temp_dir_c.as_ref().map_or(ptr::null(), |p| p.as_ptr()),
                delete_temp_on_error: if opts.delete_temp_on_error { 1 } else { 0 },
            };
            (Box::new(c_opts), password_c, temp_dir_c)
        } else {
            // Initialize with defaults
            let mut c_opts = std::mem::MaybeUninit::<ffi::SevenZipStreamOptions>::uninit();
            unsafe {
                ffi::sevenzip_stream_options_init(c_opts.as_mut_ptr());
                (Box::new(c_opts.assume_init()), None, None)
            }
        };

        // Set up progress callback
        let (callback, user_data) = if let Some(cb) = progress {
            let boxed = Box::new(cb);
            let raw = Box::into_raw(boxed);
            (
                Some(bytes_progress_callback_wrapper as unsafe extern "C" fn(u64, u64, u64, u64, *const std::os::raw::c_char, *mut std::os::raw::c_void)),
                raw as *mut std::os::raw::c_void,
            )
        } else {
            (None, ptr::null_mut())
        };

        unsafe {
            let result = ffi::sevenzip_create_7z_streaming(
                archive_path_c.as_ptr(),
                input_ptrs.as_ptr(),
                level.into(),
                &*opts_ptr,
                callback,
                user_data,
            );

            // Clean up the callback if it was allocated
            if !user_data.is_null() {
                let _boxed = Box::from_raw(user_data as *mut BytesProgressCallback);
                // Drops automatically
            }

            if result != ffi::SevenZipErrorCode::SEVENZIP_OK {
                return Err(Error::from_code(result));
            }
        }

        Ok(())
    }

    /// Extract a 7z archive with streaming decompression and byte-level progress
    ///
    /// Automatically handles split/multi-volume archives. For split archives, provide
    /// the path to the first volume (e.g., "archive.7z.001").
    ///
    /// # Arguments
    ///
    /// * `archive_path` - Path to archive (for splits, use base name like "archive.7z.001")
    /// * `output_dir` - Directory to extract to
    /// * `password` - Optional password for encrypted archives
    /// * `progress` - Optional byte-level progress callback
    ///
    /// # Example
    ///
    /// ```no_run
    /// use seven_zip::SevenZip;
    ///
    /// let sz = SevenZip::new()?;
    /// sz.extract_streaming(
    ///     "archive.7z.001",  // First volume of split archive
    ///     "output",
    ///     None,
    ///     Some(Box::new(|processed, total, file_bytes, file_total, filename| {
    ///         if total > 0 {
    ///             let percent = (processed as f64 / total as f64) * 100.0;
    ///             println!("Extracting {}: {:.1}%", filename, percent);
    ///         }
    ///     }))
    /// )?;
    /// # Ok::<(), seven_zip::Error>(())
    /// ```
    pub fn extract_streaming(
        &self,
        archive_path: impl AsRef<Path>,
        output_dir: impl AsRef<Path>,
        password: Option<&str>,
        progress: Option<BytesProgressCallback>,
    ) -> Result<()> {
        let archive_path_c = path_to_cstring(archive_path.as_ref())?;
        let output_dir_c = path_to_cstring(output_dir.as_ref())?;
        let password_c = password.map(|p| CString::new(p)).transpose()?;

        // Set up progress callback
        let (callback, user_data) = if let Some(cb) = progress {
            let boxed = Box::new(cb);
            let raw = Box::into_raw(boxed);
            (
                Some(bytes_progress_callback_wrapper as unsafe extern "C" fn(u64, u64, u64, u64, *const std::os::raw::c_char, *mut std::os::raw::c_void)),
                raw as *mut std::os::raw::c_void,
            )
        } else {
            (None, ptr::null_mut())
        };

        unsafe {
            let result = ffi::sevenzip_extract_streaming(
                archive_path_c.as_ptr(),
                output_dir_c.as_ptr(),
                password_c.as_ref().map_or(ptr::null(), |p| p.as_ptr()),
                callback,
                user_data,
            );

            // Clean up the callback if it was allocated
            if !user_data.is_null() {
                let _boxed = Box::from_raw(user_data as *mut BytesProgressCallback);
                // Drops automatically
            }

            if result != ffi::SevenZipErrorCode::SEVENZIP_OK {
                return Err(Error::from_code(result));
            }
        }

        Ok(())
    }

    /// Create a 7z archive using TRUE streaming compression (RECOMMENDED for large archives)
    ///
    /// ⚠️ **IMPORTANT**: This method processes files in 64MB chunks WITHOUT loading
    /// all data into RAM first. Use this for archives larger than 8GB to avoid
    /// out-of-memory crashes.
    ///
    /// The standard `create_archive_streaming` method (when split_size == 0) still
    /// loads all file data into memory before compression, which causes OOM for
    /// large archives. This method fixes that limitation.
    ///
    /// Memory usage: ~250MB peak regardless of archive size
    ///
    /// # Arguments
    ///
    /// * `archive_path` - Output archive path
    /// * `input_paths` - Files/directories to compress
    /// * `level` - Compression level
    /// * `options` - Streaming options (chunk size, threads, etc.)
    /// * `progress` - Optional byte-level progress callback
    ///
    /// # Example
    ///
    /// ```no_run
    /// use seven_zip::{SevenZip, CompressionLevel, StreamOptions};
    ///
    /// let sz = SevenZip::new()?;
    /// let mut opts = StreamOptions::default();
    /// opts.num_threads = 8;
    /// opts.chunk_size = 64 * 1024 * 1024; // 64MB chunks
    ///
    /// // Create 88GB archive without running out of memory
    /// sz.create_archive_true_streaming(
    ///     "forensic_evidence.7z",
    ///     &["/path/to/88gb/evidence/folder"],
    ///     CompressionLevel::Normal,
    ///     Some(&opts),
    ///     Some(Box::new(|processed, total, file_bytes, file_total, filename| {
    ///         let percent = if total > 0 {
    ///             (processed as f64 / total as f64) * 100.0
    ///         } else { 0.0 };
    ///         println!("[{:.1}%] {} ({}/{} bytes)", percent, filename, file_bytes, file_total);
    ///     }))
    /// )?;
    /// # Ok::<(), seven_zip::Error>(())
    /// ```
    pub fn create_archive_true_streaming(
        &self,
        archive_path: impl AsRef<Path>,
        input_paths: &[impl AsRef<Path>],
        level: CompressionLevel,
        options: Option<&StreamOptions>,
        progress: Option<BytesProgressCallback>,
    ) -> Result<()> {
        let archive_path_c = path_to_cstring(archive_path.as_ref())?;
        
        // Convert input paths to C strings
        let input_paths_c: Vec<CString> = input_paths
            .iter()
            .map(|p| path_to_cstring(p.as_ref()))
            .collect::<Result<_>>()?;
        let mut input_ptrs: Vec<*const i8> = input_paths_c.iter().map(|s| s.as_ptr()).collect();
        input_ptrs.push(ptr::null()); // NULL-terminate

        // Convert options to C struct
        let (opts_ptr, _password_c, _temp_dir_c) = if let Some(opts) = options {
            let password_c = opts.password.as_ref().map(|p| CString::new(p.as_str())).transpose()?;
            let temp_dir_c = opts.temp_dir.as_ref().map(|p| CString::new(p.as_str())).transpose()?;
            let c_opts = ffi::SevenZipStreamOptions {
                num_threads: opts.num_threads as i32,
                dict_size: opts.dict_size,
                solid: if opts.solid { 1 } else { 0 },
                password: password_c.as_ref().map_or(ptr::null(), |p| p.as_ptr()),
                split_size: opts.split_size,
                chunk_size: opts.chunk_size,
                temp_dir: temp_dir_c.as_ref().map_or(ptr::null(), |p| p.as_ptr()),
                delete_temp_on_error: if opts.delete_temp_on_error { 1 } else { 0 },
            };
            (Box::new(c_opts), password_c, temp_dir_c)
        } else {
            // Initialize with defaults
            let mut c_opts = std::mem::MaybeUninit::<ffi::SevenZipStreamOptions>::uninit();
            unsafe {
                ffi::sevenzip_stream_options_init(c_opts.as_mut_ptr());
                (Box::new(c_opts.assume_init()), None, None)
            }
        };

        // Set up progress callback
        let (callback, user_data) = if let Some(cb) = progress {
            let boxed = Box::new(cb);
            let raw = Box::into_raw(boxed);
            (
                Some(bytes_progress_callback_wrapper as unsafe extern "C" fn(u64, u64, u64, u64, *const std::os::raw::c_char, *mut std::os::raw::c_void)),
                raw as *mut std::os::raw::c_void,
            )
        } else {
            (None, ptr::null_mut())
        };

        unsafe {
            let result = ffi::sevenzip_create_7z_true_streaming(
                archive_path_c.as_ptr(),
                input_ptrs.as_ptr(),
                level.into(),
                &*opts_ptr,
                callback,
                user_data,
            );

            // Clean up the callback if it was allocated
            if !user_data.is_null() {
                let _boxed = Box::from_raw(user_data as *mut BytesProgressCallback);
                // Drops automatically
            }

            if result != ffi::SevenZipErrorCode::SEVENZIP_OK {
                return Err(Error::from_code(result));
            }
        }

        Ok(())
    }

    /// Compress a single file to LZMA2 format
    ///
    /// # Example
    ///
    /// ```no_run
    /// use seven_zip::{SevenZip, CompressionLevel};
    ///
    /// let sz = SevenZip::new()?;
    /// sz.compress_file("input.txt", "output.lzma2", CompressionLevel::Normal)?;
    /// # Ok::<(), seven_zip::Error>(())
    /// ```
    pub fn compress_file(
        &self,
        input_path: impl AsRef<Path>,
        output_path: impl AsRef<Path>,
        level: CompressionLevel,
    ) -> Result<()> {
        let input_path_c = path_to_cstring(input_path.as_ref())?;
        let output_path_c = path_to_cstring(output_path.as_ref())?;

        unsafe {
            let result = ffi::sevenzip_compress_file(
                input_path_c.as_ptr(),
                output_path_c.as_ptr(),
                level.into(),
                None,
                ptr::null_mut(),
            );

            if result != ffi::SevenZipErrorCode::SEVENZIP_OK {
                return Err(Error::from_code(result));
            }
        }

        Ok(())
    }

    /// Decompress a single LZMA2 file
    ///
    /// # Example
    ///
    /// ```no_run
    /// use seven_zip::SevenZip;
    ///
    /// let sz = SevenZip::new()?;
    /// sz.decompress_file("input.lzma2", "output.txt")?;
    /// # Ok::<(), seven_zip::Error>(())
    /// ```
    pub fn decompress_file(
        &self,
        input_path: impl AsRef<Path>,
        output_path: impl AsRef<Path>,
    ) -> Result<()> {
        let input_path_c = path_to_cstring(input_path.as_ref())?;
        let output_path_c = path_to_cstring(output_path.as_ref())?;

        unsafe {
            let result = ffi::sevenzip_decompress_file(
                input_path_c.as_ptr(),
                output_path_c.as_ptr(),
                None,
                ptr::null_mut(),
            );

            if result != ffi::SevenZipErrorCode::SEVENZIP_OK {
                return Err(Error::from_code(result));
            }
        }

        Ok(())
    }
}

impl Drop for SevenZip {
    fn drop(&mut self) {
        unsafe {
            ffi::sevenzip_cleanup();
        }
    }
}

// Helper functions

fn path_to_cstring(path: &Path) -> Result<CString> {
    let path_str = path.to_str()
        .ok_or_else(|| Error::InvalidParameter("Invalid path encoding".to_string()))?;
    CString::new(path_str)
        .map_err(|_| Error::InvalidParameter("Path contains null byte".to_string()))
}

unsafe extern "C" fn progress_callback_wrapper(
    completed: u64,
    total: u64,
    user_data: *mut std::os::raw::c_void,
) {
    if !user_data.is_null() {
        // SAFETY: user_data is guaranteed to be a valid ProgressCallback pointer
        // The pointer remains valid for the duration of the C function call
        unsafe {
            let callback = &mut *(user_data as *mut ProgressCallback);
            callback(completed, total);
        }
    }
}

unsafe extern "C" fn bytes_progress_callback_wrapper(
    bytes_processed: u64,
    bytes_total: u64,
    current_file_bytes: u64,
    current_file_total: u64,
    current_file_name: *const std::os::raw::c_char,
    user_data: *mut std::os::raw::c_void,
) {
    if !user_data.is_null() {
        unsafe {
            // SAFETY: user_data is guaranteed to be a valid BytesProgressCallback pointer
            let callback = &mut *(user_data as *mut BytesProgressCallback);
            
            // Convert C string to Rust &str
            let file_name = if !current_file_name.is_null() {
                CStr::from_ptr(current_file_name)
                    .to_str()
                    .unwrap_or("<invalid utf-8>")
            } else {
                ""
            };
            
            callback(bytes_processed, bytes_total, current_file_bytes, current_file_total, file_name);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_level_conversion() {
        assert_eq!(
            ffi::SevenZipCompressionLevel::from(CompressionLevel::Normal),
            ffi::SevenZipCompressionLevel::SEVENZIP_LEVEL_NORMAL
        );
    }

    #[test]
    fn test_archive_entry_compression_ratio() {
        let entry = ArchiveEntry {
            name: "test.txt".to_string(),
            size: 1000,
            packed_size: 300,
            modified_time: 0,
            attributes: 0,
            is_directory: false,
        };
        assert_eq!(entry.compression_ratio(), 70.0);
    }

    #[test]
    fn test_default_options() {
        let opts = CompressOptions::default();
        assert_eq!(opts.num_threads, 0);
        assert!(opts.solid);
        assert!(opts.password.is_none());
    }
}
