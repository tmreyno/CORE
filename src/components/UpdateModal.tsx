// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * UpdateModal — Check for updates, download, and install.
 *
 * Uses the Tauri updater plugin (@tauri-apps/plugin-updater) to check
 * GitHub releases for new versions. Shows download progress and handles
 * the install + restart flow.
 */

import { Component, Show, createSignal, createMemo, onMount } from "solid-js";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import DOMPurify from "dompurify";
import { HiOutlineArrowPath } from "./icons";
import { logger } from "../utils/logger";

const log = logger.scope("Updater");

type UpdateState = "checking" | "up-to-date" | "available" | "downloading" | "ready" | "error";

/** Lightweight markdown-to-HTML for release notes (headings, bold, lists, links, code). */
function markdownToHtml(md: string): string {
  return md
    .replace(/^#### (.+)$/gm, "<h4>$1</h4>")
    .replace(/^### (.+)$/gm, "<h3>$1</h3>")
    .replace(/^## (.+)$/gm, "<h2>$1</h2>")
    .replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>")
    .replace(/`([^`]+)`/g, "<code>$1</code>")
    .replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2">$1</a>')
    .replace(/^[-*] (.+)$/gm, "<li>$1</li>")
    .replace(/(<li>.*<\/li>\n?)+/g, (m) => `<ul>${m}</ul>`)
    .replace(/^---$/gm, "<hr/>")
    .replace(/\n{2,}/g, "<br/><br/>")
    .trim();
}

interface UpdateModalProps {
  show: boolean;
  onClose: () => void;
}

const UpdateModal: Component<UpdateModalProps> = (props) => {
  const [state, setState] = createSignal<UpdateState>("checking");
  const [update, setUpdate] = createSignal<Update | null>(null);
  const [progress, setProgress] = createSignal(0);
  const [downloadedBytes, setDownloadedBytes] = createSignal(0);
  const [totalBytes, setTotalBytes] = createSignal(0);
  const [errorMessage, setErrorMessage] = createSignal("");
  const [currentVersion] = createSignal(__APP_VERSION__);

  /**
   * Build auth headers for private GitHub repo access.
   * The token is injected at build time via VITE_GITHUB_UPDATE_TOKEN.
   * When the repo is public, this returns an empty object (harmless).
   * MUST be passed to both check() AND downloadAndInstall() — otherwise
   * the binary download from a private repo returns 404/HTML and
   * signature verification fails against garbage data.
   */
  function getAuthHeaders(): Record<string, string> {
    const token = typeof __GITHUB_UPDATE_TOKEN__ === "string" ? __GITHUB_UPDATE_TOKEN__ : "";
    if (token) {
      return { Authorization: `token ${token}` };
    }
    return {};
  }

  /** Sanitized HTML from release notes markdown */
  const releaseNotesHtml = createMemo(() => {
    const body = update()?.body;
    if (!body) return "";
    return DOMPurify.sanitize(markdownToHtml(body));
  });

  onMount(async () => {
    if (props.show) {
      await checkForUpdates();
    }
  });

  async function checkForUpdates() {
    setState("checking");
    setErrorMessage("");

    try {
      log.info("Checking for updates...");

      const headers = getAuthHeaders();
      const result = await check({ headers });

      if (result) {
        log.info(`Update available: ${result.version}`);
        setUpdate(result);
        setState("available");
      } else {
        log.info("Application is up to date");
        setState("up-to-date");
      }
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      log.error(`Update check failed: ${msg}`);
      setErrorMessage(msg);
      setState("error");
    }
  }

  async function downloadAndInstall() {
    const upd = update();
    if (!upd) return;

    setState("downloading");
    setProgress(0);
    setDownloadedBytes(0);
    setTotalBytes(0);

    try {
      log.info(`Downloading update ${upd.version}...`);

      let downloaded = 0;
      let contentLength = 0;

      // Pass auth headers for private repo binary download — without this,
      // GitHub returns 404/HTML and signature verification fails.
      const headers = getAuthHeaders();

      await upd.downloadAndInstall((event) => {
        switch (event.event) {
          case "Started":
            contentLength = event.data.contentLength ?? 0;
            setTotalBytes(contentLength);
            log.info(`Download started, size: ${formatBytes(contentLength)}`);
            break;
          case "Progress":
            downloaded += event.data.chunkLength;
            setDownloadedBytes(downloaded);
            if (contentLength > 0) {
              setProgress(Math.round((downloaded / contentLength) * 100));
            }
            break;
          case "Finished":
            log.info("Download complete, update installed");
            setProgress(100);
            setState("ready");
            break;
        }
      }, { headers });

      setState("ready");
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      log.error(`Download failed: ${msg}`);
      setErrorMessage(msg);
      setState("error");
    }
  }

  async function handleRelaunch() {
    log.info("Relaunching application...");
    await relaunch();
  }

  function formatBytes(bytes: number): string {
    if (bytes === 0) return "0 B";
    const units = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${units[i]}`;
  }

  return (
    <Show when={props.show}>
      <div class="modal-overlay" onClick={(e) => { if (e.target === e.currentTarget) props.onClose(); }}>
        <div class="modal-content w-[460px]">
          {/* Header */}
          <div class="modal-header">
            <div class="flex items-center gap-2">
              <HiOutlineArrowPath class="w-icon-base h-icon-base text-accent" />
              <h2 class="text-lg font-semibold text-txt">Software Update</h2>
            </div>
            <button class="icon-btn-sm" onClick={props.onClose} title="Close">
              <svg class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
                <path d="M6.28 5.22a.75.75 0 00-1.06 1.06L8.94 10l-3.72 3.72a.75.75 0 101.06 1.06L10 11.06l3.72 3.72a.75.75 0 101.06-1.06L11.06 10l3.72-3.72a.75.75 0 00-1.06-1.06L10 8.94 6.28 5.22z" />
              </svg>
            </button>
          </div>

          {/* Body */}
          <div class="modal-body">
            {/* Checking */}
            <Show when={state() === "checking"}>
              <div class="flex flex-col items-center gap-4 py-6">
                <div class="animate-spin w-8 h-8 border-2 border-accent border-t-transparent rounded-full" />
                <p class="text-txt-secondary text-sm">Checking for updates…</p>
              </div>
            </Show>

            {/* Up to date */}
            <Show when={state() === "up-to-date"}>
              <div class="flex flex-col items-center gap-3 py-6">
                <div class="w-12 h-12 rounded-full bg-success/10 flex items-center justify-center">
                  <svg class="w-6 h-6 text-success" viewBox="0 0 20 20" fill="currentColor">
                    <path fill-rule="evenodd" d="M16.704 4.153a.75.75 0 01.143 1.052l-8 10.5a.75.75 0 01-1.127.075l-4.5-4.5a.75.75 0 011.06-1.06l3.894 3.893 7.48-9.817a.75.75 0 011.05-.143z" clip-rule="evenodd" />
                  </svg>
                </div>
                <p class="text-txt font-medium">You're up to date!</p>
                <p class="text-txt-muted text-sm">
                  CORE-FFX {currentVersion()} is the latest version.
                </p>
              </div>
            </Show>

            {/* Update available */}
            <Show when={state() === "available"}>
              <div class="flex flex-col gap-4 py-2">
                <div class="flex items-center gap-3">
                  <div class="w-10 h-10 rounded-full bg-accent/10 flex items-center justify-center flex-shrink-0">
                    <HiOutlineArrowPath class="w-5 h-5 text-accent" />
                  </div>
                  <div>
                    <p class="text-txt font-medium">Update Available</p>
                    <p class="text-txt-muted text-sm">
                      Version {update()?.version} is ready to download.
                    </p>
                  </div>
                </div>

                <div class="bg-bg-secondary rounded-lg p-3 space-y-1">
                  <div class="flex justify-between text-sm">
                    <span class="text-txt-muted">Current version</span>
                    <span class="text-txt font-mono">{currentVersion()}</span>
                  </div>
                  <div class="flex justify-between text-sm">
                    <span class="text-txt-muted">New version</span>
                    <span class="text-accent font-mono font-medium">{update()?.version}</span>
                  </div>
                </div>

                {/* Release notes */}
                <Show when={releaseNotesHtml()}>
                  <div class="space-y-1">
                    <h3 class="text-sm font-medium text-txt">Release Notes</h3>
                    <div
                      class="bg-bg-secondary rounded-lg p-3 max-h-60 overflow-y-auto release-notes text-txt-secondary text-sm"
                      innerHTML={releaseNotesHtml()}
                    />
                  </div>
                </Show>
              </div>
            </Show>

            {/* Downloading */}
            <Show when={state() === "downloading"}>
              <div class="flex flex-col gap-4 py-4">
                <p class="text-txt text-sm font-medium">Downloading update…</p>
                <div class="w-full bg-bg-secondary rounded-full h-2.5 overflow-hidden">
                  <div
                    class="bg-accent h-full rounded-full transition-all duration-300"
                    style={{ width: `${progress()}%` }}
                  />
                </div>
                <div class="flex justify-between text-xs text-txt-muted">
                  <span>{formatBytes(downloadedBytes())} / {formatBytes(totalBytes())}</span>
                  <span>{progress()}%</span>
                </div>
              </div>
            </Show>

            {/* Ready to restart */}
            <Show when={state() === "ready"}>
              <div class="flex flex-col items-center gap-3 py-6">
                <div class="w-12 h-12 rounded-full bg-success/10 flex items-center justify-center">
                  <svg class="w-6 h-6 text-success" viewBox="0 0 20 20" fill="currentColor">
                    <path fill-rule="evenodd" d="M16.704 4.153a.75.75 0 01.143 1.052l-8 10.5a.75.75 0 01-1.127.075l-4.5-4.5a.75.75 0 011.06-1.06l3.894 3.893 7.48-9.817a.75.75 0 011.05-.143z" clip-rule="evenodd" />
                  </svg>
                </div>
                <p class="text-txt font-medium">Update installed!</p>
                <p class="text-txt-muted text-sm text-center">
                  Restart CORE-FFX to apply the update.
                </p>
              </div>
            </Show>

            {/* Error */}
            <Show when={state() === "error"}>
              <div class="flex flex-col gap-3 py-4">
                <div class="flex items-start gap-3">
                  <div class="w-10 h-10 rounded-full bg-error/10 flex items-center justify-center flex-shrink-0">
                    <svg class="w-5 h-5 text-error" viewBox="0 0 20 20" fill="currentColor">
                      <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-8-5a.75.75 0 01.75.75v4.5a.75.75 0 01-1.5 0v-4.5A.75.75 0 0110 5zm0 10a1 1 0 100-2 1 1 0 000 2z" clip-rule="evenodd" />
                    </svg>
                  </div>
                  <div>
                    <p class="text-txt font-medium">Update check failed</p>
                    <p class="text-txt-muted text-sm mt-1">{errorMessage()}</p>
                  </div>
                </div>
              </div>
            </Show>
          </div>

          {/* Footer */}
          <div class="modal-footer justify-end">
            <Show when={state() === "checking" || state() === "up-to-date" || state() === "error"}>
              <button class="btn btn-secondary" onClick={props.onClose}>Close</button>
            </Show>

            <Show when={state() === "error"}>
              <button class="btn btn-primary" onClick={checkForUpdates}>Retry</button>
            </Show>

            <Show when={state() === "available"}>
              <button class="btn btn-secondary" onClick={props.onClose}>Later</button>
              <button class="btn btn-primary" onClick={downloadAndInstall}>
                Download & Install
              </button>
            </Show>

            <Show when={state() === "downloading"}>
              <button class="btn btn-secondary" disabled>Downloading…</button>
            </Show>

            <Show when={state() === "ready"}>
              <button class="btn btn-secondary" onClick={props.onClose}>Later</button>
              <button class="btn btn-primary" onClick={handleRelaunch}>
                Restart Now
              </button>
            </Show>
          </div>
        </div>
      </div>
    </Show>
  );
};

export default UpdateModal;
