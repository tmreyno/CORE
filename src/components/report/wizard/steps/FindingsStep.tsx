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
  HiOutlineAdjustmentsHorizontal,
  HiOutlineEye,
} from "../../../icons";
import { SEVERITIES } from "../../constants";
import type { Severity } from "../../types";
import { useWizard } from "../WizardContext";
import type { SectionVisibility } from "../types";

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
    <div class="space-y-3">
      {/* Section Visibility Toggles */}
      <SectionVisibilityPanel />

      {/* AI Settings Panel */}
      <Show when={aiState().available}>
        <AiSettingsPanel />
      </Show>

      <div class="flex items-center justify-between">
        <h3 class="text-sm font-medium">Findings</h3>
        <button class="btn-sm flex items-center gap-1" onClick={ctx.addFinding}>
          + Add Finding
        </button>
      </div>

      <div class="grid grid-cols-2 gap-3">
        <div>
          <div class="flex items-center justify-between mb-1">
            <label class="text-xs font-medium">Executive Summary</label>
            <Show when={aiState().available}>
              <button
                class="text-[11px] text-accent hover:underline disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1"
                onClick={() => handleGenerateNarrative("executive_summary", ctx.setExecutiveSummary)}
                disabled={!!aiState().generating}
              >
                {aiState().generating === "executive_summary"
                  ? <><HiOutlineArrowPath class="w-3 h-3 animate-spin" /> Generating...</>
                  : <><HiOutlineCpuChip class="w-3 h-3" /> AI Generate</>
                }
              </button>
            </Show>
          </div>
          <textarea
            class="textarea text-sm h-24"
            value={ctx.executiveSummary()}
            onInput={(e) => ctx.setExecutiveSummary(e.currentTarget.value)}
            placeholder="Brief summary for non-technical readers..."
          />
        </div>

        <div>
          <label class="label">Scope</label>
          <textarea
            class="textarea text-sm h-24"
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
                class="text-[11px] text-accent hover:underline disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1"
                onClick={() => handleGenerateNarrative("methodology", ctx.setMethodology)}
                disabled={!!aiState().generating}
              >
                {aiState().generating === "methodology"
                  ? <><HiOutlineArrowPath class="w-3 h-3 animate-spin" /> Generating...</>
                  : <><HiOutlineCpuChip class="w-3 h-3" /> AI Generate</>
                }
              </button>
            </Show>
          </div>
          <textarea
            class="textarea text-sm h-24"
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
                class="text-[11px] text-accent hover:underline disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1"
                onClick={() => handleGenerateNarrative("conclusion", ctx.setConclusions)}
                disabled={!!aiState().generating}
              >
                {aiState().generating === "conclusion"
                  ? <><HiOutlineArrowPath class="w-3 h-3 animate-spin" /> Generating...</>
                  : <><HiOutlineCpuChip class="w-3 h-3" /> AI Generate</>
                }
              </button>
            </Show>
          </div>
          <textarea
            class="textarea text-sm h-24"
            value={ctx.conclusions()}
            onInput={(e) => ctx.setConclusions(e.currentTarget.value)}
            placeholder="Final conclusions..."
          />
        </div>
      </div>

      <Show when={ctx.findings().length === 0}>
        <div class="text-center py-4 text-txt-muted border border-dashed border-border rounded-lg text-xs">
          <p>No findings added yet. Click "Add Finding" to document discoveries.</p>
        </div>
      </Show>

      <For each={ctx.findings()}>
        {(finding, index) => (
          <div class="border border-border rounded-lg p-2.5 space-y-2">
            <div class="flex items-center justify-between">
              <span class="text-xs font-mono text-txt-muted">{finding.id}</span>
              <button
                class="text-error hover:text-error/80 text-xs"
                onClick={() => ctx.removeFinding(index())}
              >
                Remove
              </button>
            </div>

            <div class="grid grid-cols-3 gap-2">
              <div class="col-span-2">
                <input
                  type="text"
                  class="input-sm"
                  value={finding.title}
                  onInput={(e) => ctx.updateFinding(index(), { title: e.currentTarget.value })}
                  placeholder="Finding title..."
                />
              </div>

              <select
                class="input-sm"
                value={finding.severity}
                onChange={(e) => ctx.updateFinding(index(), { severity: e.currentTarget.value as Severity })}
              >
                <For each={SEVERITIES}>
                  {(s) => <option value={s.value}>{s.label}</option>}
                </For>
              </select>
            </div>

            <textarea
              class="textarea text-sm h-16"
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
    <div class="border border-accent/30 rounded-lg p-3 bg-accent/5">
      <div class="flex items-center justify-between mb-2">
        <div class="flex items-center gap-2">
          <div class="w-6 h-6 rounded-md bg-accent/20 flex items-center justify-center">
            <HiOutlineCpuChip class="w-3.5 h-3.5 text-accent" />
          </div>
          <div>
            <span class="font-medium text-xs">AI Report Assistant</span>
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
        <div class="grid grid-cols-3 gap-2 mt-2 pt-2 border-t border-border">
          <div>
            <label class="block text-[11px] text-txt-muted mb-0.5">Provider</label>
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
            <label class="block text-[11px] text-txt-muted mb-0.5">Model</label>
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
              <label class="block text-[11px] text-txt-muted mb-0.5">API Key</label>
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

/**
 * Section Visibility toggles - allows users to include/exclude report sections
 */
function SectionVisibilityPanel() {
  const ctx = useWizard();

  const sections: { key: keyof SectionVisibility; label: string; description: string }[] = [
    { key: "executiveSummary", label: "Executive Summary", description: "Non-technical overview for stakeholders" },
    { key: "scope", label: "Scope", description: "Scope and limitations of the examination" },
    { key: "methodology", label: "Methodology", description: "Tools and techniques employed" },
    { key: "chainOfCustody", label: "Chain of Custody", description: "Evidence handling documentation" },
    { key: "timeline", label: "Timeline", description: "Chronological event analysis" },
    { key: "conclusions", label: "Conclusions", description: "Final findings and opinions" },
    { key: "appendices", label: "Appendices", description: "Supporting data and hash tables" },
  ];

  const toggleSection = (key: keyof SectionVisibility) => {
    ctx.setEnabledSections((prev) => ({ ...prev, [key]: !prev[key] }));
  };

  const enabledCount = () =>
    Object.values(ctx.enabledSections()).filter(Boolean).length;

  return (
    <div class="p-2.5 bg-surface/30 rounded-lg border border-border/30">
      <div class="flex items-center justify-between mb-2">
        <div class="flex items-center gap-1.5">
          <HiOutlineAdjustmentsHorizontal class="w-3.5 h-3.5 text-accent/80" />
          <h4 class="text-xs font-medium">Report Sections</h4>
          <span class="text-[10px] px-1.5 py-0.5 bg-accent/10 text-accent rounded-full font-medium">
            {enabledCount()} of {sections.length}
          </span>
        </div>
        <div class="flex items-center gap-1">
          <HiOutlineEye class="w-3 h-3 text-txt/40" />
          <span class="text-[10px] text-txt/40">Toggle sections</span>
        </div>
      </div>
      <div class="grid grid-cols-2 lg:grid-cols-4 xl:grid-cols-4 gap-1.5">
        <For each={sections}>
          {(section) => {
            const enabled = () => ctx.enabledSections()[section.key];
            return (
              <button
                type="button"
                class={`flex items-center gap-2 p-1.5 rounded-md border text-left transition-all duration-150 ${
                  enabled()
                    ? "border-accent/40 bg-accent/5 hover:bg-accent/10"
                    : "border-border/30 bg-bg/30 hover:bg-bg/50 opacity-60"
                }`}
                onClick={() => toggleSection(section.key)}
                title={section.description}
              >
                <div class={`w-3.5 h-3.5 rounded border flex items-center justify-center shrink-0 transition-colors ${
                  enabled() ? "bg-accent border-accent" : "border-border/50"
                }`}>
                  <Show when={enabled()}>
                    <span class="text-white text-[8px] font-bold">✓</span>
                  </Show>
                </div>
                <span class={`text-[11px] font-medium ${enabled() ? "text-txt" : "text-txt/50"}`}>
                  {section.label}
                </span>
              </button>
            );
          }}
        </For>
      </div>
    </div>
  );
}
