// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import "./E01V3Test.css"; // Reuse E01 styles
import { formatBytes } from "./utils";

type RawInfo = {
  segment_count: number;
  total_size: number;
  segment_sizes: number[];
  first_segment: string;
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

export function RawTest() {
  const [filePath, setFilePath] = createSignal<string>("");
  const [info, setInfo] = createSignal<RawInfo | null>(null);
  const [hash, setHash] = createSignal<string>("");
  const [algorithm, setAlgorithm] = createSignal<string>("");
  const [status, setStatus] = createSignal<StatusState>(defaultStatus);
  const [verifyTime, setVerifyTime] = createSignal<number>(0);
  const [progress, setProgress] = createSignal<VerifyProgress | null>(null);

  async function selectFile() {
    try {
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: "Raw Disk Images",
            extensions: ["dd", "raw", "img", "001", "000"],
          },
        ],
      });

      if (selected) {
        setFilePath(selected as string);
        setInfo(null);
        setHash("");
        setAlgorithm("");
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

    setStatus({ kind: "working", message: "Loading raw image info..." });
    setInfo(null);

    try {
      const result = await invoke<RawInfo>("raw_info", {
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

  async function verifyHash(algo: string) {
    if (!filePath()) return;

    setStatus({
      kind: "working",
      message: `Computing ${algo.toUpperCase()} hash... 0%`,
    });
    setHash("");
    setAlgorithm(algo);
    setProgress(null);

    const startTime = performance.now();

    // Listen for progress events
    const unlisten = await listen<VerifyProgress>("verify-progress", (event) => {
      const p = event.payload;
      setProgress(p);
      setStatus({
        kind: "working",
        message: `Computing ${algo.toUpperCase()} hash... ${p.percent.toFixed(1)}%`,
      });
    });

    try {
      const result = await invoke<string>("raw_verify", {
        inputPath: filePath(),
        algorithm: algo,
      });

      const endTime = performance.now();
      const duration = (endTime - startTime) / 1000;

      setHash(result);
      setVerifyTime(duration);
      setProgress(null);
      setStatus({
        kind: "ok",
        message: `${algo.toUpperCase()} computed in ${duration.toFixed(2)}s`,
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

  const totalSize = () => info()?.total_size ?? 0;

  const throughput = () => {
    if (verifyTime() === 0) return 0;
    return totalSize() / verifyTime() / (1024 * 1024); // MB/s
  };

  return (
    <div class="container">
      <h1>Raw Disk Image Test</h1>
      <p class="subtitle">.dd, .raw, .img, .001 multi-segment support</p>

      {/* File Selection */}
      <div class="card">
        <h2>1. Select Raw Image</h2>
        <button onClick={selectFile}>Select Raw Image</button>
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
          <h2>2. Load Image Info</h2>
          <button onClick={loadInfo} disabled={status().kind === "working"}>
            Load Info
          </button>

          <Show when={info()}>
            <div class="info-section">
              <h3>Image Information</h3>
              <table class="info-table">
                <tbody>
                  <tr>
                    <td>Segments:</td>
                    <td>{info()!.segment_count}</td>
                  </tr>
                  <tr>
                    <td>Total Size:</td>
                    <td>{formatBytes(info()!.total_size)}</td>
                  </tr>
                  <tr>
                    <td>First Segment:</td>
                    <td style="font-family: monospace; font-size: 0.9em;">{info()!.first_segment}</td>
                  </tr>
                </tbody>
              </table>
              
              <Show when={info()!.segment_count > 1}>
                <h4 style="margin-top: 1rem;">Segment Sizes</h4>
                <div style="max-height: 150px; overflow-y: auto;">
                  <table class="info-table">
                    <tbody>
                      {info()!.segment_sizes.map((size, i) => (
                        <tr>
                          <td>Segment {i + 1}:</td>
                          <td>{formatBytes(size)}</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </Show>
            </div>
          </Show>
        </div>
      </Show>

      {/* Verify Hash */}
      <Show when={filePath()}>
        <div class="card">
          <h2>3. Compute Hash</h2>
          
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
                Processing: {formatBytes(progress()!.current)} / {formatBytes(progress()!.total)} ({progress()!.percent.toFixed(1)}%)
              </div>
              <div class="progress-bar-bg">
                <div class="progress-bar-fill" style={`width: ${progress()!.percent}%`} />
              </div>
            </div>
          </Show>

          <Show when={hash()}>
            <div class="hash-result">
              <h3>Hash Result ({algorithm().toUpperCase()})</h3>
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
        <h3>About Raw Images</h3>
        <ul>
          <li>
            <strong>Formats:</strong> .dd, .raw, .img, .001/.002/.003...
          </li>
          <li>
            <strong>Multi-Segment:</strong> Automatically discovers all segments
          </li>
          <li>
            <strong>Hashing:</strong> 9 algorithms (MD5, SHA-1/256/512, BLAKE2/3, XXH3/64, CRC32)
          </li>
          <li>
            <strong>Performance:</strong> Sequential streaming for maximum I/O throughput
          </li>
          <li>
            <strong>Use Case:</strong> Verify integrity of dd/raw forensic acquisitions
          </li>
        </ul>
      </div>
    </div>
  );
}
