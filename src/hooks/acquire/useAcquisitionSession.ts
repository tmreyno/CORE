// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useAcquisitionSession — Manages a lightweight .acquisition.json session file
 * for the Acquire edition. Replaces the .cffx + .ffxdb project system.
 *
 * Lifecycle: create() / load() / close()
 * Mutations: addAcquisition(), updateAcquisition(), addCollection(), etc.
 * Persistence: auto-save on every mutation via write_text_file.
 */

import { createSignal, type Accessor } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { logger } from "../../utils/logger";
import { addRecentProject } from "../../components/preferences";
import { getBasename } from "../../utils/pathUtils";
import { isTauri } from "../../utils/platform";
import type {
  AcquisitionSession,
  SessionAcquisitionRecord,
  SessionActivityEntry,
} from "../../types/acquisitionSession";
import {
  createEmptySession,
  ACQUISITION_SESSION_EXTENSION,
} from "../../types/acquisitionSession";

const log = logger.scope("AcqSession");

// =============================================================================
// Hook interface
// =============================================================================

interface AcquisitionSessionManager {
  // State
  session: Accessor<AcquisitionSession | null>;
  sessionPath: Accessor<string>;
  hasSession: Accessor<boolean>;

  // Derived
  caseNumber: Accessor<string>;
  examiner: Accessor<string>;
  projectName: Accessor<string>;
  outputFolder: Accessor<string>;
  evidenceFolder: Accessor<string>;

  // Lifecycle
  create: (opts: CreateSessionOpts) => Promise<void>;
  load: (path: string) => Promise<void>;
  loadParsed: (parsed: AcquisitionSession, path: string) => void;
  close: () => void;

  // Mutations — all auto-save
  addAcquisition: (record: SessionAcquisitionRecord) => void;
  updateAcquisition: (id: string, updates: Partial<SessionAcquisitionRecord>) => void;
  addActivity: (entry: Omit<SessionActivityEntry, "id" | "timestamp">) => void;
}

export interface CreateSessionOpts {
  caseNumber: string;
  examiner: string;
  outputFolder: string;
  organization?: string;
  caseName?: string;
}

// =============================================================================
// Hook
// =============================================================================

