// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show, createSignal, createMemo } from "solid-js";
import type { MetadataField } from "../HexViewer";
import { isE01Container } from "../EvidenceTree/containerDetection";
import { readEwfImageInfo, type EwfImageInfo } from "../../api/ewfExport";
import { HiOutlineClipboardDocument, HiOutlineArrowPath } from "../icons";
import { FormatHeader } from "./FormatHeader";
import { FileInfoSection } from "./FileInfoSection";
import { HexLocationsSection } from "./HexLocationsSection";
import { ContainerDetailsSection } from "./ContainerDetailsSection";
import { CategoryFieldsSection } from "./CategoryFieldsSection";
import { HexRegionsSection } from "./HexRegionsSection";
import { CATEGORY_ORDER, ROW_STYLES } from "./types";
import type { MetadataPanelProps } from "./types";

export type { MetadataPanelProps };

export function MetadataPanel(props: MetadataPanelProps) {
  // Track expanded categories
  const [expandedCategories, setExpandedCategories] = createSignal<Set<string>>(
    new Set(["Format", "Case Info", "Hashes", "Container", "_container", "_hexLocations"])
  );

  // Enhanced EWF info from libewf-ffi (loaded on demand)
  const [enhancedEwfInfo, setEnhancedEwfInfo] = createSignal<EwfImageInfo | null>(null);
  const [enhancedEwfLoading, setEnhancedEwfLoading] = createSignal(false);
  const [enhancedEwfError, setEnhancedEwfError] = createSignal<string | null>(null);

  const fetchEnhancedEwfInfo = async () => {
    const path = props.fileInfo?.path;
    if (!path) return;

    setEnhancedEwfLoading(true);
    setEnhancedEwfError(null);

    try {
      const info = await readEwfImageInfo(path);
      setEnhancedEwfInfo(info);
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      setEnhancedEwfError(msg);
    } finally {
      setEnhancedEwfLoading(false);
    }
  };

  const toggleCategory = (category: string) => {
    setExpandedCategories((prev) => {
      const next = new Set(prev);
      if (next.has(category)) {
        next.delete(category);
      } else {
        next.add(category);
      }
      return next;
    });
  };

  const isExpanded = (category: string) => expandedCategories().has(category);

  // Memoized: Group fields by category
  const groupedFields = createMemo(() => {
    const meta = props.metadata;
    if (!meta?.fields.length) return new Map<string, MetadataField[]>();

    const groups = new Map<string, MetadataField[]>();
    for (const field of meta.fields) {
      const category = field.category || "General";
      if (!groups.has(category)) {
        groups.set(category, []);
      }
      groups.get(category)!.push(field);
    }

    // Sort by preferred order
    const sortedGroups = new Map<string, MetadataField[]>();
    for (const cat of CATEGORY_ORDER) {
      if (groups.has(cat)) {
        sortedGroups.set(cat, groups.get(cat)!);
        groups.delete(cat);
      }
    }
    for (const [cat, fields] of groups) {
      sortedGroups.set(cat, fields);
    }

    return sortedGroups;
  });

  // Memoized: Get EWF info from container (either E01 or L01)
  const ewfInfo = createMemo(() => {
    const info = props.containerInfo;
    if (!info) return null;
    return info.e01 || info.l01 || null;
  });

  // Memoized: EWF section offsets for hex navigation
  const ewfOffsets = createMemo(() => {
    const ewf = ewfInfo();
    if (!ewf) return null;
    const headerOffset = ewf.header_section_offset ?? 0xd;
    const volumeOffset = ewf.volume_section_offset ?? 0x59;
    const volumeDataStart = volumeOffset + 76;
    const headerDataStart = headerOffset + 76;
    return { headerOffset, volumeOffset, volumeDataStart, headerDataStart };
  });

  const categoryEntries = createMemo(() => [...groupedFields().entries()]);
  const regionCount = createMemo(() => props.metadata?.regions?.length ?? 0);
  const hasMetadata = createMemo(() => !!props.metadata);
  const hasFileInfo = createMemo(() => !!props.fileInfo);

  const handleRowClick = (offset: number | undefined | null, size?: number) => {
    if (offset !== undefined && offset !== null && props.onRegionClick) {
      props.onRegionClick(offset, size);
    }
  };

  const s = ROW_STYLES;

  return (
    <div class="flex flex-col h-full bg-bg text-xs overflow-auto">
      <Show when={!hasMetadata() && !hasFileInfo()}>
        <div class="flex flex-col items-center justify-center py-8 text-txt-muted">
          <HiOutlineClipboardDocument class="w-8 h-8 mb-2 opacity-40" />
          <span class="text-xs">No metadata</span>
        </div>
      </Show>

      {/* Format header */}
      <FormatHeader metadata={props.metadata} />

      {/* File info row */}
      <FileInfoSection
        fileInfo={props.fileInfo}
        rowBase={s.rowBase}
        rowGrid={s.rowGrid}
        keyStyle={s.keyStyle}
        valueStyle={s.valueStyle}
        offsetStyle={s.offsetStyle}
      />

      {/* Selected entry hex locations */}
      <HexLocationsSection
        selectedEntry={props.selectedEntry}
        isExpanded={isExpanded}
        toggleCategory={toggleCategory}
        handleRowClick={handleRowClick}
        categoryHeader={s.categoryHeader}
        rowBase={s.rowBase}
        rowGrid={s.rowGrid}
        rowClickable={s.rowClickable}
        keyStyle={s.keyStyle}
        valueStyle={s.valueStyle}
        offsetStyle={s.offsetStyle}
        offsetClickable={s.offsetClickable}
      />

      {/* Loading indicator for missing container info on E01 */}
      <Show
        when={
          !ewfInfo() &&
          props.fileInfo?.container_type &&
          isE01Container(props.fileInfo.container_type)
        }
      >
        <div class="border-b border-border">
          <div class={s.categoryHeader}>
            <span class="flex items-center gap-1 text-2xs leading-tight font-medium text-txt-tertiary">
              <HiOutlineArrowPath class="w-3 h-3 animate-spin" /> Loading container info...
            </span>
          </div>
        </div>
      </Show>

      {/* Container details (EWF) */}
      <Show when={ewfInfo()}>
        {(ewf) => {
          const offsets = ewfOffsets();
          return (
            <ContainerDetailsSection
              ewf={ewf}
              headerDataStart={offsets?.headerDataStart ?? 0}
              volumeDataStart={offsets?.volumeDataStart ?? 0}
              isExpanded={isExpanded}
              toggleCategory={toggleCategory}
              handleRowClick={handleRowClick}
              enhancedEwfInfo={enhancedEwfInfo}
              enhancedEwfLoading={enhancedEwfLoading}
              enhancedEwfError={enhancedEwfError}
              fetchEnhancedEwfInfo={fetchEnhancedEwfInfo}
              categoryHeader={s.categoryHeader}
              rowBase={s.rowBase}
              rowGrid={s.rowGrid}
              rowClickable={s.rowClickable}
              keyStyle={s.keyStyle}
              valueStyle={s.valueStyle}
              offsetStyle={s.offsetStyle}
              offsetClickable={s.offsetClickable}
            />
          );
        }}
      </Show>

      {/* Metadata categories */}
      <Show when={hasMetadata()}>
        <CategoryFieldsSection
          categoryEntries={categoryEntries()}
          isExpanded={isExpanded}
          toggleCategory={toggleCategory}
          handleRowClick={handleRowClick}
          categoryHeader={s.categoryHeader}
          rowBase={s.rowBase}
          rowGrid={s.rowGrid}
          rowClickable={s.rowClickable}
          keyStyle={s.keyStyle}
          valueStyle={s.valueStyle}
          offsetStyle={s.offsetStyle}
          offsetClickable={s.offsetClickable}
        />
      </Show>

      {/* Hex regions */}
      <HexRegionsSection
        metadata={props.metadata}
        regionCount={regionCount()}
        isExpanded={isExpanded}
        toggleCategory={toggleCategory}
        handleRowClick={handleRowClick}
        categoryHeader={s.categoryHeader}
        rowBase={s.rowBase}
        rowGrid={s.rowGrid}
        rowClickable={s.rowClickable}
        keyStyle={s.keyStyle}
        valueStyle={s.valueStyle}
        offsetStyle={s.offsetStyle}
        offsetClickable={s.offsetClickable}
      />
    </div>
  );
}
