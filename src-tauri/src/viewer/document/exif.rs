// =============================================================================
// CORE-FFX - Forensic File Explorer
// EXIF Metadata Extractor - Photo forensics
// =============================================================================

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use exif::{In, Reader, Tag};

use super::error::{DocumentError, DocumentResult};

/// GPS coordinates extracted from photo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpsCoordinates {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f64>,
    pub latitude_ref: String,  // N or S
    pub longitude_ref: String, // E or W
}

/// EXIF metadata extracted from photo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExifMetadata {
    pub path: String,
    // Camera info
    pub make: Option<String>,
    pub model: Option<String>,
    pub software: Option<String>,
    pub lens_model: Option<String>,
    // Capture settings
    pub exposure_time: Option<String>,
    pub f_number: Option<String>,
    pub iso: Option<u32>,
    pub focal_length: Option<String>,
    pub flash: Option<String>,
    // Timestamps (forensically important!)
    pub date_time_original: Option<String>,
    pub date_time_digitized: Option<String>,
    pub date_time: Option<String>,
    pub gps_timestamp: Option<String>,
    // GPS
    pub gps: Option<GpsCoordinates>,
    // Image info
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub orientation: Option<u16>,
    pub color_space: Option<String>,
    // Forensic indicators
    pub image_unique_id: Option<String>,
    pub owner_name: Option<String>,
    pub serial_number: Option<String>,
    // All raw tags for complete analysis
    pub raw_tags: Vec<(String, String)>,
}

/// Extract EXIF metadata from an image file
pub fn extract_exif(path: impl AsRef<Path>) -> DocumentResult<ExifMetadata> {
    let path = path.as_ref();
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    
    let exif = Reader::new()
        .read_from_container(&mut reader)
        .map_err(|e| DocumentError::Parse(format!("Failed to read EXIF: {}", e)))?;
    
    // Helper to get string value
    let get_str = |tag: Tag| -> Option<String> {
        exif.get_field(tag, In::PRIMARY)
            .map(|f| f.display_value().with_unit(&exif).to_string())
    };
    
    // Helper to get u32 value
    let get_u32 = |tag: Tag| -> Option<u32> {
        exif.get_field(tag, In::PRIMARY)
            .and_then(|f| f.value.get_uint(0))
    };
    
    // Helper to get u16 value
    let get_u16 = |tag: Tag| -> Option<u16> {
        exif.get_field(tag, In::PRIMARY)
            .and_then(|f| f.value.get_uint(0).map(|v| v as u16))
    };
    
    // Extract GPS if available
    let gps = extract_gps(&exif);
    
    // Collect all raw tags
    let raw_tags: Vec<(String, String)> = exif.fields()
        .map(|f| {
            (
                f.tag.to_string(),
                f.display_value().with_unit(&exif).to_string()
            )
        })
        .collect();
    
    Ok(ExifMetadata {
        path: path.to_string_lossy().to_string(),
        // Camera info
        make: get_str(Tag::Make),
        model: get_str(Tag::Model),
        software: get_str(Tag::Software),
        lens_model: get_str(Tag::LensModel),
        // Capture settings
        exposure_time: get_str(Tag::ExposureTime),
        f_number: get_str(Tag::FNumber),
        iso: get_u32(Tag::PhotographicSensitivity),
        focal_length: get_str(Tag::FocalLength),
        flash: get_str(Tag::Flash),
        // Timestamps
        date_time_original: get_str(Tag::DateTimeOriginal),
        date_time_digitized: get_str(Tag::DateTimeDigitized),
        date_time: get_str(Tag::DateTime),
        gps_timestamp: get_str(Tag::GPSTimeStamp),
        // GPS
        gps,
        // Image info
        width: get_u32(Tag::PixelXDimension).or(get_u32(Tag::ImageWidth)),
        height: get_u32(Tag::PixelYDimension).or(get_u32(Tag::ImageLength)),
        orientation: get_u16(Tag::Orientation),
        color_space: get_str(Tag::ColorSpace),
        // Forensic indicators
        image_unique_id: get_str(Tag::ImageUniqueID),
        owner_name: None, // Not standard EXIF
        serial_number: get_str(Tag::BodySerialNumber),
        // All raw tags
        raw_tags,
    })
}

fn extract_gps(exif: &exif::Exif) -> Option<GpsCoordinates> {
    let lat = exif.get_field(Tag::GPSLatitude, In::PRIMARY)?;
    let lon = exif.get_field(Tag::GPSLongitude, In::PRIMARY)?;
    let lat_ref = exif.get_field(Tag::GPSLatitudeRef, In::PRIMARY)?;
    let lon_ref = exif.get_field(Tag::GPSLongitudeRef, In::PRIMARY)?;
    
    // Parse latitude
    let lat_val = parse_gps_coord(&lat.value)?;
    let lon_val = parse_gps_coord(&lon.value)?;
    
    let lat_ref_str = lat_ref.display_value().to_string();
    let lon_ref_str = lon_ref.display_value().to_string();
    
    let latitude = if lat_ref_str.contains('S') { -lat_val } else { lat_val };
    let longitude = if lon_ref_str.contains('W') { -lon_val } else { lon_val };
    
    // Try to get altitude
    let altitude = exif.get_field(Tag::GPSAltitude, In::PRIMARY)
        .and_then(|f| {
            if let exif::Value::Rational(ref v) = f.value {
                v.first().map(|r| r.to_f64())
            } else {
                None
            }
        });
    
    Some(GpsCoordinates {
        latitude,
        longitude,
        altitude,
        latitude_ref: lat_ref_str,
        longitude_ref: lon_ref_str,
    })
}

fn parse_gps_coord(value: &exif::Value) -> Option<f64> {
    if let exif::Value::Rational(ref rationals) = value {
        if rationals.len() >= 3 {
            let degrees = rationals[0].to_f64();
            let minutes = rationals[1].to_f64();
            let seconds = rationals[2].to_f64();
            return Some(degrees + minutes / 60.0 + seconds / 3600.0);
        }
    }
    None
}

/// Check if file has EXIF data without full parsing
pub fn has_exif(path: impl AsRef<Path>) -> bool {
    let path = path.as_ref();
    if let Ok(file) = File::open(path) {
        let mut reader = BufReader::new(file);
        return Reader::new().read_from_container(&mut reader).is_ok();
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_gps_coordinates_struct() {
        let gps = GpsCoordinates {
            latitude: 37.7749,
            longitude: -122.4194,
            altitude: Some(10.0),
            latitude_ref: "N".to_string(),
            longitude_ref: "W".to_string(),
        };
        assert_eq!(gps.latitude, 37.7749);
    }
}
