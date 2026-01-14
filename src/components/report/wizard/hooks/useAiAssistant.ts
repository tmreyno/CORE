// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useAiAssistant Hook - AI integration for report narrative generation
 */

import { createSignal, onMount } from "solid-js";
import {
  isAiAvailable,
  getAiProviders,
  checkOllamaConnection,
  generateAiNarrative,
  buildEvidenceContext,
  type AiProviderInfo,
  type NarrativeType,
} from "../../../../report/api";
import type { CaseInfo, Finding, EvidenceItem, HashValue } from "../../types";

// =============================================================================
// TYPES
// =============================================================================

export interface AiAssistantState {
  /** Whether AI is available */
  available: boolean;
  /** Available AI providers */
  providers: AiProviderInfo[];
  /** Currently selected provider */
  selectedProvider: string;
  /** Currently selected model */
  selectedModel: string;
  /** API key for providers that require it */
  apiKey: string;
  /** Whether Ollama is connected (for Ollama provider) */
  ollamaConnected: boolean;
  /** Currently generating narrative type */
  generating: NarrativeType | null;
  /** Last error message */
  error: string | null;
  /** Whether AI settings panel is visible */
  showSettings: boolean;
}

export interface AiAssistantActions {
  /** Set selected provider */
  setProvider: (provider: string) => void;
  /** Set selected model */
  setModel: (model: string) => void;
  /** Set API key */
  setApiKey: (key: string) => void;
  /** Toggle settings visibility */
  toggleSettings: () => void;
  /** Refresh Ollama connection status */
  refreshOllamaStatus: () => Promise<void>;
  /** Generate narrative for a section */
  generateNarrative: (
    type: NarrativeType,
    setter: (value: string) => void,
    context: AiContext
  ) => Promise<void>;
  /** Get current provider info */
  getCurrentProviderInfo: () => AiProviderInfo | undefined;
}

export interface AiContext {
  caseInfo: CaseInfo;
  findings: Finding[];
  evidenceItems: EvidenceItem[];
}

// =============================================================================
// HOOK
// =============================================================================

/**
 * Hook for managing AI assistant functionality in the report wizard.
 */
export function useAiAssistant(): [() => AiAssistantState, AiAssistantActions] {
  // State signals
  const [available, setAvailable] = createSignal(false);
  const [providers, setProviders] = createSignal<AiProviderInfo[]>([]);
  const [selectedProvider, setSelectedProvider] = createSignal("ollama");
  const [selectedModel, setSelectedModel] = createSignal("llama3.2");
  const [apiKey, setApiKey] = createSignal("");
  const [ollamaConnected, setOllamaConnected] = createSignal(false);
  const [generating, setGenerating] = createSignal<NarrativeType | null>(null);
  const [error, setError] = createSignal<string | null>(null);
  const [showSettings, setShowSettings] = createSignal(false);

  // Initialize AI availability
  onMount(async () => {
    try {
      const aiAvail = await isAiAvailable();
      setAvailable(aiAvail);

      if (aiAvail) {
        const aiProviders = await getAiProviders();
        setProviders(aiProviders);

        // Set defaults from first provider
        if (aiProviders.length > 0) {
          setSelectedProvider(aiProviders[0].id);
          setSelectedModel(aiProviders[0].default_model);
        }

        // Check Ollama connection
        const connected = await checkOllamaConnection();
        setOllamaConnected(connected);
      }
    } catch (e) {
      console.error("Failed to initialize AI:", e);
      setAvailable(false);
    }
  });

  // State accessor
  const state = (): AiAssistantState => ({
    available: available(),
    providers: providers(),
    selectedProvider: selectedProvider(),
    selectedModel: selectedModel(),
    apiKey: apiKey(),
    ollamaConnected: ollamaConnected(),
    generating: generating(),
    error: error(),
    showSettings: showSettings(),
  });

  // Actions
  const actions: AiAssistantActions = {
    setProvider: (provider: string) => {
      setSelectedProvider(provider);
      const info = providers().find((p) => p.id === provider);
      if (info) {
        setSelectedModel(info.default_model);
      }
      if (provider === "ollama") {
        actions.refreshOllamaStatus();
      }
    },

    setModel: setSelectedModel,

    setApiKey: setApiKey,

    toggleSettings: () => setShowSettings(!showSettings()),

    refreshOllamaStatus: async () => {
      try {
        const connected = await checkOllamaConnection();
        setOllamaConnected(connected);
      } catch (e) {
        setOllamaConnected(false);
      }
    },

    getCurrentProviderInfo: () => providers().find((p) => p.id === selectedProvider()),

    generateNarrative: async (
      type: NarrativeType,
      setter: (value: string) => void,
      context: AiContext
    ) => {
      if (!available() || generating()) return;

      // Validate provider state
      if (selectedProvider() === "ollama" && !ollamaConnected()) {
        setError("Ollama is not running. Please start Ollama first (run 'ollama serve' in terminal).");
        return;
      }

      if (selectedProvider() === "openai" && !apiKey()) {
        setError("OpenAI API key is required. Please enter your API key in AI settings.");
        setShowSettings(true);
        return;
      }

      setGenerating(type);
      setError(null);

      try {
        // Build context string
        const aiContext = buildContextString(context);

        const result = await generateAiNarrative(
          aiContext,
          type,
          selectedProvider(),
          selectedModel(),
          selectedProvider() === "openai" ? apiKey() : undefined
        );

        setter(result);
      } catch (e) {
        console.error("AI generation failed:", e);
        setError(String(e));
      } finally {
        setGenerating(null);
      }
    },
  };

  return [state, actions];
}

// =============================================================================
// HELPERS
// =============================================================================

/**
 * Builds a context string for AI narrative generation.
 */
function buildContextString(context: AiContext): string {
  // Build evidence context
  const evidenceContext = buildEvidenceContext(
    context.evidenceItems.map((item) => ({
      evidence_id: item.evidence_id,
      description: item.description,
      evidence_type: item.evidence_type,
      model: item.model,
      serial_number: item.serial_number,
      capacity: item.capacity,
      acquisition_hashes: item.acquisition_hashes.map((h: HashValue) => ({
        item: h.algorithm,
        algorithm: h.algorithm,
        value: h.value,
        verified: h.verified,
      })),
      image_info: item.acquisition_tool
        ? {
            format: "",
            file_names: [],
            total_size: 0,
            acquisition_tool: item.acquisition_tool,
          }
        : undefined,
      notes: item.notes,
    }))
  );

  // Add case context
  let result = `=== CASE INFORMATION ===\n`;
  result += `Case Number: ${context.caseInfo.case_number || "Not specified"}\n`;
  result += `Case Name: ${context.caseInfo.case_name || "Not specified"}\n`;
  result += `Agency: ${context.caseInfo.agency || "Not specified"}\n`;
  result += `Investigation Type: ${context.caseInfo.investigation_type || "Not specified"}\n`;
  result += `\n${evidenceContext}`;

  // Add existing findings summary
  if (context.findings.length > 0) {
    result += `\n=== FINDINGS ===\n`;
    for (const finding of context.findings) {
      result += `- ${finding.title} (${finding.severity}): ${finding.description}\n`;
    }
  }

  return result;
}
