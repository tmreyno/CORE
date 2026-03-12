// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Search query engine for the Tantivy search index.
//!
//! Parses user queries, applies filters, executes search, and generates
//! result snippets with highlighted matches.

use std::ops::Bound;

use tantivy::collector::TopDocs;
use tantivy::query::{AllQuery, BooleanQuery, Occur, Query, QueryParser, RangeQuery, TermQuery};
use tantivy::schema::{IndexRecordOption, Value};
use tantivy::snippet::SnippetGenerator;
use tantivy::{TantivyDocument, Term};
use tracing::debug;

use super::SearchIndex;

// =============================================================================
// Search Result Types
// =============================================================================

/// A search result from the Tantivy index.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    /// Unique document ID
    pub doc_id: String,
    /// Container file path
    pub container_path: String,
    /// Container type (e.g., "ad1", "e01", "zip")
    pub container_type: String,
    /// Full path within the container
    pub entry_path: String,
    /// Filename
    pub filename: String,
    /// File extension
    pub extension: String,
    /// File size in bytes
    pub size: u64,
    /// Last modified timestamp (unix)
    pub modified: i64,
    /// Is this a directory?
    pub is_dir: bool,
    /// File category
    pub file_category: String,
    /// BM25 relevance score
    pub score: f32,
    /// Snippet with `<mark>` highlighted matches (from content or filename)
    pub snippet: String,
    /// Whether this was a content match (vs metadata/filename match)
    pub content_match: bool,
}

/// Search options.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchOptions {
    /// The user's search query string
    pub query: String,
    /// Maximum results to return
    #[serde(default = "default_limit")]
    pub limit: usize,
    /// Filter by container type(s): ["ad1", "e01", "zip", ...]
    #[serde(default)]
    pub container_types: Vec<String>,
    /// Filter by file extension(s): ["pdf", "docx", "txt", ...]
    #[serde(default)]
    pub extensions: Vec<String>,
    /// Filter by file category(s): ["document", "email", "code", ...]
    #[serde(default)]
    pub categories: Vec<String>,
    /// Minimum file size (bytes)
    pub min_size: Option<u64>,
    /// Maximum file size (bytes)
    pub max_size: Option<u64>,
    /// Include directories in results
    #[serde(default)]
    pub include_dirs: bool,
    /// Search file content (in addition to filename/path)
    #[serde(default = "default_true")]
    pub search_content: bool,
    /// Filter by specific container path
    pub container_path: Option<String>,
}

fn default_limit() -> usize {
    100
}

fn default_true() -> bool {
    true
}

/// Search results with aggregate info.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResults {
    /// Matching results (up to options.limit)
    pub hits: Vec<SearchHit>,
    /// Total number of matching documents (before limit)
    pub total_hits: u64,
    /// How long the search took (ms)
    pub elapsed_ms: u64,
    /// Facet counts by category
    pub category_counts: Vec<FacetCount>,
    /// Facet counts by container type
    pub container_type_counts: Vec<FacetCount>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FacetCount {
    pub label: String,
    pub count: u64,
}

// =============================================================================
// Query Execution
// =============================================================================

/// Execute a search against the index.
pub fn search(index: &SearchIndex, options: &SearchOptions) -> Result<SearchResults, String> {
    let start = std::time::Instant::now();
    let searcher = index.searcher();
    let fields = &index.fields;

    // Build the query
    let query = build_query(index, options)?;

    // Execute search with extra capacity for post-filtering
    let fetch_limit = options.limit * 2 + 50; // Over-fetch to handle dir filtering
    let top_docs = searcher
        .search(&query, &TopDocs::with_limit(fetch_limit))
        .map_err(|e| format!("Search failed: {}", e))?;

    let total_hits = top_docs.len() as u64;

    // Build snippet generators for filename and content
    let filename_snippet_gen = SnippetGenerator::create(&searcher, &query, fields.filename)
        .map_err(|e| format!("Snippet gen (filename): {}", e))?;

    let content_snippet_gen = if options.search_content {
        SnippetGenerator::create(&searcher, &query, fields.content).ok()
    } else {
        None
    };

    // Collect results
    let mut hits = Vec::with_capacity(options.limit);
    for (score, doc_addr) in top_docs {
        if hits.len() >= options.limit {
            break;
        }

        let doc = searcher
            .doc::<TantivyDocument>(doc_addr)
            .map_err(|e| format!("Failed to retrieve doc: {}", e))?;

        let is_dir_val = doc
            .get_first(fields.is_dir)
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        // Filter out directories if not requested
        if is_dir_val == 1 && !options.include_dirs {
            continue;
        }

        let doc_id = get_text(&doc, fields.doc_id);
        let container_path = get_text(&doc, fields.container_path);
        let container_type = get_text(&doc, fields.container_type);
        let entry_path = get_text(&doc, fields.entry_path);
        let filename = get_text(&doc, fields.filename);
        let extension = get_text(&doc, fields.extension);
        let file_category = get_text(&doc, fields.file_category);
        let size = doc
            .get_first(fields.size)
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let modified = doc
            .get_first(fields.modified)
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        // Generate snippet — prefer content snippet if available
        let (snippet, content_match) = if let Some(ref gen) = content_snippet_gen {
            let content_snip = gen.snippet_from_doc(&doc);
            let snippet_html = content_snip.to_html();
            if snippet_html.contains("<b>") {
                (snippet_html, true)
            } else {
                // Fall back to filename snippet
                let fn_snip = filename_snippet_gen.snippet_from_doc(&doc);
                let fn_html = fn_snip.to_html();
                if fn_html.contains("<b>") {
                    (fn_html, false)
                } else {
                    (String::new(), false)
                }
            }
        } else {
            let fn_snip = filename_snippet_gen.snippet_from_doc(&doc);
            (fn_snip.to_html(), false)
        };

        hits.push(SearchHit {
            doc_id,
            container_path,
            container_type,
            entry_path,
            filename,
            extension,
            size,
            modified,
            is_dir: is_dir_val == 1,
            file_category,
            score,
            snippet,
            content_match,
        });
    }

    // Build facet counts (cheap — counts from results, not from all docs)
    let category_counts = count_facets(&hits, |h| &h.file_category);
    let container_type_counts = count_facets(&hits, |h| &h.container_type);

    let elapsed_ms = start.elapsed().as_millis() as u64;
    debug!(
        "Search '{}' returned {} hits in {}ms",
        options.query,
        hits.len(),
        elapsed_ms
    );

    Ok(SearchResults {
        hits,
        total_hits,
        elapsed_ms,
        category_counts,
        container_type_counts,
    })
}

