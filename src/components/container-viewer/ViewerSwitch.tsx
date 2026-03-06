// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ViewerSwitch — Routes the preview to the appropriate lazy-loaded viewer
 * based on file type memos. Wrapped in Suspense + ErrorBoundary.
 */

import { Switch, Match, Suspense, type Accessor } from "solid-js";
import { CompactErrorBoundary } from "../ErrorBoundary";
import { HexViewer } from "../HexViewer";
import { TextViewer } from "../TextViewer";
import type { SelectedEntry } from "../EvidenceTree";
import type { ViewerMetadataSection } from "../../types/viewerMetadata";
import type { ContentDetectResult } from "./types";
import {
  DocumentViewer,
  PdfViewer,
  SpreadsheetViewer,
  ImageViewer,
  EmailViewer,
  PstViewer,
  PlistViewer,
  ExifPanel,
  BinaryViewer,
  RegistryViewer,
  DatabaseViewer,
  OfficeViewer,
} from "./lazyViewers";

export interface ViewerSwitchProps {
  entry: SelectedEntry;
  previewPath: string;
  detectedFormat: Accessor<ContentDetectResult | null>;
  onMetadata: (section: ViewerMetadataSection | null) => void;
  // File type flags
  fileIsPdf: Accessor<boolean>;
  fileIsImage: Accessor<boolean>;
  fileIsSpreadsheet: Accessor<boolean>;
  fileIsOffice: Accessor<boolean>;
  fileIsEmail: Accessor<boolean>;
  fileIsPst: Accessor<boolean>;
  fileIsPlist: Accessor<boolean>;
  fileIsBinary: Accessor<boolean>;
  fileIsRegistry: Accessor<boolean>;
  fileIsDatabase: Accessor<boolean>;
  fileIsDetectedText: Accessor<boolean>;
  fileIsDocument: Accessor<boolean>;
}

export function ViewerSwitch(props: ViewerSwitchProps) {
  return (
    <Suspense
      fallback={
        <div class="flex items-center justify-center h-full text-txt-muted text-sm">
          Loading viewer...
        </div>
      }
    >
      <CompactErrorBoundary name="ViewerSwitch">
        <Switch
          fallback={
            <DocumentViewer path={props.previewPath} onMetadata={props.onMetadata} />
          }
        >
          <Match when={props.fileIsPdf()}>
            <PdfViewer path={props.previewPath} />
          </Match>
          <Match when={props.fileIsImage()}>
            <ImageViewer path={props.previewPath} />
            <ExifPanel path={props.previewPath} onMetadata={props.onMetadata} class="hidden" />
          </Match>
          <Match when={props.fileIsSpreadsheet()}>
            <SpreadsheetViewer path={props.previewPath} onMetadata={props.onMetadata} />
          </Match>
          <Match when={props.fileIsOffice()}>
            <OfficeViewer path={props.previewPath} onMetadata={props.onMetadata} />
          </Match>
          <Match when={props.fileIsEmail()}>
            <EmailViewer path={props.previewPath} onMetadata={props.onMetadata} />
          </Match>
          <Match when={props.fileIsPst()}>
            <PstViewer path={props.previewPath} onMetadata={props.onMetadata} />
          </Match>
          <Match when={props.fileIsPlist()}>
            <PlistViewer path={props.previewPath} onMetadata={props.onMetadata} />
          </Match>
          <Match when={props.fileIsBinary()}>
            <BinaryViewer path={props.previewPath} onMetadata={props.onMetadata} />
          </Match>
          <Match when={props.fileIsRegistry()}>
            <RegistryViewer path={props.previewPath} onMetadata={props.onMetadata} />
          </Match>
          <Match when={props.fileIsDatabase()}>
            <DatabaseViewer path={props.previewPath} onMetadata={props.onMetadata} />
          </Match>
          <Match when={props.fileIsDetectedText()}>
            <TextViewer
              file={{
                path: props.previewPath,
                filename: props.entry.name,
                container_type: "",
                size: props.entry.size,
                segment_count: 1,
              }}
            />
          </Match>
          <Match when={props.detectedFormat()?.viewerType === "Hex"}>
            <HexViewer entry={props.entry} />
          </Match>
          <Match when={props.detectedFormat()?.viewerType === "Archive"}>
            <HexViewer entry={props.entry} />
          </Match>
        </Switch>
      </CompactErrorBoundary>
    </Suspense>
  );
}
