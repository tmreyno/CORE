// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! RDF/Turtle serialization and parsing for AFF4 metadata.
//!
//! AFF4 stores all metadata in `information.turtle` — an RDF file using
//! Turtle syntax with subject-predicate-object triples. This module provides
//! a minimal Turtle emitter and parser (not a full RDF engine).

use std::collections::HashMap;

use crate::error::Aff4Result;

// ─── RDF Graph ───────────────────────────────────────────────────────────────

/// A simple RDF graph as subject → (predicate → values) triples.
///
/// Values are stored as strings. For typed literals, the full Turtle
/// representation is stored (e.g., `"32768"^^<xsd:integer>`).
#[derive(Debug, Clone, Default)]
pub struct RdfGraph {
    /// subject_uri → predicate_uri → list of object values
    triples: HashMap<String, HashMap<String, Vec<String>>>,
}

impl RdfGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a triple to the graph.
    pub fn add(&mut self, subject: &str, predicate: &str, object: &str) {
        self.triples
            .entry(subject.to_string())
            .or_default()
            .entry(predicate.to_string())
            .or_default()
            .push(object.to_string());
    }

    /// Get all objects for a subject-predicate pair.
    pub fn get(&self, subject: &str, predicate: &str) -> Option<&Vec<String>> {
        self.triples
            .get(subject)
            .and_then(|preds| preds.get(predicate))
    }

    /// Get the first (or only) object for a subject-predicate pair.
    pub fn get_first(&self, subject: &str, predicate: &str) -> Option<&str> {
        self.get(subject, predicate)
            .and_then(|v| v.first())
            .map(|s| s.as_str())
    }

    /// Get all subjects in the graph.
    pub fn subjects(&self) -> impl Iterator<Item = &str> {
        self.triples.keys().map(|s| s.as_str())
    }

    /// Get all predicate-value pairs for a subject.
    pub fn predicates(&self, subject: &str) -> Option<&HashMap<String, Vec<String>>> {
        self.triples.get(subject)
    }

    /// Check whether the graph contains any triples.
    pub fn is_empty(&self) -> bool {
        self.triples.is_empty()
    }

    /// Get subjects that have a specific type triple.
    pub fn subjects_with_type(&self, type_uri: &str) -> Vec<String> {
        let rdf_type = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
        self.triples
            .iter()
            .filter(|(_, preds)| {
                preds
                    .get(rdf_type)
                    .map(|types| types.iter().any(|t| t == type_uri))
                    .unwrap_or(false)
            })
            .map(|(subj, _)| subj.clone())
            .collect()
    }
}

// ─── Turtle Serializer ───────────────────────────────────────────────────────

/// Standard prefixes used in AFF4 Turtle files.
const TURTLE_PREFIXES: &[(&str, &str)] = &[
    ("aff4", "http://aff4.org/Schema#"),
    ("rdf", "http://www.w3.org/1999/02/22-rdf-syntax-ns#"),
    ("xsd", "http://www.w3.org/2001/XMLSchema#"),
    ("dc", "http://purl.org/dc/elements/1.1/"),
];

/// Serialize an RDF graph to Turtle format.
pub fn serialize_turtle(graph: &RdfGraph) -> String {
    let mut out = String::new();

    // Write prefixes
    for (prefix, uri) in TURTLE_PREFIXES {
        out.push_str(&format!("@prefix {}: <{}> .\n", prefix, uri));
    }
    out.push('\n');

    // Sort subjects for deterministic output
    let mut subjects: Vec<&str> = graph.subjects().collect();
    subjects.sort();

    for subject in subjects {
        if let Some(predicates) = graph.predicates(subject) {
            out.push_str(&format!("<{}>\n", subject));

            let mut preds: Vec<(&String, &Vec<String>)> = predicates.iter().collect();
            preds.sort_by_key(|(k, _)| k.as_str());

            for (i, (predicate, objects)) in preds.iter().enumerate() {
                let pred_str = compact_uri(predicate);
                let is_last = i == preds.len() - 1;

                for (j, object) in objects.iter().enumerate() {
                    let obj_str = format_object(object);
                    let sep = if is_last && j == objects.len() - 1 {
                        "."
                    } else {
                        ";"
                    };
                    out.push_str(&format!("    {} {} {}\n", pred_str, obj_str, sep));
                }
            }
            out.push('\n');
        }
    }

    out
}

