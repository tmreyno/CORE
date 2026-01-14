// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * FindingsStep - Fourth wizard step for documenting findings
 */

import { For, Show } from "solid-js";
import {
  HiOutlineCpuChip,
  HiOutlineArrowPath,
  HiOutlineExclamationTriangle,
} from "../../../icons";
import { SEVERITIES } from "../../constants";
import type { Severity } from "../../types";
import { useWizard } from "../WizardContext";

export function FindingsStep() {
  const ctx = useWizard();
  const aiState = ctx.aiState;

  // Generate AI narrative wrapper
  const handleGenerateNarrative = async (
    type: "executive_summary" | "methodology" | "conclusion",
    setter: (value: string) => void
  ) => {
    await ctx.aiActions.generateNarrative(type, setter, {
      caseInfo: ctx.caseInfo(),
      findings: ctx.findings(),
      evidenceItems: ctx.buildReport().evidence_items,
    });
  };

  return (
    <div class="space-y-5">
      {/* AI Settings Panel */}
      <Show when={aiState().available}>
        <AiSettingsPanel />
      </Show>

      <div class="flex items-center justify-between">
        <h3 class="text-lg font-medium">Findings</h3>
        <button class="btn-action-primary" onClick={ctx.addFinding}>
          + Add Finding
        </button>
      </div>

      <div class="grid grid-cols-2 gap-4">
        <div>
          <div class="flex items-center justify-between mb-1">
            <label class="text-sm font-medium">Executive Summary</label>
            <Show when={aiState().available}>
              <button
                class="text-xs text-accent hover:underline disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1"
                onClick={() => handleGenerateNarrative("executive_summary", ctx.setExecutiveSummary)}
                disabled={!!aiState().generating}
              >
                {aiState().generating === "executive_summary"
                  ? <><HiOutlineArrowPath class="w-3 h-3 animate-spin" /> Generating...</>
                  : <><HiOutlineCpuChip class="w-3 h-3" /> Generate with AI</>
                }
              </button>
            </Show>
          </div>
          <textarea
            class="textarea h-32"
            value={ctx.executiveSummary()}
            onInput={(e) => ctx.setExecutiveSummary(e.currentTarget.value)}
            placeholder="Brief summary for non-technical readers..."
          />
        </div>

        <div>
          <label class="label">Scope</label>
          <textarea
            class="textarea h-32"
            value={ctx.scope()}
            onInput={(e) => ctx.setScope(e.currentTarget.value)}
            placeholder="Scope of the examination..."
          />
        </div>

        <div>
          <div class="flex items-center justify-between mb-1">
            <label class="label">Methodology</label>
            <Show when={aiState().available}>
              <button
                class="text-xs text-accent hover:underline disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1"
                onClick={() => handleGenerateNarrative("methodology", ctx.setMethodology)}
                disabled={!!aiState().generating}
              >
                {aiState().generating === "methodology"
                  ? <><HiOutlineArrowPath class="w-3 h-3 animate-spin" /> Generating...</>
                  : <><HiOutlineCpuChip class="w-3 h-3" /> Generate with AI</>
                }
              </button>
            </Show>
          </div>
          <textarea
            class="textarea h-32"
            value={ctx.methodology()}
            onInput={(e) => ctx.setMethodology(e.currentTarget.value)}
            placeholder="Examination methodology employed..."
          />
        </div>

        <div>
          <div class="flex items-center justify-between mb-1">
            <label class="label">Conclusions</label>
            <Show when={aiState().available}>
              <button
                class="text-xs text-accent hover:underline disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1"
                onClick={() => handleGenerateNarrative("conclusion", ctx.setConclusions)}
                disabled={!!aiState().generating}
              >
                {aiState().generating === "conclusion"
                  ? <><HiOutlineArrowPath class="w-3 h-3 animate-spin" /> Generating...</>
                  : <><HiOutlineCpuChip class="w-3 h-3" /> Generate with AI</>
                }
              </button>
            </Show>
          </div>
          <textarea
            class="textarea h-32"
            value={ctx.conclusions()}
            onInput={(e) => ctx.setConclusions(e.currentTarget.value)}
            placeholder="Final conclusions..."
          />
        </div>
      </div>

      <Show when={ctx.findings().length === 0}>
        <div class="text-center py-6 text-txt-muted border border-dashed border-border rounded">
          <p>No findings added yet.</p>
          <p class="text-sm">Click "Add Finding" to document discoveries.</p>
        </div>
      </Show>

      <For each={ctx.findings()}>
        {(finding, index) => (
          <div class="border border-border rounded p-3 space-y-3">
            <div class="flex items-center justify-between">
              <span class="text-sm font-mono text-txt-muted">{finding.id}</span>
              <button
                class="text-error hover:text-error/80 text-sm"
                onClick={() => ctx.removeFinding(index())}
              >
                Remove
              </button>
            </div>

            <div class="grid grid-cols-3 gap-3">
              <div class="col-span-2">
                <input
                  type="text"
                  class="w-full px-2 py-1.5 bg-bg border border-border rounded text-sm focus:outline-none focus:border-accent"
                  value={finding.title}
                  onInput={(e) => ctx.updateFinding(index(), { title: e.currentTarget.value })}
                  placeholder="Finding title..."
                />
              </div>

              <select
                class="px-2 py-1.5 bg-bg border border-border rounded text-sm focus:outline-none focus:border-accent"
                value={finding.severity}
                onChange={(e) => ctx.updateFinding(index(), { severity: e.currentTarget.value as Severity })}
              >
                <For each={SEVERITIES}>
                  {(s) => <option value={s.value}>{s.label}</option>}
                </For>
              </select>
            </div>

            <textarea
              class="w-full px-2 py-1.5 bg-bg border border-border rounded text-sm focus:outline-none focus:border-accent h-20 resize-none"
              value={finding.description}
              onInput={(e) => ctx.updateFinding(index(), { description: e.currentTarget.value })}
              placeholder="Detailed description of the finding..."
            />
          </div>
        )}
      </For>
    </div>
  );
}

