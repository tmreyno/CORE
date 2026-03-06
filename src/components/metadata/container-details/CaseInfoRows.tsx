// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { HeaderFieldRow } from "./DetailRow";
import type { ContainerDetailsSectionProps } from "./types";

type CaseInfoRowsProps = Pick<
  ContainerDetailsSectionProps,
  | "ewf"
  | "headerDataStart"
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

export function CaseInfoRows(props: CaseInfoRowsProps) {
  const styles = {
    categoryHeader: props.categoryHeader,
    rowBase: props.rowBase,
    rowGrid: props.rowGrid,
    rowClickable: props.rowClickable,
    keyStyle: props.keyStyle,
    valueStyle: props.valueStyle,
    offsetStyle: props.offsetStyle,
    offsetClickable: props.offsetClickable,
  };

  return (
    <>
      <HeaderFieldRow
        label="CASE #"
        value={props.ewf().case_number ?? null}
        headerDataStart={props.headerDataStart}
        onRowClick={props.handleRowClick}
        {...styles}
      />
      <HeaderFieldRow
        label="EVIDENCE #"
        value={props.ewf().evidence_number ?? null}
        headerDataStart={props.headerDataStart}
        onRowClick={props.handleRowClick}
        {...styles}
      />
      <HeaderFieldRow
        label="EXAMINER"
        value={props.ewf().examiner_name ?? null}
        headerDataStart={props.headerDataStart}
        onRowClick={props.handleRowClick}
        {...styles}
      />
      <HeaderFieldRow
        label="ACQUIRED"
        value={props.ewf().acquiry_date ?? null}
        headerDataStart={props.headerDataStart}
        onRowClick={props.handleRowClick}
        {...styles}
      />
      <HeaderFieldRow
        label="SYSTEM DATE"
        value={props.ewf().system_date ?? null}
        headerDataStart={props.headerDataStart}
        onRowClick={props.handleRowClick}
        {...styles}
      />
      <HeaderFieldRow
        label="DESCRIPTION"
        value={props.ewf().description ?? null}
        headerDataStart={props.headerDataStart}
        onRowClick={props.handleRowClick}
        {...styles}
      />
      <HeaderFieldRow
        label="NOTES"
        value={props.ewf().notes ?? null}
        headerDataStart={props.headerDataStart}
        onRowClick={props.handleRowClick}
        {...styles}
      />
      <HeaderFieldRow
        label="MODEL"
        value={props.ewf().model ?? null}
        headerDataStart={props.headerDataStart}
        onRowClick={props.handleRowClick}
        {...styles}
      />
      <HeaderFieldRow
        label="SERIAL #"
        value={props.ewf().serial_number ?? null}
        headerDataStart={props.headerDataStart}
        onRowClick={props.handleRowClick}
        {...styles}
      />
    </>
  );
}
