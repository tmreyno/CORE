// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import "./E01V3Test.css";
import { formatBytes } from "./utils";

type E01V3Info = {
  segment_count: number;
  chunk_count: number;
  sector_count: number;
  bytes_per_sector: number;
  sectors_per_chunk: number;
};

type StatusKind = "idle" | "working" | "ok" | "error";

type StatusState = {
  kind: StatusKind;
  message: string;
};

type VerifyProgress = {
  path: string;
  current: number;
  total: number;
  percent: number;
};

const defaultStatus: StatusState = {
  kind: "idle",
  message: "Ready",
};

export function E01V3Test() {
  const [filePath, setFilePath] = createSignal<string>("");
  const [info, setInfo] = createSignal<E01V3Info | null>(null);
  const [hash, setHash] = createSignal<string>("");
  const [status, setStatus] = createSignal<StatusState>(defaultStatus);
  const [verifyTime, setVerifyTime] = createSignal<number>(0);
  const [progress, setProgress] = createSignal<VerifyProgress | null>(null);

  async function selectFile() {
    try {
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: "E01 Files",
            extensions: ["E01", "e01"],
          },
        ],
      });

      if (selected) {
        setFilePath(selected as string);
        setInfo(null);
        setHash("");
        setStatus(defaultStatus);
      }
    } catch (error) {
      setStatus({
        kind: "error",
        message: `Failed to select file: ${error}`,
      });
    }
  }

  async function loadInfo() {
    if (!filePath()) return;

    setStatus({ kind: "working", message: "Loading E01 info..." });
    setInfo(null);

    try {
      const result = await invoke<E01V3Info>("e01_v3_info", {
        inputPath: filePath(),
      });

      setInfo(result);
      setStatus({ kind: "ok", message: "Info loaded successfully" });
    } catch (error) {
      setStatus({
        kind: "error",
        message: `Failed to load info: ${error}`,
      });
    }
  }

  async function verifyHash(algorithm: string) {
    if (!filePath()) return;

    setStatus({
      kind: "working",
      message: `Computing ${algorithm.toUpperCase()} hash... 0%`,
    });
    setHash("");
    setProgress(null);

    const startTime = performance.now();

    // Listen for progress events
    const unlisten = await listen<VerifyProgress>("verify-progress", (event) => {
      const p = event.payload;
      setProgress(p);
      setStatus({
        kind: "working",
        message: `Computing ${algorithm.toUpperCase()} hash... ${p.percent.toFixed(1)}%`,
      });
    });

    try {
      const result = await invoke<string>("e01_v3_verify", {
        inputPath: filePath(),
        algorithm: algorithm,
      });

      const endTime = performance.now();
      const duration = (endTime - startTime) / 1000;

      setHash(result);
      setVerifyTime(duration);
      setProgress(null);
      setStatus({
        kind: "ok",
        message: `${algorithm.toUpperCase()} computed in ${duration.toFixed(2)}s`,
      });
    } catch (error) {
      setStatus({
        kind: "error",
        message: `Failed to verify: ${error}`,
      });
      setProgress(null);
    } finally {
      unlisten();
    }
  }

  const totalSize = () => {
    const i = info();
    if (!i) return 0;
    return i.sector_count * i.bytes_per_sector;
  };

  const throughput = () => {
    if (verifyTime() === 0) return 0;
    return totalSize() / verifyTime() / (1024 * 1024); // MB/s
  };

  return (
    <div class="container">
      <h1>E01 v3 Implementation Test</h1>
      <p class="subtitle">libewf-inspired architecture with Rust</p>

      {/* File Selection */}
      <div class="card">
        <h2>1. Select E01 File</h2>
        <button onClick={selectFile}>Select E01 File</button>
        <Show when={filePath()}>
          <div class="file-info">
            <strong>Selected:</strong>
            <div class="file-path">{filePath()}</div>
          </div>
        </Show>
      </div>

      {/* Load Info */}
      <Show when={filePath()}>
        <div class="card">
          <h2>2. Load Container Info</h2>
          <button onClick={loadInfo} disabled={status().kind === "working"}>
            Load Info
          </button>

          <Show when={info()}>
            <div class="info-section">
              <h3>Container Information</h3>
              <table class="info-table">
                <tbody>
                  <tr>
                    <td>Segments:</td>
                    <td>{info()!.segment_count}</td>
                  </tr>
                  <tr>
                    <td>Chunks:</td>
                    <td>{info()!.chunk_count.toLocaleString()}</td>
                  </tr>
                  <tr>
                    <td>Sectors:</td>
                    <td>{info()!.sector_count.toLocaleString()}</td>
                  </tr>
                  <tr>
                    <td>Bytes per Sector:</td>
                    <td>{info()!.bytes_per_sector}</td>
                  </tr>
                  <tr>
                    <td>Sectors per Chunk:</td>
                    <td>{info()!.sectors_per_chunk}</td>
                  </tr>
                  <tr>
                    <td>Total Size:</td>
                    <td>{formatBytes(totalSize())}</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </Show>
        </div>
      </Show>

      {/* Verify Hash */}
      <Show when={filePath()}>
        <div class="card">
          <h2>3. Verify Hash</h2>
          
          {/* Cryptographic Hashes (Forensic Standard) */}
          <h4 style="margin: 0.5rem 0; color: #888;">Cryptographic (Forensic)</h4>
          <div class="button-group">
            <button
              onClick={() => verifyHash("md5")}
              disabled={status().kind === "working"}
            >
              MD5
            </button>
            <button
              onClick={() => verifyHash("sha1")}
              disabled={status().kind === "working"}
            >
              SHA-1
            </button>
            <button
              onClick={() => verifyHash("sha256")}
              disabled={status().kind === "working"}
            >
              SHA-256
            </button>
            <button
              onClick={() => verifyHash("sha512")}
              disabled={status().kind === "working"}
            >
              SHA-512
            </button>
          </div>
          
          {/* Modern Hashes */}
          <h4 style="margin: 0.5rem 0; color: #888;">Modern / High Performance</h4>
          <div class="button-group">
            <button
              onClick={() => verifyHash("blake3")}
              disabled={status().kind === "working"}
            >
              BLAKE3
            </button>
            <button
              onClick={() => verifyHash("blake2")}
              disabled={status().kind === "working"}
            >
              BLAKE2b
            </button>
          </div>
          
          {/* Non-Cryptographic (Speed Tests) */}
          <h4 style="margin: 0.5rem 0; color: #888;">Non-Cryptographic (Fast)</h4>
          <div class="button-group">
            <button
              onClick={() => verifyHash("xxh64")}
              disabled={status().kind === "working"}
            >
              XXH64
            </button>
            <button
              onClick={() => verifyHash("xxh3")}
              disabled={status().kind === "working"}
            >
              XXH3
            </button>
            <button
              onClick={() => verifyHash("crc32")}
              disabled={status().kind === "working"}
            >
              CRC32
            </button>
          </div>

          <Show when={progress()}>
            <div class="progress-container">
              <div class="progress-label">
                Processing: {progress()!.current.toLocaleString()} / {progress()!.total.toLocaleString()} chunks ({progress()!.percent.toFixed(1)}%)
              </div>
              <div class="progress-bar-bg">
                <div class="progress-bar-fill" style={`width: ${progress()!.percent}%`} />
              </div>
            </div>
          </Show>

          <Show when={hash()}>
            <div class="hash-result">
              <h3>Hash Result</h3>
              <div class="hash-value">{hash()}</div>
              <div class="performance-stats">
                <div>
                  <strong>Time:</strong> {verifyTime().toFixed(2)}s
                </div>
                <div>
                  <strong>Throughput:</strong> {throughput().toFixed(2)} MB/s
                </div>
                <div>
                  <strong>Data Processed:</strong> {formatBytes(totalSize())}
                </div>
              </div>

              <div class="comparison">
                <h4>Expected libewf Result:</h4>
                <div class="expected-hash">
                  aee4fcd9301c03b3b054623ca261959a
                </div>
                <div class="match-indicator">
                  <Show
                    when={
                      hash().toLowerCase() ===
                      "aee4fcd9301c03b3b054623ca261959a"
                    }
                    fallback={
                      <span class="mismatch">✗ Hash does not match</span>
                    }
                  >
                    <span class="match">✓ Hash matches libewf!</span>
                  </Show>
                </div>
              </div>
            </div>
          </Show>
        </div>
      </Show>

      {/* Status Bar */}
      <div class={`status status-${status().kind}`}>
        <Show when={status().kind === "working"}>
          <div class="spinner" />
        </Show>
        {status().message}
      </div>

      {/* Info Panel */}
      <div class="info-panel">
        <h3>About E01 v3</h3>
        <ul>
          <li>
            <strong>Architecture:</strong> Based on libewf's proven design
          </li>
          <li>
            <strong>File I/O Pool:</strong> LRU cache with max open files limit
          </li>
          <li>
            <strong>Chunk Cache:</strong> Intelligent caching of decompressed
            data
          </li>
          <li>
            <strong>Performance:</strong> Target 300+ MB/s (libewf: ~331 MB/s)
          </li>
          <li>
            <strong>Safety:</strong> Rust's ownership model prevents memory
            errors
          </li>
        </ul>
      </div>
    </div>
  );
}
