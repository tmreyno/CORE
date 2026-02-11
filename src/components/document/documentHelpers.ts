// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Document Helper Utilities
 * 
 * Utility functions for document viewing:
 * - Format icon mapping
 * - Search within rendered HTML
 * - Print document
 * - Download as HTML
 */

import { getPreference } from "../preferences";

/**
 * Get emoji icon for document format
 */
export function getFormatIcon(format: string): string {
  switch (format.toLowerCase()) {
    case "pdf":
      return "📄";
    case "docx":
    case "doc":
      return "📝";
    case "html":
    case "htm":
      return "🌐";
    case "markdown":
    case "md":
      return "📋";
    default:
      return "📃";
  }
}

/**
 * Search within rendered HTML content
 * 
 * Highlights all occurrences of search query in the content container.
 * Returns number of matches found.
 * 
 * @param contentRef - Container element with rendered HTML
 * @param searchQuery - Text to search for
 * @returns Number of matches found
 */
export function performSearch(contentRef: HTMLElement | undefined, searchQuery: string): number {
  if (!searchQuery || !contentRef) {
    return 0;
  }

  // Clear previous highlights
  const highlighted = contentRef.querySelectorAll(".search-highlight");
  highlighted.forEach((el) => {
    const parent = el.parentNode;
    if (parent) {
      parent.replaceChild(document.createTextNode(el.textContent || ""), el);
      parent.normalize();
    }
  });

  // Walk text nodes to find and highlight matches
  let count = 0;
  const walk = document.createTreeWalker(contentRef, NodeFilter.SHOW_TEXT, null);
  const matches: { node: Text; start: number; length: number }[] = [];
  
  const caseSensitive = getPreference("caseSensitiveSearch");
  let node: Text | null;
  while ((node = walk.nextNode() as Text)) {
    const text = node.textContent || "";
    const searchText = caseSensitive ? text : text.toLowerCase();
    const searchTerm = caseSensitive ? searchQuery : searchQuery.toLowerCase();
    let start = 0;
    let idx: number;
    while ((idx = searchText.indexOf(searchTerm, start)) !== -1) {
      matches.push({ node, start: idx, length: searchQuery.length });
      count++;
      start = idx + 1;
    }
  }

  // Apply highlights in reverse order to preserve node offsets
  // Group matches by node to handle multiple matches in the same text node
  const nodeGroups = new Map<Text, { start: number; length: number }[]>();
  for (const match of matches) {
    const group = nodeGroups.get(match.node) || [];
    group.push({ start: match.start, length: match.length });
    nodeGroups.set(match.node, group);
  }

  for (const [textNode, nodeMatches] of nodeGroups) {
    const parent = textNode.parentNode;
    if (!parent) continue;

    const text = textNode.textContent || "";
    // Sort matches in reverse order so we can split from the end
    const sorted = nodeMatches.sort((a, b) => b.start - a.start);

    const frag = document.createDocumentFragment();
    let remaining = text;
    // Process from end to start
    for (const { start, length } of sorted) {
      const after = remaining.substring(start + length);
      const matched = remaining.substring(start, start + length);
      remaining = remaining.substring(0, start);

      if (after) frag.prepend(document.createTextNode(after));

      const highlight = document.createElement("span");
      highlight.className = "search-highlight";
      highlight.textContent = matched;
      frag.prepend(highlight);
    }
    if (remaining) frag.prepend(document.createTextNode(remaining));

    parent.replaceChild(frag, textNode);
  }

  // Scroll first match into view
  if (count > 0) {
    const firstHighlight = contentRef.querySelector(".search-highlight");
    firstHighlight?.scrollIntoView({ behavior: "smooth", block: "center" });
  }

  return count;
}

/**
 * Print document in new window
 * 
 * Opens rendered HTML in new window and triggers print dialog
 * 
 * @param htmlContent - Rendered HTML content to print
 */
export function printDocument(htmlContent: string | undefined): void {
  if (!htmlContent) return;

  const printWindow = window.open("", "_blank");
  if (printWindow) {
    printWindow.document.write(htmlContent);
    printWindow.document.close();
    printWindow.onload = () => {
      printWindow.print();
    };
  }
}

/**
 * Download document as HTML file
 * 
 * Creates downloadable HTML file from rendered content
 * 
 * @param htmlContent - Rendered HTML content
 * @param title - Document title for filename
 */
export function downloadHtml(htmlContent: string | undefined, title: string = "document"): void {
  if (!htmlContent) return;

  const blob = new Blob([htmlContent], { type: "text/html" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = `${title}.html`;
  a.click();
  URL.revokeObjectURL(url);
}
