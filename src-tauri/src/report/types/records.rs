// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Record types: hash records, timeline events, custody records, and COC form types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};

use super::{deserialize_datetime_flexible, deserialize_datetime_opt};

/// Flexible hash algorithm deserializer that accepts enum or string
fn deserialize_hash_algorithm<'de, D>(deserializer: D) -> Result<HashAlgorithm, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum AlgoOrString {
        Algo(HashAlgorithm),
        String(String),
    }

    match AlgoOrString::deserialize(deserializer)? {
        AlgoOrString::Algo(a) => Ok(a),
        AlgoOrString::String(s) => match s.to_uppercase().as_str() {
            "MD5" => Ok(HashAlgorithm::MD5),
            "SHA1" | "SHA-1" => Ok(HashAlgorithm::SHA1),
            "SHA256" | "SHA-256" => Ok(HashAlgorithm::SHA256),
            "SHA512" | "SHA-512" => Ok(HashAlgorithm::SHA512),
            "BLAKE2B" | "BLAKE2" => Ok(HashAlgorithm::Blake2b),
            "BLAKE3" => Ok(HashAlgorithm::Blake3),
            "XXH3" => Ok(HashAlgorithm::XXH3),
            "XXH64" => Ok(HashAlgorithm::XXH64),
            _ => Err(D::Error::custom(format!("Unknown hash algorithm: {}", s))),
        },
    }
}

/// Supported hash algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HashAlgorithm {
    MD5,
    SHA1,
    SHA256,
    SHA512,
    Blake2b,
    Blake3,
    XXH3,
    XXH64,
}

impl HashAlgorithm {
    pub fn as_str(&self) -> &'static str {
        match self {
            HashAlgorithm::MD5 => "MD5",
            HashAlgorithm::SHA1 => "SHA-1",
            HashAlgorithm::SHA256 => "SHA-256",
            HashAlgorithm::SHA512 => "SHA-512",
            HashAlgorithm::Blake2b => "BLAKE2b",
            HashAlgorithm::Blake3 => "BLAKE3",
            HashAlgorithm::XXH3 => "XXH3",
            HashAlgorithm::XXH64 => "XXH64",
        }
    }
}

/// Hash record
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HashRecord {
    /// Item being hashed (also accepts item_reference from frontend)
    #[serde(alias = "item_reference")]
    pub item: String,
    /// Hash algorithm
    #[serde(deserialize_with = "deserialize_hash_algorithm")]
    pub algorithm: HashAlgorithm,
    /// Hash value (hex string)
    pub value: String,
    /// When the hash was computed (also accepts timestamp from frontend)
    #[serde(default, alias = "timestamp", deserialize_with = "deserialize_datetime_opt")]
    pub computed_at: Option<DateTime<Utc>>,
    /// Verification status
    #[serde(default)]
    pub verified: Option<bool>,
}

impl Default for HashRecord {
    fn default() -> Self {
        Self {
            item: String::new(),
            algorithm: HashAlgorithm::SHA256,
            value: String::new(),
            computed_at: None,
            verified: None,
        }
    }
}

/// Timeline event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TimelineEvent {
    /// Event timestamp
    #[serde(deserialize_with = "deserialize_datetime_flexible")]
    pub timestamp: DateTime<Utc>,
    /// Timestamp type (created, modified, accessed, etc.)
    pub timestamp_type: String,
    /// Event description
    pub description: String,
    /// Source of the event
    pub source: String,
    /// Related file or artifact
    #[serde(default)]
    pub artifact: Option<String>,
    /// Related evidence ID
    #[serde(default)]
    pub evidence_id: Option<String>,
    /// Significance
    #[serde(default)]
    pub significance: Option<String>,
}

impl Default for TimelineEvent {
    fn default() -> Self {
        Self {
            timestamp: Utc::now(),
            timestamp_type: String::new(),
            description: String::new(),
            source: String::new(),
            artifact: None,
            evidence_id: None,
            significance: None,
        }
    }
}