/// Compact a full URI using known prefixes.
fn compact_uri(uri: &str) -> String {
    for (prefix, namespace) in TURTLE_PREFIXES {
        if let Some(local) = uri.strip_prefix(namespace) {
            return format!("{}:{}", prefix, local);
        }
    }
    format!("<{}>", uri)
}

/// Format an object value for Turtle output.
fn format_object(value: &str) -> String {
    if value.starts_with("http://") || value.starts_with("https://") || value.starts_with("aff4://")
    {
        // URI reference
        compact_uri(value)
    } else if let Some(stripped) = value.strip_prefix("^^int:") {
        // Integer literal
        format!(
            "\"{}\"^^<http://www.w3.org/2001/XMLSchema#integer>",
            stripped
        )
    } else if let Some(stripped) = value.strip_prefix("^^long:") {
        // Long literal
        format!("\"{}\"^^<http://www.w3.org/2001/XMLSchema#long>", stripped)
    } else if let Some(stripped) = value.strip_prefix("^^dateTime:") {
        // DateTime literal
        format!(
            "\"{}\"^^<http://www.w3.org/2001/XMLSchema#dateTime>",
            stripped
        )
    } else {
        // String literal
        format!("\"{}\"", escape_turtle_string(value))
    }
}

/// Escape special characters in a Turtle string literal.
fn escape_turtle_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

// ─── Convenience builders ────────────────────────────────────────────────────

/// Add an integer-valued triple.
pub fn add_integer(graph: &mut RdfGraph, subject: &str, predicate: &str, value: u64) {
    graph.add(subject, predicate, &format!("^^int:{}", value));
}

/// Add a long-valued triple.
pub fn add_long(graph: &mut RdfGraph, subject: &str, predicate: &str, value: i64) {
    graph.add(subject, predicate, &format!("^^long:{}", value));
}

/// Add a dateTime-valued triple (ISO 8601).
pub fn add_datetime(graph: &mut RdfGraph, subject: &str, predicate: &str, iso: &str) {
    graph.add(subject, predicate, &format!("^^dateTime:{}", iso));
}

/// Add a URI-valued triple.
pub fn add_uri(graph: &mut RdfGraph, subject: &str, predicate: &str, uri: &str) {
    graph.add(subject, predicate, uri);
}

/// Add a hash triple in the AFF4 format: `aff4:hash "hex_digest"^^<aff4:AlgName>`.
///
/// # Arguments
/// - `predicate` — the RDF predicate (e.g., `rdf_predicates::STORED_HASH`,
///   `MAP_POINT_HASH`, `MAP_IDX_HASH`, `BLOCK_MAP_HASH`)
/// - `algorithm_uri` — the hash algorithm URI (e.g., from `Aff4HashAlgorithm::rdf_uri()`)
/// - `hex_digest` — the hex-encoded digest string
pub fn add_hash(
    graph: &mut RdfGraph,
    subject: &str,
    predicate: &str,
    algorithm_uri: &str,
    hex_digest: &str,
) {
    let hash_value = format!("^^hash:{}:{}", algorithm_uri, hex_digest);
    graph.add(subject, predicate, &hash_value);
}

// ─── Turtle Parser ───────────────────────────────────────────────────────────