/**
 * AI Settings Panel sub-component
 */
function AiSettingsPanel() {
  const ctx = useWizard();
  const aiState = ctx.aiState;
  const aiActions = ctx.aiActions;

  return (
    <div class="border border-accent/30 rounded-xl p-4 bg-accent/5">
      <div class="flex items-center justify-between mb-3">
        <div class="flex items-center gap-2">
          <div class="w-8 h-8 rounded-lg bg-accent/20 flex items-center justify-center">
            <HiOutlineCpuChip class="w-5 h-5 text-accent" />
          </div>
          <div>
            <span class="font-medium text-sm">AI Report Assistant</span>
            <Show when={aiState().selectedProvider === "ollama"}>
              <span class={`ml-2 text-xs px-1.5 py-0.5 rounded ${aiState().ollamaConnected ? 'bg-success/20 text-success' : 'bg-error/20 text-error'}`}>
                {aiState().ollamaConnected ? "Connected" : "Disconnected"}
              </span>
            </Show>
          </div>
        </div>
        <button
          class="text-sm text-accent hover:underline"
          onClick={aiActions.toggleSettings}
        >
          {aiState().showSettings ? "Hide Settings" : "Settings"}
        </button>
      </div>

      <Show when={aiState().showSettings}>
        <div class="grid grid-cols-3 gap-3 mt-3 pt-3 border-t border-border">
          <div>
            <label class="block text-xs text-txt-muted mb-1">Provider</label>
            <select
              class="w-full px-2 py-1.5 bg-bg border border-border rounded text-sm"
              value={aiState().selectedProvider}
              onChange={(e) => aiActions.setProvider(e.currentTarget.value)}
            >
              <For each={aiState().providers}>
                {(p) => <option value={p.id}>{p.name}</option>}
              </For>
            </select>
          </div>

          <div>
            <label class="block text-xs text-txt-muted mb-1">Model</label>
            <select
              class="w-full px-2 py-1.5 bg-bg border border-border rounded text-sm"
              value={aiState().selectedModel}
              onChange={(e) => aiActions.setModel(e.currentTarget.value)}
            >
              <For each={aiActions.getCurrentProviderInfo()?.available_models || []}>
                {(m) => <option value={m}>{m}</option>}
              </For>
            </select>
          </div>

          <Show when={aiActions.getCurrentProviderInfo()?.requires_api_key}>
            <div>
              <label class="block text-xs text-txt-muted mb-1">API Key</label>
              <input
                type="password"
                class="w-full px-2 py-1.5 bg-bg border border-border rounded text-sm"
                value={aiState().apiKey}
                onInput={(e) => aiActions.setApiKey(e.currentTarget.value)}
                placeholder="sk-..."
              />
            </div>
          </Show>

          <Show when={aiState().selectedProvider === "ollama" && !aiState().ollamaConnected}>
            <div class="col-span-3 text-sm text-error flex items-center gap-2">
              <HiOutlineExclamationTriangle class="w-4 h-4" />
              <span>Ollama not running.</span>
              <button class="text-accent hover:underline" onClick={aiActions.refreshOllamaStatus}>
                Retry
              </button>
              <span class="text-txt-muted">| Run: <code class="bg-bg px-1 rounded">ollama serve</code></span>
            </div>
          </Show>
        </div>
      </Show>

      <Show when={aiState().error}>
        <div class="mt-2 text-sm text-error bg-error/10 rounded p-2">
          {aiState().error}
        </div>
      </Show>
    </div>
  );
}
