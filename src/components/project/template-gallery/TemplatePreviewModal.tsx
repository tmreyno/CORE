// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show } from "solid-js";
import { HiOutlineX } from "../../icons";
import type { ProjectTemplate } from "../../../hooks/useProjectTemplates";

interface TemplatePreviewModalProps {
  template: ProjectTemplate;
  onClose: () => void;
  onApply: (templateId: string) => void;
}

export const TemplatePreviewModal: Component<TemplatePreviewModalProps> = (
  props
) => {
  return (
    <>
      <div
        class="fixed inset-0 bg-black/50 z-modal-backdrop"
        onClick={props.onClose}
      />
      <div class="fixed inset-0 z-modal flex items-center justify-center p-4">
        <div class="bg-bg-panel rounded-lg border border-border w-full max-w-4xl max-h-[80vh] flex flex-col">
          <div class="flex items-center justify-between p-4 border-b border-border">
            <h3 class="text-lg font-semibold text-txt">{props.template.name}</h3>
            <button
              onClick={props.onClose}
              class="p-2 hover:bg-bg-hover rounded-md text-txt-secondary hover:text-txt"
            >
              <HiOutlineX class="w-icon-base h-icon-base" />
            </button>
          </div>
          <div class="flex-1 overflow-auto p-4">
            <div class="space-y-4">
              <div>
                <h4 class="text-sm font-medium text-txt mb-2">Description</h4>
                <p class="text-sm text-txt-secondary">
                  {props.template.description}
                </p>
              </div>
              <Show when={props.template.bookmarks.length > 0}>
                <div>
                  <h4 class="text-sm font-medium text-txt mb-2">
                    Bookmarks ({props.template.bookmarks.length})
                  </h4>
                  <div class="text-sm text-txt-muted">
                    Template includes {props.template.bookmarks.length} bookmarks
                  </div>
                </div>
              </Show>
              <Show when={props.template.notes.length > 0}>
                <div>
                  <h4 class="text-sm font-medium text-txt mb-2">
                    Notes ({props.template.notes.length})
                  </h4>
                  <div class="text-sm text-txt-muted">
                    Template includes {props.template.notes.length} notes
                  </div>
                </div>
              </Show>
            </div>
          </div>
          <div class="modal-footer justify-end">
            <button onClick={props.onClose} class="btn-sm">
              Close
            </button>
            <button
              onClick={() => {
                props.onApply(props.template.id);
                props.onClose();
              }}
              class="btn-sm-primary"
            >
              Apply Template
            </button>
          </div>
        </div>
      </div>
    </>
  );
};