/// Parse Turtle content into an RDF graph.
///
/// This is a simplified parser that handles the subset of Turtle used by AFF4:
/// - @prefix declarations
/// - Subject-predicate-object triples with `;` and `.` terminators
/// - URI references `<uri>`, prefixed names `prefix:local`
/// - String literals `"value"`, typed literals `"value"^^<type>`
/// - Integer literals, long literals
pub fn parse_turtle(content: &str) -> Aff4Result<RdfGraph> {
    let mut graph = RdfGraph::new();
    let mut prefixes: HashMap<String, String> = HashMap::new();

    // Add standard prefixes as defaults
    for (p, u) in TURTLE_PREFIXES {
        prefixes.insert(p.to_string(), u.to_string());
    }

    let mut current_subject: Option<String> = None;
    let mut in_multiline = false;

    for line in content.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // @prefix declarations
        if line.starts_with("@prefix") || line.starts_with("@base") {
            if let Some(rest) = line.strip_prefix("@prefix") {
                let rest = rest.trim().trim_end_matches('.');
                if let Some(colon_pos) = rest.find(':') {
                    let prefix = rest[..colon_pos].trim().to_string();
                    let uri = rest[colon_pos + 1..]
                        .trim()
                        .trim_start_matches('<')
                        .trim_end_matches('>')
                        .trim()
                        .to_string();
                    prefixes.insert(prefix, uri);
                }
            }
            continue;
        }

        // Check if this starts a new subject (line starts with <uri>)
        if line.starts_with('<') && !in_multiline {
            // Extract subject URI
            if let Some(end) = line.find('>') {
                current_subject = Some(line[1..end].to_string());
                // Check if there's a predicate-object on the same line
                let rest = line[end + 1..].trim();
                if !rest.is_empty() {
                    parse_predicate_object(rest, &current_subject, &prefixes, &mut graph);
                }
                in_multiline = true;
            }
            continue;
        }

        // Continuation of current subject
        if in_multiline {
            if line.ends_with('.') {
                // End of this subject block
                let rest = line.trim_end_matches('.').trim();
                if !rest.is_empty() {
                    parse_predicate_object(rest, &current_subject, &prefixes, &mut graph);
                }
                in_multiline = false;
                continue;
            }

            let rest = line.trim_end_matches(';').trim();
            if !rest.is_empty() {
                parse_predicate_object(rest, &current_subject, &prefixes, &mut graph);
            }
        }
    }

    Ok(graph)
}

/// Parse a "predicate object" pair from a Turtle line.
fn parse_predicate_object(
    line: &str,
    subject: &Option<String>,
    prefixes: &HashMap<String, String>,
    graph: &mut RdfGraph,
) {
    let subject = match subject {
        Some(s) => s,
        None => return,
    };

    let (pred_str, obj_str) = match split_first_token(line) {
        Some(pair) => pair,
        None => return,
    };

    let predicate = resolve_uri(&pred_str, prefixes);
    let object = parse_object(&obj_str, prefixes);

    graph.add(subject, &predicate, &object);
}

/// Split line into first token and remainder.
fn split_first_token(line: &str) -> Option<(String, String)> {
    let line = line.trim();

    if line.starts_with('<') {
        // URI token
        if let Some(end) = line.find('>') {
            let token = line[..=end].to_string();
            let rest = line[end + 1..].trim().to_string();
            return Some((token, rest));
        }
    }

    // Prefixed name or other token — split on whitespace
    if let Some(pos) = line.find(char::is_whitespace) {
        let token = line[..pos].to_string();
        let rest = line[pos..].trim().to_string();
        Some((token, rest))
    } else {
        None
    }
}

/// Resolve a (possibly prefixed) URI to a full URI.
fn resolve_uri(token: &str, prefixes: &HashMap<String, String>) -> String {
    // Full URI: <http://...>
    if token.starts_with('<') && token.ends_with('>') {
        return token[1..token.len() - 1].to_string();
    }

    // Prefixed name: prefix:local
    if let Some(colon) = token.find(':') {
        let prefix = &token[..colon];
        let local = &token[colon + 1..];
        if let Some(ns) = prefixes.get(prefix) {
            return format!("{}{}", ns, local);
        }
    }

    token.to_string()
}

/// Parse an object value from Turtle.
fn parse_object(token: &str, prefixes: &HashMap<String, String>) -> String {
    let token = token.trim().trim_end_matches(';').trim_end_matches('.').trim();

    // URI reference
    if token.starts_with('<') && token.contains('>') {
        let end = token.find('>').unwrap();
        return token[1..end].to_string();
    }

    // Prefixed name
    if !token.starts_with('"') && token.contains(':') && !token.contains(' ') {
        return resolve_uri(token, prefixes);
    }

    // String literal (possibly typed)
    if token.starts_with('"') {
        return parse_string_literal(token);
    }

    // Plain value
    token.to_string()
}

