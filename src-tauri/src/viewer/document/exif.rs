// =============================================================================
// CORE-FFX - Forensic File Explorer
// EXIF Metadata Extractor - Photo forensics
// =============================================================================

use exif::{In, Reader, Tag};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Cursor, Read, Seek};
use std::path::Path;

use super::error::{DocumentError, DocumentResult};

pub(crate) const MAX_EXIF_SOURCE_BYTES: u64 = 100 * 1024 * 1024;
const MAX_EXIF_RAW_TAGS: usize = 1024;
const MAX_EXIF_DISPLAY_VALUE_CHARS: usize = 2048;
const MAX_EXIF_IMAGE_PIXELS: u64 = 100_000_000;

pub(crate) fn ensure_exif_size_allowed(size: u64) -> DocumentResult<()> {
    if size > MAX_EXIF_SOURCE_BYTES {
        return Err(DocumentError::Parse(format!(
            "Image file too large for EXIF extraction ({:.1} MiB, max {} MiB)",
            size as f64 / (1024.0 * 1024.0),
            MAX_EXIF_SOURCE_BYTES / (1024 * 1024)
        )));
    }
    Ok(())
}

fn read_exif_source_with_limit<R: Read>(reader: R, max_bytes: u64) -> DocumentResult<Vec<u8>> {
    let mut data = Vec::new();
    reader
        .take(max_bytes.saturating_add(1))
        .read_to_end(&mut data)?;
    if data.len() as u64 > max_bytes {
        return Err(DocumentError::Parse(format!(
            "Image file too large for EXIF extraction (actual read exceeded max {} MiB)",
            max_bytes / (1024 * 1024)
        )));
    }
    Ok(data)
}

/// GPS coordinates extracted from photo
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GpsCoordinates {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f64>,
    pub latitude_ref: String,  // N or S
    pub longitude_ref: String, // E or W
}

impl GpsCoordinates {
    /// Create new GPS coordinates
    #[inline]
    pub fn new(latitude: f64, longitude: f64) -> Self {
        Self {
            latitude,
            longitude,
            latitude_ref: if latitude >= 0.0 {
                "N".to_string()
            } else {
                "S".to_string()
            },
            longitude_ref: if longitude >= 0.0 {
                "E".to_string()
            } else {
                "W".to_string()
            },
            altitude: None,
        }
    }

    /// Set altitude
    #[inline]
    pub fn with_altitude(mut self, altitude: f64) -> Self {
        self.altitude = Some(altitude);
        self
    }

    /// Set reference strings
    #[inline]
    pub fn with_refs(mut self, lat_ref: impl Into<String>, lon_ref: impl Into<String>) -> Self {
        self.latitude_ref = lat_ref.into();
        self.longitude_ref = lon_ref.into();
        self
    }

    /// Format as decimal degrees string
    #[inline]
    pub fn to_decimal_string(&self) -> String {
        format!("{:.6}, {:.6}", self.latitude, self.longitude)
    }

    /// Format as DMS (degrees, minutes, seconds) string
    pub fn to_dms_string(&self) -> String {
        let lat_d = self.latitude.abs();
        let lat_deg = lat_d.floor() as i32;
        let lat_min = ((lat_d - lat_deg as f64) * 60.0).floor() as i32;
        let lat_sec = (lat_d - lat_deg as f64 - lat_min as f64 / 60.0) * 3600.0;

        let lon_d = self.longitude.abs();
        let lon_deg = lon_d.floor() as i32;
        let lon_min = ((lon_d - lon_deg as f64) * 60.0).floor() as i32;
        let lon_sec = (lon_d - lon_deg as f64 - lon_min as f64 / 60.0) * 3600.0;

        format!(
            "{}°{}'{:.2}\"{}  {}°{}'{:.2}\"{}",
            lat_deg,
            lat_min,
            lat_sec,
            &self.latitude_ref,
            lon_deg,
            lon_min,
            lon_sec,
            &self.longitude_ref
        )
    }
}

/// EXIF metadata extracted from photo
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

