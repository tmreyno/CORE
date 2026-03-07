// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { type Component, createSignal, Show } from "solid-js";

export const AboutContent: Component = () => {
  const [showLicenses, setShowLicenses] = createSignal(false);

  return (
    <div class="space-y-4">
      <div class="text-center py-4">
        <div class="inline-flex items-center justify-center w-16 h-16 bg-accent/10 rounded-2xl mb-3">
          <span class="text-4xl">🔍</span>
        </div>
        <h3 class="text-xl font-bold text-txt">CORE-FFX</h3>
        <p class="text-txt-secondary text-sm">Forensic File Xplorer</p>
        <p class="text-txt-muted text-xs mt-1">© 2024–2026 CORE-FFX Project Contributors</p>
        <p class="text-txt-muted text-xs">Licensed under the MIT License</p>
      </div>

      <div class="space-y-2">
        <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
          <div class="font-medium text-txt text-sm">Technology Stack</div>
          <p class="text-xs text-txt-muted mt-1">
            <strong>Backend:</strong> Rust + Tauri v2 — native performance with web-based UI<br />
            <strong>Frontend:</strong> SolidJS + TypeScript — reactive UI with fine-grained updates<br />
            <strong>Storage:</strong> SQLite (per-project .ffxdb) — reliable, portable data persistence
          </p>
        </div>
        <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
          <div class="font-medium text-txt text-sm">Key Libraries</div>
          <p class="text-xs text-txt-muted mt-1">
            <strong>libewf</strong> — EWF image creation (E01 export)<br />
            <strong>LZMA SDK 24.09</strong> — 7z archive creation<br />
            <strong>Pure-Rust parsers</strong> — E01/L01 reading, AD1, UFED, filesystem drivers
          </p>
        </div>
      </div>

      {/* Third-Party Software Notice */}
      <div class="space-y-2">
        <button
          class="w-full text-left p-3 bg-bg-secondary rounded-lg border border-border/50 hover:bg-bg-hover transition-colors"
          onClick={() => setShowLicenses(!showLicenses())}
        >
          <div class="flex items-center justify-between">
            <div class="font-medium text-txt text-sm">Third-Party Software Licenses</div>
            <span class="text-txt-muted text-xs">{showLicenses() ? "▲ Hide" : "▼ Show"}</span>
          </div>
          <p class="text-xs text-txt-muted mt-1">
            CORE-FFX incorporates open-source software under various licenses.
          </p>
        </button>

        <Show when={showLicenses()}>
          <div class="p-3 bg-bg-secondary rounded-lg border border-border/50 space-y-3 text-xs text-txt-muted">
            {/* LGPL Notice — legally required */}
            <div>
              <div class="font-semibold text-txt text-sm mb-1">⚖️ LGPL-3.0 Notice — libewf</div>
              <p>
                This application links against{" "}
                <strong>libewf</strong> (© 2006–2025 Joachim Metz), licensed under the{" "}
                <strong>GNU Lesser General Public License v3.0 or later</strong>.
                The libewf source code is available at{" "}
                <a
                  href="https://github.com/libyal/libewf"
                  class="text-accent hover:underline"
                  target="_blank"
                  rel="noopener"
                >
                  github.com/libyal/libewf
                </a>
                . Users may re-link this application against a modified version of libewf
                in accordance with the LGPL.
              </p>
            </div>

            {/* Native C Libraries */}
            <div>
              <div class="font-semibold text-txt text-sm mb-1">Native Libraries</div>
              <table class="w-full text-left">
                <thead>
                  <tr class="border-b border-border/30">
                    <th class="pb-1 pr-2">Library</th>
                    <th class="pb-1 pr-2">License</th>
                    <th class="pb-1">Purpose</th>
                  </tr>
                </thead>
                <tbody>
                  <tr><td class="py-0.5 pr-2">libarchive</td><td class="pr-2">BSD 2-Clause</td><td>Archive reading</td></tr>
                  <tr><td class="py-0.5 pr-2">libewf</td><td class="pr-2">LGPL-3.0</td><td>EWF image creation</td></tr>
                  <tr><td class="py-0.5 pr-2">LZMA SDK</td><td class="pr-2">Public Domain</td><td>7z archive creation</td></tr>
                  <tr><td class="py-0.5 pr-2">zlib</td><td class="pr-2">zlib License</td><td>Compression</td></tr>
                </tbody>
              </table>
            </div>

            {/* Key Open Source */}
            <div>
              <div class="font-semibold text-txt text-sm mb-1">Key Open-Source Components</div>
              <table class="w-full text-left">
                <thead>
                  <tr class="border-b border-border/30">
                    <th class="pb-1 pr-2">Component</th>
                    <th class="pb-1 pr-2">License</th>
                  </tr>
                </thead>
                <tbody>
                  <tr><td class="py-0.5 pr-2">Tauri Framework</td><td>MIT / Apache-2.0</td></tr>
                  <tr><td class="py-0.5 pr-2">SolidJS</td><td>MIT</td></tr>
                  <tr><td class="py-0.5 pr-2">PDF.js (pdfjs-dist)</td><td>Apache-2.0</td></tr>
                  <tr><td class="py-0.5 pr-2">DOMPurify</td><td>MPL-2.0 / Apache-2.0</td></tr>
                  <tr><td class="py-0.5 pr-2">Tailwind CSS</td><td>MIT</td></tr>
                  <tr><td class="py-0.5 pr-2">SQLite (rusqlite)</td><td>MIT</td></tr>
                </tbody>
              </table>
            </div>

            {/* Fonts */}
            <div>
              <div class="font-semibold text-txt text-sm mb-1">Fonts</div>
              <p>
                <strong>Inter</strong> (© Rasmus Andersson) and{" "}
                <strong>JetBrains Mono</strong> (© JetBrains s.r.o.) are used under the{" "}
                <strong>SIL Open Font License 1.1</strong>.
              </p>
            </div>

            {/* Full Listing */}
            <div class="pt-1 border-t border-border/30">
              <p>
                For the complete list of all dependencies and their licenses, see{" "}
                <strong>THIRD_PARTY_LICENSES.md</strong> in the application source repository.
                Over 800 Rust crates and 25+ npm packages are used, primarily under MIT
                and Apache-2.0 licenses.
              </p>
            </div>
          </div>
        </Show>
      </div>
    </div>
  );
};