/// Parse a Turtle string literal, handling typed literals.
fn parse_string_literal(token: &str) -> String {
    // Find the closing quote
    let content_start = 1; // after opening "
    let mut i = content_start;
    let bytes = token.as_bytes();

    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i += 2; // skip escaped char
        } else if bytes[i] == b'"' {
            break;
        } else {
            i += 1;
        }
    }

    let value = unescape_turtle_string(&token[content_start..i]);

    // Check for type annotation after closing quote
    let rest = &token[i + 1..]; // after closing "
    if let Some(type_part) = rest.strip_prefix("^^") {
        let type_uri = type_part
            .trim_start_matches('<')
            .trim_end_matches('>')
            .trim();

        // Return the raw value for most purposes
        // Callers can check the type via the predicate
        if type_uri.ends_with("#integer") || type_uri.ends_with("#long") {
            return value;
        }
        if type_uri.ends_with("#dateTime") {
            return value;
        }
        // Hash typed literal: value is hex, type is algorithm URI
        if type_uri.contains("aff4.org/Schema#") {
            // Return as hash-typed value so caller can extract algorithm
            return format!("{}|{}", type_uri, value);
        }
    }

    value
}

/// Unescape Turtle string escape sequences.
fn unescape_turtle_string(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some('"') => result.push('"'),
                Some('\\') => result.push('\\'),
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_simple() {
        let mut graph = RdfGraph::new();
        graph.add(
            "aff4://vol-1",
            "http://www.w3.org/1999/02/22-rdf-syntax-ns#type",
            "http://aff4.org/Schema#ZipVolume",
        );
        add_integer(
            &mut graph,
            "aff4://vol-1/image",
            "http://aff4.org/Schema#chunkSize",
            32768,
        );

        let turtle = serialize_turtle(&graph);
        assert!(turtle.contains("@prefix aff4:"));
        assert!(turtle.contains("<aff4://vol-1>"));
        assert!(turtle.contains("rdf:type"));
        assert!(turtle.contains("aff4:ZipVolume"));
    }

    #[test]
    fn test_parse_simple() {
        let turtle = r#"
@prefix aff4: <http://aff4.org/Schema#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

<aff4://vol-1>
    rdf:type aff4:ZipVolume ;
    aff4:chunkSize "32768"^^<http://www.w3.org/2001/XMLSchema#integer> .
"#;

        let graph = parse_turtle(turtle).unwrap();
        assert!(!graph.is_empty());

        let types = graph.subjects_with_type("http://aff4.org/Schema#ZipVolume");
        assert!(types.contains(&"aff4://vol-1".to_string()));

        let chunk_size = graph.get_first(
            "aff4://vol-1",
            "http://aff4.org/Schema#chunkSize",
        );
        assert_eq!(chunk_size, Some("32768"));
    }

    #[test]
    fn test_roundtrip() {
        let mut graph = RdfGraph::new();
        graph.add(
            "aff4://test",
            "http://www.w3.org/1999/02/22-rdf-syntax-ns#type",
            "http://aff4.org/Schema#Image",
        );
        graph.add(
            "aff4://test",
            "http://purl.org/dc/elements/1.1/description",
            "Test image",
        );

        let turtle = serialize_turtle(&graph);
        let parsed = parse_turtle(&turtle).unwrap();

        let desc = parsed.get_first(
            "aff4://test",
            "http://purl.org/dc/elements/1.1/description",
        );
        assert_eq!(desc, Some("Test image"));
    }

    #[test]
    fn test_compact_uri() {
        assert_eq!(
            compact_uri("http://aff4.org/Schema#ZipVolume"),
            "aff4:ZipVolume"
        );
        assert_eq!(
            compact_uri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"),
            "rdf:type"
        );
        assert_eq!(
            compact_uri("http://example.org/foo"),
            "<http://example.org/foo>"
        );
    }
}
