// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show, type Component } from "solid-js";
import type { ToolsModeProps } from "./types";
import { ToolsTabSelector } from "./ToolsTabSelector";
import { TestTab } from "./TestTab";
import { RepairTab } from "./RepairTab";
import { ValidateTab } from "./ValidateTab";
import { ExtractTab } from "./ExtractTab";
import { CompressTab } from "./CompressTab";
import { DecompressTab } from "./DecompressTab";

export const ToolsModeComponent: Component<ToolsModeProps> = (props) => {
  return (
    <div class="space-y-3">
      <ToolsTabSelector active={props.toolsTab} onSelect={props.setToolsTab} />

      <Show when={props.toolsTab() === "test"}>
        <TestTab archivePath={props.testArchivePath} setArchivePath={props.setTestArchivePath} />
      </Show>

      <Show when={props.toolsTab() === "repair"}>
        <RepairTab
          corruptedPath={props.repairCorruptedPath}
          setCorruptedPath={props.setRepairCorruptedPath}
          outputPath={props.repairOutputPath}
          setOutputPath={props.setRepairOutputPath}
        />
      </Show>

      <Show when={props.toolsTab() === "validate"}>
        <ValidateTab archivePath={props.validateArchivePath} setArchivePath={props.setValidateArchivePath} />
      </Show>

      <Show when={props.toolsTab() === "extract"}>
        <ExtractTab
          firstVolume={props.extractFirstVolume}
          setFirstVolume={props.setExtractFirstVolume}
          outputDir={props.extractOutputDir}
          setOutputDir={props.setExtractOutputDir}
        />
      </Show>

      <Show when={props.toolsTab() === "compress"}>
        <CompressTab
          algorithm={props.lzmaAlgorithm}
          setAlgorithm={props.setLzmaAlgorithm}
          level={props.lzmaLevel}
          setLevel={props.setLzmaLevel}
          inputPath={props.lzmaInputPath}
          setInputPath={props.setLzmaInputPath}
          outputPath={props.lzmaOutputPath}
          setOutputPath={props.setLzmaOutputPath}
        />
      </Show>

      <Show when={props.toolsTab() === "decompress"}>
        <DecompressTab
          inputPath={props.lzmaDecompressInput}
          setInputPath={props.setLzmaDecompressInput}
          outputPath={props.lzmaDecompressOutput}
          setOutputPath={props.setLzmaDecompressOutput}
        />
      </Show>
    </div>
  );
};