/// Chain of custody record
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct CustodyRecord {
    /// Evidence item ID
    #[serde(default)]
    pub evidence_id: String,
    /// Date/time of transfer
    pub timestamp: DateTime<Utc>,
    /// Person releasing custody
    pub released_by: String,
    /// Person receiving custody
    pub received_by: String,
    /// Purpose of transfer
    pub purpose: Option<String>,
    /// Location
    pub location: Option<String>,
    /// Notes
    pub notes: Option<String>,
}

impl CustodyRecord {
    /// Create a new custody record
    #[inline]
    pub fn new(
        evidence_id: impl Into<String>,
        released_by: impl Into<String>,
        received_by: impl Into<String>,
    ) -> Self {
        Self {
            evidence_id: evidence_id.into(),
            timestamp: Utc::now(),
            released_by: released_by.into(),
            received_by: received_by.into(),
            ..Default::default()
        }
    }

    /// Set timestamp
    #[inline]
    pub fn with_timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.timestamp = timestamp;
        self
    }

    /// Set purpose
    #[inline]
    pub fn with_purpose(mut self, purpose: impl Into<String>) -> Self {
        self.purpose = Some(purpose.into());
        self
    }

    /// Set location
    #[inline]
    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }

    /// Set notes
    #[inline]
    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }
}

// =============================================================================
// CHAIN OF CUSTODY (COC) FORM 7-01 TYPES
// =============================================================================

/// EPA CID OCEFT Form 7-01 — Chain of Custody item
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CocItem {
    /// Internal UI identifier
    pub id: String,
    /// Unique COC item number (e.g., "24-06627-011")
    pub coc_number: String,
    /// Evidence item ID reference
    pub evidence_id: String,
    /// Case number
    pub case_number: String,
    /// Item description
    pub description: String,
    /// Evidence type/category
    pub item_type: String,
    /// Make/manufacturer
    pub make: Option<String>,
    /// Model
    pub model: Option<String>,
    /// Serial number
    pub serial_number: Option<String>,
    /// Capacity/size description
    pub capacity: Option<String>,
    /// Condition when received
    pub condition: String,
    /// Original evidence acquisition date
    pub acquisition_date: String,
    /// Date/time item entered chain of custody
    pub entered_custody_date: String,
    /// Who submitted/surrendered the evidence
    pub submitted_by: String,
    /// Who received the evidence
    pub received_by: String,
    /// Location where evidence was received
    pub received_location: Option<String>,
    /// Storage location
    pub storage_location: Option<String>,
    /// Reason for submission
    pub reason_submitted: Option<String>,
    /// Transfer records for this item
    #[serde(default)]
    pub transfers: Vec<CocTransfer>,
    /// Hash values at intake
    #[serde(default)]
    pub intake_hashes: Vec<CocHashValue>,
    /// Notes
    pub notes: Option<String>,
    /// Disposition status
    pub disposition: Option<String>,
    /// Disposition date
    pub disposition_date: Option<String>,
    /// Disposition notes
    pub disposition_notes: Option<String>,
}

/// COC transfer/handoff record
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CocTransfer {
    /// Internal UI identifier
    pub id: String,
    /// Date/time of transfer
    pub timestamp: String,
    /// Person releasing custody
    pub released_by: String,
    /// Person receiving custody
    pub received_by: String,
    /// Purpose of transfer
    pub purpose: String,
    /// Location of transfer
    pub location: Option<String>,
    /// Transfer method (in-person, courier, mail)
    pub method: Option<String>,
    /// Notes
    pub notes: Option<String>,
}