export function useAcquisitionSession(): AcquisitionSessionManager {
  const [session, setSession] = createSignal<AcquisitionSession | null>(null);
  const [sessionPath, setSessionPath] = createSignal("");

  // Derived accessors
  const hasSession = () => session() !== null;
  const caseNumber = () => session()?.caseNumber ?? "";
  const examiner = () => session()?.examiner ?? "";
  const outputFolder = () => session()?.outputFolder ?? "";
  const evidenceFolder = () => session()?.evidenceFolder ?? "";

  const projectName = () => {
    const s = session();
    if (!s) return "";
    if (s.caseName) return s.caseName;
    if (s.caseNumber) return `Case ${s.caseNumber}`;
    return "Acquisition Session";
  };

  // ─── Persistence ──────────────────────────────────────────────────────

  /** Debounced save — coalesces rapid mutations */
  let saveTimer: ReturnType<typeof setTimeout> | null = null;

  function scheduleSave(): void {
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(() => {
      saveTimer = null;
      const s = session();
      const p = sessionPath();
      if (!s || !p) return;
      const updated = { ...s, modifiedAt: new Date().toISOString() };
      setSession(updated);
      if (!isTauri) return;
      const json = JSON.stringify(updated, null, 2);
      invoke("write_text_file", { path: p, content: json }).catch((err) => {
        log.warn("Session save failed:", err);
      });
    }, 300);
  }

  // ─── Lifecycle ────────────────────────────────────────────────────────

  async function create(opts: CreateSessionOpts): Promise<void> {
    const t0 = performance.now();
    log.info(`create() called for case=${opts.caseNumber}`);

    const fileName = buildSessionFileName(opts.caseNumber);
    const filePath = `${opts.outputFolder}/${fileName}`;

    const newSession = createEmptySession({
      caseNumber: opts.caseNumber,
      examiner: opts.examiner,
      outputFolder: opts.outputFolder,
      sessionFilePath: filePath,
      organization: opts.organization,
      caseName: opts.caseName,
    });

    if (isTauri) {
      // Gather system info at session creation
      try {
        log.info(`invoke get_hostname (+${(performance.now() - t0).toFixed(0)}ms)`);
        const hostname = await invoke<string>("get_hostname").catch(() => "");
        log.info(`get_hostname done (+${(performance.now() - t0).toFixed(0)}ms)`);
        const username = await invoke<string>("get_current_username").catch(() => "");
        log.info(`get_current_username done (+${(performance.now() - t0).toFixed(0)}ms)`);
        newSession.system.hostname = hostname;
        newSession.system.username = username;
      } catch (e) {
        log.warn("Failed to gather system info:", e);
      }
    }

    log.info(`setSession() (+${(performance.now() - t0).toFixed(0)}ms)`);
    setSession(newSession);
    setSessionPath(filePath);
    log.info(`signals updated (+${(performance.now() - t0).toFixed(0)}ms)`);

    if (isTauri) {
      // Write initial file
      const json = JSON.stringify(newSession, null, 2);
      log.info(`invoke write_text_file (+${(performance.now() - t0).toFixed(0)}ms)`);
      await invoke("write_text_file", { path: filePath, content: json });
      log.info(`write_text_file done (+${(performance.now() - t0).toFixed(0)}ms)`);
    } else {
      log.info("Browser preview acquisition session created in memory");
    }
    addRecentProject(filePath, opts.caseName || opts.caseNumber || getBasename(filePath));

    log.info(`Session created: ${filePath} (total=${(performance.now() - t0).toFixed(0)}ms)`);
    addActivity({ action: "session_created", description: `Session created for case ${opts.caseNumber}` });
  }

  async function load(path: string): Promise<void> {
    const content = await invoke<string>("read_text_file", { path });
    const parsed = JSON.parse(content) as AcquisitionSession;
    loadParsed(parsed, path);
  }

  function loadParsed(parsed: AcquisitionSession, path: string): void {
    // Basic validation
    if (!parsed.version || !parsed.caseNumber) {
      throw new Error("Invalid acquisition session file");
    }

    setSession(parsed);
    setSessionPath(path);
    addRecentProject(path, parsed.caseName || parsed.caseNumber || getBasename(path));
    log.info(`Session loaded: ${path}`);
    addActivity({ action: "session_loaded", description: `Session loaded from ${path}` });
  }

  function close(): void {
    // Flush any pending save
    if (saveTimer) {
      clearTimeout(saveTimer);
      saveTimer = null;
      const s = session();
      const p = sessionPath();
      if (s && p) {
        const updated = { ...s, modifiedAt: new Date().toISOString() };
        if (isTauri) {
          const json = JSON.stringify(updated, null, 2);
          invoke("write_text_file", { path: p, content: json }).catch(() => {});
        }
      }
    }
    setSession(null);
    setSessionPath("");
    log.info("Session closed");
  }

  // ─── Mutations ────────────────────────────────────────────────────────

  function addAcquisition(record: SessionAcquisitionRecord): void {
    setSession((prev) => {
      if (!prev) return prev;
      return { ...prev, acquisitions: [...prev.acquisitions, record] };
    });
    scheduleSave();
  }

  function updateAcquisition(id: string, updates: Partial<SessionAcquisitionRecord>): void {
    setSession((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        acquisitions: prev.acquisitions.map((a) =>
          a.id === id ? { ...a, ...updates } : a,
        ),
      };
    });
    scheduleSave();
  }

  function addActivity(entry: Omit<SessionActivityEntry, "id" | "timestamp">): void {
    const fullEntry: SessionActivityEntry = {
      id: `act-${Date.now()}-${Math.random().toString(36).substring(2, 6)}`,
      timestamp: new Date().toISOString(),
      ...entry,
    };
    setSession((prev) => {
      if (!prev) return prev;
      return { ...prev, activity: [...prev.activity, fullEntry] };
    });
    scheduleSave();
  }

  return {
    session,
    sessionPath,
    hasSession,
    caseNumber,
    examiner,
    projectName,
    outputFolder,
    evidenceFolder,
    create,
    load,
    loadParsed,
    close,
    addAcquisition,
    updateAcquisition,
    addActivity,
  };
}

// =============================================================================
// Helpers
// =============================================================================

function buildSessionFileName(caseNumber: string): string {
  const safe = caseNumber.replace(/[^a-zA-Z0-9._-]/g, "_") || "session";
  const date = new Date().toISOString().slice(0, 10); // YYYY-MM-DD
  return `${safe}_${date}${ACQUISITION_SESSION_EXTENSION}`;
}