impl ExifMetadata {
    /// Create new ExifMetadata for a path
    #[inline]
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            ..Default::default()
        }
    }

    /// Set camera make and model
    #[inline]
    pub fn with_camera(mut self, make: impl Into<String>, model: impl Into<String>) -> Self {
        self.make = Some(make.into());
        self.model = Some(model.into());
        self
    }

    /// Set exposure settings
    #[inline]
    pub fn with_exposure(
        mut self,
        exposure_time: impl Into<String>,
        f_number: impl Into<String>,
        iso: u32,
    ) -> Self {
        self.exposure_time = Some(exposure_time.into());
        self.f_number = Some(f_number.into());
        self.iso = Some(iso);
        self
    }

    /// Set original date/time (primary forensic timestamp)
    #[inline]
    pub fn with_date_time_original(mut self, dt: impl Into<String>) -> Self {
        self.date_time_original = Some(dt.into());
        self
    }

    /// Set GPS coordinates
    #[inline]
    pub fn with_gps(mut self, gps: GpsCoordinates) -> Self {
        self.gps = Some(gps);
        self
    }

    /// Set image dimensions
    #[inline]
    pub fn with_dimensions(mut self, width: u32, height: u32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    /// Add raw tags
    #[inline]
    pub fn with_raw_tags(mut self, tags: Vec<(String, String)>) -> Self {
        self.raw_tags = tags;
        self
    }

    /// Get camera display string (make + model)
    #[inline]
    pub fn camera_display(&self) -> Option<String> {
        match (&self.make, &self.model) {
            (Some(m), Some(md)) => Some(format!("{} {}", m, md)),
            (Some(m), None) => Some(m.clone()),
            (None, Some(md)) => Some(md.clone()),
            (None, None) => None,
        }
    }

    /// Get dimensions display string
    #[inline]
    pub fn dimensions_display(&self) -> Option<String> {
        match (self.width, self.height) {
            (Some(w), Some(h)) => Some(format!("{} × {}", w, h)),
            _ => None,
        }
    }

    /// Get primary timestamp (forensically most important)
    #[inline]
    pub fn primary_timestamp(&self) -> Option<&String> {
        self.date_time_original
            .as_ref()
            .or(self.date_time_digitized.as_ref())
            .or(self.date_time.as_ref())
    }

    /// Check if GPS data is available
    #[inline]
    pub fn has_gps(&self) -> bool {
        self.gps.is_some()
    }

    /// Check if any forensic indicators are present
    #[inline]
    pub fn has_forensic_indicators(&self) -> bool {
        self.image_unique_id.is_some() || self.serial_number.is_some() || self.owner_name.is_some()
    }
}

/// Extract EXIF metadata from an image file
pub fn extract_exif(path: impl AsRef<Path>) -> DocumentResult<ExifMetadata> {
    let path = path.as_ref();
    ensure_exif_size_allowed(std::fs::metadata(path)?.len())?;
    let file = File::open(path)?;
    extract_exif_from_reader(path.to_string_lossy(), file)
}

