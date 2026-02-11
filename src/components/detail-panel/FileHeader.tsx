// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component } from "solid-js";
import type { DiscoveredFile } from "../../types";
import { typeClass } from "../../utils";
import { getContainerTypeIcon } from "../tree";

interface FileHeaderProps {
  file: DiscoveredFile;
}

export const FileHeader: Component<FileHeaderProps> = (props) => {
  return (
    <div class="flex flex-col gap-1">
      <span class={`inline-flex items-center gap-1.5 text-xs px-2 py-0.5 rounded w-fit ${typeClass(props.file.container_type)}`}>
        {(() => {
          const IconComponent = getContainerTypeIcon(props.file.container_type);
          return <IconComponent class="w-3 h-3" />;
        })()} {props.file.container_type}
      </span>
      <h2 class="text-lg font-semibold text-txt truncate" title={props.file.filename}>
        {props.file.filename}
      </h2>
      <p class="text-xs text-txt-muted truncate" title={props.file.path}>
        {props.file.path}
      </p>
    </div>
  );
};