/// Hash value stored with a COC item
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CocHashValue {
    /// Hash algorithm (e.g., "MD5", "SHA-256")
    pub algorithm: String,
    /// Hash value string
    pub value: String,
    /// Whether the hash has been verified
    pub verified: Option<bool>,
    /// Timestamp of hash computation
    pub timestamp: Option<String>,
    /// Reference to evidence item
    pub item_reference: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // HashAlgorithm
    // =========================================================================

    #[test]
    fn test_hash_algorithm_as_str_all_variants() {
        assert_eq!(HashAlgorithm::MD5.as_str(), "MD5");
        assert_eq!(HashAlgorithm::SHA1.as_str(), "SHA-1");
        assert_eq!(HashAlgorithm::SHA256.as_str(), "SHA-256");
        assert_eq!(HashAlgorithm::SHA512.as_str(), "SHA-512");
        assert_eq!(HashAlgorithm::Blake2b.as_str(), "BLAKE2b");
        assert_eq!(HashAlgorithm::Blake3.as_str(), "BLAKE3");
        assert_eq!(HashAlgorithm::XXH3.as_str(), "XXH3");
        assert_eq!(HashAlgorithm::XXH64.as_str(), "XXH64");
    }

    #[test]
    fn test_hash_algorithm_serialization_roundtrip() {
        for algo in [
            HashAlgorithm::MD5,
            HashAlgorithm::SHA1,
            HashAlgorithm::SHA256,
            HashAlgorithm::SHA512,
            HashAlgorithm::Blake2b,
            HashAlgorithm::Blake3,
            HashAlgorithm::XXH3,
            HashAlgorithm::XXH64,
        ] {
            let json = serde_json::to_string(&algo).unwrap();
            let back: HashAlgorithm = serde_json::from_str(&json).unwrap();
            assert_eq!(back, algo);
        }
    }

    // =========================================================================
    // deserialize_hash_algorithm
    // =========================================================================

    #[test]
    fn test_deserialize_hash_algorithm_from_string_variants() {
        // Test via HashRecord deserialization
        let cases = [
            ("MD5", HashAlgorithm::MD5),
            ("SHA1", HashAlgorithm::SHA1),
            ("SHA-1", HashAlgorithm::SHA1),
            ("SHA256", HashAlgorithm::SHA256),
            ("SHA-256", HashAlgorithm::SHA256),
            ("SHA512", HashAlgorithm::SHA512),
            ("SHA-512", HashAlgorithm::SHA512),
            ("BLAKE2B", HashAlgorithm::Blake2b),
            ("BLAKE2", HashAlgorithm::Blake2b),
            ("BLAKE3", HashAlgorithm::Blake3),
            ("XXH3", HashAlgorithm::XXH3),
            ("XXH64", HashAlgorithm::XXH64),
        ];
        for (input, expected) in cases {
            let json = format!(r#"{{"item":"test","algorithm":"{}","value":"abc"}}"#, input);
            let record: HashRecord = serde_json::from_str(&json).unwrap();
            assert_eq!(record.algorithm, expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_deserialize_hash_algorithm_unknown_fails() {
        let json = r#"{"item":"test","algorithm":"UNKNOWN","value":"abc"}"#;
        let result: Result<HashRecord, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    // =========================================================================
    // HashRecord defaults and deserialization
    // =========================================================================

    #[test]
    fn test_hash_record_default() {
        let record = HashRecord::default();
        assert_eq!(record.algorithm, HashAlgorithm::SHA256);
        assert!(record.item.is_empty());
        assert!(record.verified.is_none());
    }

    #[test]
    fn test_hash_record_item_reference_alias() {
        let json = r#"{"item_reference":"disk.E01","algorithm":"MD5","value":"abc123"}"#;
        let record: HashRecord = serde_json::from_str(json).unwrap();
        assert_eq!(record.item, "disk.E01");
    }

    // =========================================================================
    // CustodyRecord builder
    // =========================================================================

    #[test]
    fn test_custody_record_builder() {
        let record = CustodyRecord::new("E001", "Alice", "Bob")
            .with_purpose("Analysis")
            .with_location("Lab 1")
            .with_notes("Sealed bag");

        assert_eq!(record.evidence_id, "E001");
        assert_eq!(record.released_by, "Alice");
        assert_eq!(record.received_by, "Bob");
        assert_eq!(record.purpose.unwrap(), "Analysis");
        assert_eq!(record.location.unwrap(), "Lab 1");
        assert_eq!(record.notes.unwrap(), "Sealed bag");
    }

    #[test]
    fn test_custody_record_with_timestamp() {
        let ts = Utc::now();
        let record = CustodyRecord::new("E1", "A", "B").with_timestamp(ts);
        assert_eq!(record.timestamp, ts);
    }

    // =========================================================================
    // TimelineEvent default
    // =========================================================================

    #[test]
    fn test_timeline_event_default() {
        let event = TimelineEvent::default();
        assert!(event.timestamp_type.is_empty());
        assert!(event.description.is_empty());
        assert!(event.artifact.is_none());
    }
}