// =============================================================================
// Query Building
// =============================================================================

fn build_query(index: &SearchIndex, options: &SearchOptions) -> Result<Box<dyn Query>, String> {
    let fields = &index.fields;
    let mut subqueries: Vec<(Occur, Box<dyn Query>)> = Vec::new();

    // Main text query — search filename and optionally content
    if !options.query.is_empty() {
        let text_query = build_text_query(index, &options.query, options.search_content)?;
        subqueries.push((Occur::Must, text_query));
    }

    // Filter: container type
    if !options.container_types.is_empty() {
        let type_query = build_terms_or_query(fields.container_type, &options.container_types);
        subqueries.push((Occur::Must, type_query));
    }

    // Filter: extensions
    if !options.extensions.is_empty() {
        let ext_query = build_terms_or_query(fields.extension, &options.extensions);
        subqueries.push((Occur::Must, ext_query));
    }

    // Filter: categories
    if !options.categories.is_empty() {
        let cat_query = build_terms_or_query(fields.file_category, &options.categories);
        subqueries.push((Occur::Must, cat_query));
    }

    // Filter: specific container
    if let Some(ref cp) = options.container_path {
        let term = Term::from_field_text(fields.container_path, cp);
        let q = TermQuery::new(term, IndexRecordOption::Basic);
        subqueries.push((Occur::Must, Box::new(q)));
    }

    // Filter: size range
    if let Some(min) = options.min_size {
        let q = RangeQuery::new_u64_bounds(
            "size".to_string(),
            Bound::Included(min),
            Bound::Included(u64::MAX),
        );
        subqueries.push((Occur::Must, Box::new(q)));
    }
    if let Some(max) = options.max_size {
        let q = RangeQuery::new_u64_bounds(
            "size".to_string(),
            Bound::Included(0),
            Bound::Included(max),
        );
        subqueries.push((Occur::Must, Box::new(q)));
    }

    // Filter: exclude directories
    if !options.include_dirs {
        let dir_term = Term::from_field_u64(fields.is_dir, 0);
        let q = TermQuery::new(dir_term, IndexRecordOption::Basic);
        subqueries.push((Occur::Must, Box::new(q)));
    }

    if subqueries.is_empty() {
        // No filters, no query → return all docs
        Ok(Box::new(AllQuery))
    } else if subqueries.len() == 1 {
        Ok(subqueries.pop().unwrap().1)
    } else {
        Ok(Box::new(BooleanQuery::new(subqueries)))
    }
}

/// Build the text query for the user's search string.
/// Searches across filename and optionally content fields with boosting.
fn build_text_query(
    index: &SearchIndex,
    query_str: &str,
    search_content: bool,
) -> Result<Box<dyn Query>, String> {
    let fields = &index.fields;

    let search_fields = if search_content {
        vec![fields.filename, fields.entry_path, fields.content]
    } else {
        vec![fields.filename, fields.entry_path]
    };

    let mut parser = QueryParser::for_index(&index.index, search_fields);

    // Boost filename matches over content matches
    parser.set_field_boost(fields.filename, 3.0);
    parser.set_field_boost(fields.entry_path, 1.5);
    if search_content {
        parser.set_field_boost(fields.content, 1.0);
    }

    // Enable fuzzy search for typo tolerance (1 edit distance)
    parser.set_field_fuzzy(fields.filename, true, 1, true);

    let query = parser
        .parse_query(query_str)
        .map_err(|e| format!("Failed to parse query '{}': {}", query_str, e))?;

    Ok(query)
}

/// Build an OR query from multiple string terms for a STRING field.
fn build_terms_or_query(field: tantivy::schema::Field, values: &[String]) -> Box<dyn Query> {
    let subqueries: Vec<(Occur, Box<dyn Query>)> = values
        .iter()
        .map(|v| {
            let term = Term::from_field_text(field, &v.to_lowercase());
            let q: Box<dyn Query> = Box::new(TermQuery::new(term, IndexRecordOption::Basic));
            (Occur::Should, q)
        })
        .collect();
    Box::new(BooleanQuery::new(subqueries))
}

// =============================================================================
// Helpers
// =============================================================================

/// Extract a text field value from a Tantivy document.
fn get_text(doc: &tantivy::TantivyDocument, field: tantivy::schema::Field) -> String {
    doc.get_first(field)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

/// Count facets from result hits.
fn count_facets<F>(hits: &[SearchHit], key_fn: F) -> Vec<FacetCount>
where
    F: Fn(&SearchHit) -> &str,
{
    let mut counts: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
    for hit in hits {
        let key = key_fn(hit).to_string();
        if !key.is_empty() {
            *counts.entry(key).or_insert(0) += 1;
        }
    }
    let mut result: Vec<FacetCount> = counts
        .into_iter()
        .map(|(label, count)| FacetCount { label, count })
        .collect();
    result.sort_by(|a, b| b.count.cmp(&a.count));
    result
}