/// Extract EXIF metadata from any seekable byte stream.
pub fn extract_exif_from_reader(
    source_id: impl Into<String>,
    reader: impl Read + Seek,
) -> DocumentResult<ExifMetadata> {
    let source_id = source_id.into();
    let data = read_exif_source_with_limit(reader, MAX_EXIF_SOURCE_BYTES)?;
    let mut reader = Cursor::new(data);

    let exif = Reader::new()
        .read_from_container(&mut reader)
        .map_err(|e| DocumentError::Parse(format!("Failed to read EXIF: {}", e)))?;

    // Helper to get string value
    let get_str = |tag: Tag| -> Option<String> {
        exif.get_field(tag, In::PRIMARY)
            .map(|f| clean_display_value(f.display_value().with_unit(&exif).to_string()))
    };

    // Helper to get u32 value
    let get_u32 = |tag: Tag| -> Option<u32> {
        exif.get_field(tag, In::PRIMARY)
            .and_then(|f| f.value.get_uint(0))
    };

    // Helper to get u16 value
    let get_u16 = |tag: Tag| -> Option<u16> {
        exif.get_field(tag, In::PRIMARY)
            .and_then(|f| f.value.get_uint(0).and_then(exif_uint_to_u16))
    };

    // Extract GPS if available
    let gps = extract_gps(&exif);

    // Collect all raw tags
    let raw_tags: Vec<(String, String)> = exif
        .fields()
        .take(MAX_EXIF_RAW_TAGS)
        .map(|f| {
            (
                f.tag.to_string(),
                clean_display_value(f.display_value().with_unit(&exif).to_string()),
            )
        })
        .collect();
    let (width, height) = validated_exif_dimensions(
        get_u32(Tag::PixelXDimension).or(get_u32(Tag::ImageWidth)),
        get_u32(Tag::PixelYDimension).or(get_u32(Tag::ImageLength)),
    );

    Ok(ExifMetadata {
        path: source_id,
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
        width,
        height,
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

fn clean_display_value(value: String) -> String {
    let value = value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(&value)
        .to_string();
    truncate_display_value(value, MAX_EXIF_DISPLAY_VALUE_CHARS)
}

fn truncate_display_value(value: String, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value;
    }

    let mut truncated: String = value.chars().take(max_chars).collect();
    truncated.push_str("...");
    truncated
}

fn exif_uint_to_u16(value: u32) -> Option<u16> {
    u16::try_from(value).ok()
}

fn validated_exif_dimensions(
    width: Option<u32>,
    height: Option<u32>,
) -> (Option<u32>, Option<u32>) {
    let (Some(width), Some(height)) = (width, height) else {
        return (None, None);
    };
    if width == 0 || height == 0 {
        return (None, None);
    }
    let Some(pixels) = u64::from(width).checked_mul(u64::from(height)) else {
        return (None, None);
    };
    if pixels > MAX_EXIF_IMAGE_PIXELS {
        return (None, None);
    }
    (Some(width), Some(height))
}

fn extract_gps(exif: &exif::Exif) -> Option<GpsCoordinates> {
    let lat = exif.get_field(Tag::GPSLatitude, In::PRIMARY)?;
    let lon = exif.get_field(Tag::GPSLongitude, In::PRIMARY)?;
    let lat_ref = exif.get_field(Tag::GPSLatitudeRef, In::PRIMARY)?;
    let lon_ref = exif.get_field(Tag::GPSLongitudeRef, In::PRIMARY)?;

    // Parse latitude
    let lat_val = parse_gps_coord(&lat.value)?;
    let lon_val = parse_gps_coord(&lon.value)?;

    let lat_ref_str = gps_ref_from_display(lat_ref.display_value().to_string(), "N", "S")?;
    let lon_ref_str = gps_ref_from_display(lon_ref.display_value().to_string(), "E", "W")?;

    let latitude = match lat_ref_str.as_str() {
        "S" => -lat_val,
        "N" => lat_val,
        _ => return None,
    };
    let longitude = match lon_ref_str.as_str() {
        "W" => -lon_val,
        "E" => lon_val,
        _ => return None,
    };
    if !is_valid_gps_coordinate(latitude, longitude) {
        return None;
    }

    // Try to get altitude
    let altitude = exif.get_field(Tag::GPSAltitude, In::PRIMARY).and_then(|f| {
        if let exif::Value::Rational(ref v) = f.value {
            v.first().and_then(rational_to_f64)
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

fn gps_ref_from_display(value: String, positive: &str, negative: &str) -> Option<String> {
    let value = clean_display_value(value);
    let value = value.trim();
    if value == positive || value == negative {
        Some(value.to_string())
    } else {
        None
    }
}

fn rational_to_f64(value: &exif::Rational) -> Option<f64> {
    if value.denom == 0 {
        return None;
    }
    let value = value.to_f64();
    value.is_finite().then_some(value)
}

fn parse_gps_coord(value: &exif::Value) -> Option<f64> {
    if let exif::Value::Rational(ref rationals) = value {
        if rationals.len() >= 3 {
            let degrees = rational_to_f64(&rationals[0])?;
            let minutes = rational_to_f64(&rationals[1])?;
            let seconds = rational_to_f64(&rationals[2])?;
            let coordinate = degrees + minutes / 60.0 + seconds / 3600.0;
            return coordinate.is_finite().then_some(coordinate);
        }
    }
    None
}

fn is_valid_gps_coordinate(latitude: f64, longitude: f64) -> bool {
    latitude.is_finite()
        && longitude.is_finite()
        && (-90.0..=90.0).contains(&latitude)
        && (-180.0..=180.0).contains(&longitude)
}

/// Check if file has EXIF data without full parsing
pub fn has_exif(path: impl AsRef<Path>) -> bool {
    let path = path.as_ref();
    if std::fs::metadata(path)
        .map(|meta| ensure_exif_size_allowed(meta.len()).is_err())
        .unwrap_or(true)
    {
        return false;
    }
    if let Ok(file) = File::open(path) {
        let Ok(data) = read_exif_source_with_limit(file, MAX_EXIF_SOURCE_BYTES) else {
            return false;
        };
        let mut reader = Cursor::new(data);
        return Reader::new().read_from_container(&mut reader).is_ok();
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Write};

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

    #[test]
    fn extract_exif_from_reader_reads_tiff_metadata() {
        let mut tiff = Vec::new();
        tiff.extend_from_slice(b"II");
        tiff.extend_from_slice(&42u16.to_le_bytes());
        tiff.extend_from_slice(&8u32.to_le_bytes());
        tiff.extend_from_slice(&1u16.to_le_bytes());
        tiff.extend_from_slice(&0x010fu16.to_le_bytes()); // Make
        tiff.extend_from_slice(&2u16.to_le_bytes()); // ASCII
        tiff.extend_from_slice(&5u32.to_le_bytes()); // "CORE\0"
        tiff.extend_from_slice(&26u32.to_le_bytes());
        tiff.extend_from_slice(&0u32.to_le_bytes());
        tiff.extend_from_slice(b"CORE\0");

        let metadata = extract_exif_from_reader("memory.tiff", Cursor::new(tiff)).unwrap();

        assert_eq!(metadata.path, "memory.tiff");
        assert_eq!(metadata.make.as_deref(), Some("CORE"));
        assert!(metadata
            .raw_tags
            .iter()
            .any(|(tag, value)| tag == "Make" && value == "CORE"));
    }

    #[test]
    fn read_exif_source_with_limit_reads_within_limit() {
        let data = read_exif_source_with_limit(Cursor::new(b"abc"), 3).unwrap();

        assert_eq!(data, b"abc");
    }

    #[test]
    fn read_exif_source_with_limit_rejects_actual_read_limit() {
        let err = read_exif_source_with_limit(Cursor::new(b"abcd"), 3).unwrap_err();

        assert!(err.to_string().contains("too large for EXIF extraction"));
    }

    #[test]
    fn extract_exif_rejects_sparse_oversized_file_before_parsing() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"II*\0").unwrap();
        tmp.as_file_mut()
            .set_len(MAX_EXIF_SOURCE_BYTES + 1)
            .unwrap();

        let err = extract_exif(tmp.path()).unwrap_err();

        assert!(err.to_string().contains("too large for EXIF extraction"));
        assert!(!has_exif(tmp.path()));
    }

    #[test]
    fn parse_gps_coord_rejects_zero_denominator_rational() {
        let value = exif::Value::Rational(vec![
            exif::Rational { num: 37, denom: 1 },
            exif::Rational { num: 46, denom: 0 },
            exif::Rational {
                num: 2964,
                denom: 100,
            },
        ]);

        assert!(parse_gps_coord(&value).is_none());
    }

    #[test]
    fn gps_coordinate_range_validation_rejects_huge_rational_value() {
        let value = exif::Value::Rational(vec![
            exif::Rational {
                num: u32::MAX,
                denom: 1,
            },
            exif::Rational { num: 0, denom: 1 },
            exif::Rational { num: 0, denom: 1 },
        ]);

        assert!(parse_gps_coord(&value).is_some());
        assert!(!is_valid_gps_coordinate(
            parse_gps_coord(&value).unwrap(),
            0.0
        ));
    }

    #[test]
    fn gps_coordinate_range_validation_rejects_out_of_range_values() {
        assert!(is_valid_gps_coordinate(37.7749, -122.4194));
        assert!(!is_valid_gps_coordinate(90.1, 0.0));
        assert!(!is_valid_gps_coordinate(0.0, -180.1));
        assert!(!is_valid_gps_coordinate(f64::NAN, 0.0));
    }

    #[test]
    fn gps_ref_from_display_accepts_only_exact_refs() {
        assert_eq!(
            gps_ref_from_display("\"S\"".to_string(), "N", "S").as_deref(),
            Some("S")
        );
        assert_eq!(
            gps_ref_from_display("W".to_string(), "E", "W").as_deref(),
            Some("W")
        );
        assert!(gps_ref_from_display("South".to_string(), "N", "S").is_none());
        assert!(gps_ref_from_display("SW".to_string(), "E", "W").is_none());
        assert!(gps_ref_from_display(String::new(), "N", "S").is_none());
    }

    #[test]
    fn clean_display_value_strips_quotes_and_truncates() {
        let value = format!("\"{}\"", "a".repeat(MAX_EXIF_DISPLAY_VALUE_CHARS + 1));

        let cleaned = clean_display_value(value);

        assert_eq!(cleaned.chars().count(), MAX_EXIF_DISPLAY_VALUE_CHARS + 3);
        assert!(cleaned.ends_with("..."));
        assert!(!cleaned.starts_with('"'));
    }

    #[test]
    fn truncate_display_value_is_unicode_safe() {
        let truncated = truncate_display_value("åß∂ƒ".to_string(), 3);

        assert_eq!(truncated, "åß∂...");
    }

    #[test]
    fn exif_uint_to_u16_rejects_overflow() {
        assert_eq!(exif_uint_to_u16(u16::MAX as u32), Some(u16::MAX));
        assert_eq!(exif_uint_to_u16(u16::MAX as u32 + 1), None);
    }

    #[test]
    fn validated_exif_dimensions_reject_zero_or_excessive_values() {
        assert_eq!(
            validated_exif_dimensions(Some(640), Some(480)),
            (Some(640), Some(480))
        );
        assert_eq!(validated_exif_dimensions(Some(0), Some(480)), (None, None));
        assert_eq!(
            validated_exif_dimensions(Some(100_001), Some(1_000)),
            (None, None)
        );
        assert_eq!(validated_exif_dimensions(Some(640), None), (None, None));
    }
}
