// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { Accessor } from "solid-js";
import type { EwfInfo } from "../../../types/containerInfo";
import type { EwfImageInfo } from "../../../api/ewfExport";

/** Shared style constants passed through from MetadataPanel */
export interface RowStyles {
  categoryHeader: string;
  rowBase: string;
  rowGrid: string;
  rowClickable: string;
  keyStyle: string;
  valueStyle: string;
  offsetStyle: string;
  offsetClickable: string;
}

export interface ContainerDetailsSectionProps extends RowStyles {
  ewf: Accessor<EwfInfo>;
  headerDataStart: number;
  volumeDataStart: number;
  isExpanded: (key: string) => boolean;
  toggleCategory: (key: string) => void;
  handleRowClick: (offset: number | undefined | null, size?: number) => void;
  enhancedEwfInfo: Accessor<EwfImageInfo | null>;
  enhancedEwfLoading: Accessor<boolean>;
  enhancedEwfError: Accessor<string | null>;
  fetchEnhancedEwfInfo: () => void;
}
