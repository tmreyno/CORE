// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show, type Accessor } from "solid-js";
import { HiOutlineComputerDesktop } from "../icons";
import type { DiscoveredFile, ContainerInfo } from "../../types";
import type { DriveInfo } from "../../api/drives";
import type { SystemStats } from "../../hooks";
import { EvidenceCollectionPanel } from "../EvidenceCollectionPanel";
import SystemInfoPanel from "./SystemInfoPanel";
import AcquireProcessShell from "./AcquireProcessShell";

export interface AcquireCollectionViewProps {
  onBack: () => void;
  caseNumber?: Accessor<string | undefined>;
  projectName?: Accessor<string | undefined>;
  examinerName?: Accessor<string | undefined>;
  collectionId?: Accessor<string | undefined>;
  readOnly?: Accessor<boolean>;
  discoveredFiles?: Accessor<DiscoveredFile[]>;
  fileInfoMap?: Accessor<Map<string, ContainerInfo>>;
  systemDrivesData: Accessor<DriveInfo[]>;
  systemStatsData: Accessor<SystemStats | null>;
  evidenceItemFolder: Accessor<string>;
  showSystemPanel: Accessor<boolean>;
  setShowSystemPanel: (value: boolean | ((prev: boolean) => boolean)) => void;
}

const AcquireCollectionView: Component<AcquireCollectionViewProps> = (props) => {
  return (
    <AcquireProcessShell
      title="Evidence Collection"
      onBack={props.onBack}
      headerActions={(
        <button
          class="icon-btn-sm"
          classList={{ "text-accent": props.showSystemPanel(), "text-txt-muted": !props.showSystemPanel() }}
          onClick={() => props.setShowSystemPanel((prev) => !prev)}
          title={props.showSystemPanel() ? "Hide System Info" : "Show System Info"}
        >
          <HiOutlineComputerDesktop class="w-icon-sm h-icon-sm" />
        </button>
      )}
    >
      <div class="flex flex-1 min-h-0 overflow-hidden">
        <div class="flex-1 min-h-0 overflow-auto">
          <EvidenceCollectionPanel
            caseNumber={props.caseNumber?.()}
            projectName={props.projectName?.()}
            examinerName={props.examinerName?.()}
            collectionId={props.collectionId?.()}
            readOnly={props.readOnly?.()}
            discoveredFiles={props.discoveredFiles?.() ?? []}
            fileInfoMap={props.fileInfoMap?.() ?? new Map()}
            systemDrives={props.systemDrivesData()}
            systemStats={props.systemStatsData?.()}
            evidenceItemFolder={props.evidenceItemFolder()}
            onClose={props.onBack}
          />
        </div>

        <Show when={props.showSystemPanel()}>
          <div class="w-72 shrink-0 border-l border-border overflow-hidden">
            <SystemInfoPanel
              systemStats={props.systemStatsData()}
              drives={props.systemDrivesData()}
            />
          </div>
        </Show>
      </div>
    </AcquireProcessShell>
  );
};

export default AcquireCollectionView;