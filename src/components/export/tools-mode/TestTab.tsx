// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { open } from "@tauri-apps/plugin-dialog";
import { HiOutlineInformationCircle } from "../../icons";
import type { Accessor } from "solid-js";

interface TestTabProps {
  archivePath: Accessor<string>;
  setArchivePath: (path: string) => void;
}

export function TestTab(props: TestTabProps) {
  return (
    <div class="space-y-3">
      <div class="info-card">
        <HiOutlineInformationCircle class="w-5 h-5 text-info" />
        <div>
          <div class="font-medium text-txt">Test Archive Integrity</div>
          <div class="text-xs text-txt-muted mt-1">
            Verify archive contents without extraction. Checks CRC and structure.
          </div>
        </div>
      </div>
      <div class="space-y-2">
        <label class="label">Archive File</label>
        <div class="flex gap-2">
          <input
            type="text"
            class="input-inline"
            value={props.archivePath()}
            onInput={(e) => props.setArchivePath(e.currentTarget.value)}
            placeholder="Select archive to test..."
          />
          <button class="btn-sm" onClick={async () => {
            const selected = await open({ directory: false, multiple: false, filters: [{ name: "Archives", extensions: ["7z", "zip"] }] });
            if (selected) props.setArchivePath(selected as string);
          }}>
            Browse
          </button>
        </div>
      </div>
    </div>
  );
}
