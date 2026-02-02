use seven_zip::SevenZip;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sz = SevenZip::new()?;
    
    println!("Testing Archive Format Support\n");
    println!("═══════════════════════════════════════════\n");
    
    // Test with a ZIP file
    println!("Testing ZIP file support...");
    let zip_path = std::path::Path::new("../test.zip");
    if zip_path.exists() {
        match sz.list("../test.zip", None) {
            Ok(entries) => {
                println!("  ✓ ZIP format: SUPPORTED");
                println!("    Files found: {}", entries.len());
                for entry in entries {
                    println!("      - {}", entry.name);
                }
            }
            Err(e) => println!("  ✗ ZIP format: NOT SUPPORTED ({})", e),
        }
    } else {
        println!("  ⚠ ZIP test file not found at {:?}", zip_path);
    }
    println!();
    
    // Test with a 7z file
    println!("Testing 7z file support...");
    let sz_path = std::path::Path::new("demo_normal.7z");
    if sz_path.exists() {
        match sz.list("demo_normal.7z", None) {
            Ok(entries) => {
                println!("  ✓ 7z format: SUPPORTED");
                println!("    Files found: {}", entries.len());
            }
            Err(e) => println!("  ✗ 7z format: NOT SUPPORTED ({})", e),
        }
    } else {
        println!("  ⚠ 7z test file not found at {:?}", sz_path);
    }
    println!();    println!("═══════════════════════════════════════════");
    println!("\nNote: The 7z SDK primarily supports 7z format.");
    println!("Full format support (ZIP, TAR, RAR, etc.) requires");
    println!("additional codec implementations not included in LZMA SDK.");
    
    Ok(())
}
