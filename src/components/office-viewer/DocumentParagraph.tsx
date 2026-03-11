// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { ParagraphHint } from "./types";

/** Renders a single paragraph with styling based on its ParagraphHint. */
export function DocumentParagraph(props: { text: string; hint: ParagraphHint }) {
  switch (props.hint) {
    case "title":
      return (
        <h1 class="text-2xl font-bold text-txt mt-6 mb-3 leading-tight select-text">
          {props.text}
        </h1>
      );
    case "subtitle":
      return (
        <h2 class="text-lg font-medium text-txt-secondary mt-1 mb-4 leading-snug select-text">
          {props.text}
        </h2>
      );
    case "heading1":
      return (
        <h2 class="text-xl font-bold text-txt mt-8 mb-2 leading-tight select-text border-b border-border pb-1">
          {props.text}
        </h2>
      );
    case "heading2":
      return (
        <h3 class="text-lg font-semibold text-txt mt-6 mb-2 leading-snug select-text">
          {props.text}
        </h3>
      );
    case "heading3":
      return (
        <h4 class="text-base font-semibold text-txt mt-5 mb-1.5 leading-snug select-text">
          {props.text}
        </h4>
      );
    case "heading4":
      return (
        <h5 class="text-sm font-semibold text-txt-secondary mt-4 mb-1 leading-snug select-text">
          {props.text}
        </h5>
      );
    case "listItem":
      return (
        <div class="flex gap-2 ml-6 my-1 select-text">
          <span class="text-txt-faint select-none mt-0.5">•</span>
          <p class="text-lg text-txt leading-relaxed">
            {props.text}
          </p>
        </div>
      );
    case "quote":
      return (
        <blockquote class="ml-4 pl-4 border-l-2 border-border my-3 select-text">
          <p class="text-lg text-txt-secondary italic leading-relaxed">
            {props.text}
          </p>
        </blockquote>
      );
    default:
      return (
        <p class="text-lg text-txt leading-relaxed my-2 whitespace-pre-wrap select-text">
          {props.text}
        </p>
      );
  }
}
