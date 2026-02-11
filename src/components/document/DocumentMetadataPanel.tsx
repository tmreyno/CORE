// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * DocumentMetadataPanel Component
 * 
 * Sidebar panel displaying document metadata:
 * - Title, author, subject
 * - Format, file size, page count, word count
 * - Created/modified dates
 * - Creator, producer
 * - Keywords
 * - Encryption status
 */

import { Component, Show, For } from "solid-js";
import { HiOutlineInformationCircle, HiOutlineExclamationTriangle } from "solid-icons/hi";

interface DocumentMetadataDisplay {
  title?: string | null;
  author?: string | null;
  subject?: string | null;
  format?: string;
  wordCount?: string | number | null;
  createdDate?: string | null;
  modifiedDate?: string | null;
  creator?: string | null;
  producer?: string | null;
  keywords?: string[];
  encrypted?: boolean;
}

interface DocumentMetadataPanelProps {
  metadataDisplay: DocumentMetadataDisplay | null;
  documentTitle: string;
  fileSizeDisplay: string;
  pageCount: number;
}

export const DocumentMetadataPanel: Component<DocumentMetadataPanelProps> = (props) => {
  return (
    <div class="w-72 border-l border-border bg-bg-panel overflow-y-auto p-4">
      <h3 class="font-semibold mb-4 flex items-center gap-2">
        <HiOutlineInformationCircle class="w-5 h-5" />
        Document Metadata
      </h3>
      
      <div class="space-y-3 text-sm">
        <Show when={props.metadataDisplay?.title}>
          <div>
            <div class="text-txt-muted text-xs uppercase">Title</div>
            <div class="font-medium">{props.documentTitle}</div>
          </div>
        </Show>
        
        <Show when={props.metadataDisplay?.author}>
          <div>
            <div class="text-txt-muted text-xs uppercase">Author</div>
            <div>{props.metadataDisplay?.author}</div>
          </div>
        </Show>
        
        <Show when={props.metadataDisplay?.subject}>
          <div>
            <div class="text-txt-muted text-xs uppercase">Subject</div>
            <div>{props.metadataDisplay?.subject}</div>
          </div>
        </Show>
        
        <div>
          <div class="text-txt-muted text-xs uppercase">Format</div>
          <div>{props.metadataDisplay?.format}</div>
        </div>
        
        <div>
          <div class="text-txt-muted text-xs uppercase">File Size</div>
          <div>{props.fileSizeDisplay}</div>
        </div>
        
        <Show when={props.pageCount > 0}>
          <div>
            <div class="text-txt-muted text-xs uppercase">Pages</div>
            <div>{props.pageCount}</div>
          </div>
        </Show>
        
        <Show when={props.metadataDisplay?.wordCount}>
          <div>
            <div class="text-txt-muted text-xs uppercase">Word Count</div>
            <div>{props.metadataDisplay?.wordCount}</div>
          </div>
        </Show>
        
        <Show when={props.metadataDisplay?.createdDate}>
          <div>
            <div class="text-txt-muted text-xs uppercase">Created</div>
            <div>{props.metadataDisplay?.createdDate}</div>
          </div>
        </Show>
        
        <Show when={props.metadataDisplay?.modifiedDate}>
          <div>
            <div class="text-txt-muted text-xs uppercase">Modified</div>
            <div>{props.metadataDisplay?.modifiedDate}</div>
          </div>
        </Show>
        
        <Show when={props.metadataDisplay?.creator}>
          <div>
            <div class="text-txt-muted text-xs uppercase">Creator</div>
            <div>{props.metadataDisplay?.creator}</div>
          </div>
        </Show>
        
        <Show when={props.metadataDisplay?.producer}>
          <div>
            <div class="text-txt-muted text-xs uppercase">Producer</div>
            <div>{props.metadataDisplay?.producer}</div>
          </div>
        </Show>
        
        <Show when={(props.metadataDisplay?.keywords?.length ?? 0) > 0}>
          <div>
            <div class="text-txt-muted text-xs uppercase">Keywords</div>
            <div class="flex flex-wrap gap-1 mt-1">
              <For each={props.metadataDisplay?.keywords}>
                {(keyword) => (
                  <span class="px-2 py-0.5 bg-bg-hover rounded text-xs">
                    {keyword}
                  </span>
                )}
              </For>
            </div>
          </div>
        </Show>
        
        <Show when={props.metadataDisplay?.encrypted}>
          <div class="flex items-center gap-2 text-warning">
            <HiOutlineExclamationTriangle class="w-4 h-4" />
            <span>Encrypted document</span>
          </div>
        </Show>
      </div>
    </div>
  );
};
