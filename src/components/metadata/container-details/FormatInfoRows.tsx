// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { formatBytes } from "../../../utils";
import { DetailRow } from "./DetailRow";
import type { ContainerDetailsSectionProps } from "./types";

type FormatInfoRowsProps = Pick<
  ContainerDetailsSectionProps,
  | "ewf"
  | "volumeDataStart"
  | "handleRowClick"
  | "categoryHeader"
  | "rowBase"
  | "rowGrid"
  | "rowClickable"
  | "keyStyle"
  | "valueStyle"
  | "offsetStyle"
  | "offsetClickable"
>;

export function FormatInfoRows(props: FormatInfoRowsProps) {
  return (
    <>
      <DetailRow
        label="FORMAT"
        value={props.ewf().format_version}
        offset={0x0}
        offsetSize={8}
        clickable
        onRowClick={props.handleRowClick}
        {...styleProps(props)}
      />
      <DetailRow
        label="SEGMENTS"
        value={props.ewf().segment_count}
        offset={0x9}
        offsetSize={2}
        clickable
        onRowClick={props.handleRowClick}
        {...styleProps(props)}
      />
      <DetailRow
        label="TOTAL SIZE"
        value={formatBytes(props.ewf().total_size)}
        {...styleProps(props)}
      />
      <DetailRow
        label="COMPRESSION"
        value={props.ewf().compression || "Unknown"}
        offset={props.volumeDataStart + 0x38}
        offsetSize={1}
        clickable
        onRowClick={props.handleRowClick}
        {...styleProps(props)}
      />
      <DetailRow
        label="BYTES/SECTOR"
        value={props.ewf().bytes_per_sector}
        offset={props.volumeDataStart + 0x0c}
        offsetSize={4}
        clickable
        onRowClick={props.handleRowClick}
        {...styleProps(props)}
      />
      <DetailRow
        label="SECTORS/CHUNK"
        value={props.ewf().sectors_per_chunk}
        offset={props.volumeDataStart + 0x08}
        offsetSize={4}
        clickable
        onRowClick={props.handleRowClick}
        {...styleProps(props)}
      />
    </>
  );
}

/** Extract RowStyles from a props superset */
function styleProps(p: FormatInfoRowsProps) {
  return {
    categoryHeader: p.categoryHeader,
    rowBase: p.rowBase,
    rowGrid: p.rowGrid,
    rowClickable: p.rowClickable,
    keyStyle: p.keyStyle,
    valueStyle: p.valueStyle,
    offsetStyle: p.offsetStyle,
    offsetClickable: p.offsetClickable,
  };
}
