// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, type Accessor, type Setter } from "solid-js";
import type { TemplateCategory } from "../../../hooks/useProjectTemplates";

interface CreateTemplateDialogProps {
  name: Accessor<string>;
  setName: Setter<string>;
  category: Accessor<TemplateCategory>;
  setCategory: Setter<TemplateCategory>;
  description: Accessor<string>;
  setDescription: Setter<string>;
  onClose: () => void;
  onCreate: () => void;
}

export const CreateTemplateDialog: Component<CreateTemplateDialogProps> = (
  props
) => {
  return (
    <div class="modal-overlay" onClick={props.onClose}>
      <div class="modal-content max-w-md" onClick={(e) => e.stopPropagation()}>
        <div class="modal-header">
          <h3 class="text-lg font-semibold text-txt">
            Create Template from Project
          </h3>
        </div>
        <div class="modal-body space-y-4">
          <div class="form-group">
            <label class="label">Template Name</label>
            <input
              type="text"
              value={props.name()}
              onInput={(e) => props.setName(e.currentTarget.value)}
              class="input"
              placeholder="Enter template name"
            />
          </div>
          <div class="form-group">
            <label class="label">Category</label>
            <select
              value={props.category()}
              onChange={(e) =>
                props.setCategory(e.currentTarget.value as TemplateCategory)
              }
              class="input"
            >
              <option value="Mobile">Mobile Forensics</option>
              <option value="Computer">Computer Forensics</option>
              <option value="Network">Network Forensics</option>
              <option value="Cloud">Cloud Forensics</option>
              <option value="IncidentResponse">Incident Response</option>
              <option value="Memory">Memory Analysis</option>
              <option value="Malware">Malware Analysis</option>
              <option value="EDiscovery">E-Discovery</option>
              <option value="General">General</option>
              <option value="Custom">Custom</option>
            </select>
          </div>
          <div class="form-group">
            <label class="label">Description (Optional)</label>
            <textarea
              value={props.description()}
              onInput={(e) => props.setDescription(e.currentTarget.value)}
              rows={3}
              class="textarea"
              placeholder="Describe this template..."
            />
          </div>
        </div>
        <div class="modal-footer justify-end">
          <button onClick={props.onClose} class="btn-sm">
            Cancel
          </button>
          <button
            onClick={props.onCreate}
            disabled={!props.name()}
            class="btn-sm-primary"
          >
            Create Template
          </button>
        </div>
      </div>
    </div>
  );
};
