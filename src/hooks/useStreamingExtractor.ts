import { createSignal, onMount, onCleanup, createMemo } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { logger } from "../utils/logger";

const log = logger.scope("StreamingExtractor");

export enum ExtractionPriority {
  Low = 0,
  Normal = 1,
  High = 2,
  Critical = 3,
}

export interface StreamExtractionJob {
  id: string;
  sourcePath: string;
  destPath: string;
  containerPath: string;
  containerType: string;
  priority: ExtractionPriority;
  sizeBytes: number;
  expectedHash?: string;
  hashAlgorithm?: string;
}

export type JobState = "queued" | "extracting" | "verifying" | "complete" | "failed" | "cancelled";

export interface JobStatus {
  id: string;
  status: JobState;
  progressBytes: number;
  totalBytes: number;
  throughputMbps: number;
  error?: string;
  completedHash?: string;
  elapsedMs: number;
}

export interface StreamProgress {
  streamId: string;
  totalJobs: number;
  queuedJobs: number;
  activeJobs: number;
  completedJobs: number;
  failedJobs: number;
  totalBytes: number;
  extractedBytes: number;
  overallThroughputMbps: number;
  etaSeconds?: number;
}

export interface FileAvailableEvent {
  streamId: string;
  jobId: string;
  filePath: string;
  sizeBytes: number;
  hash?: string;
  verified: boolean;
}

export function useStreamingExtractor() {
  const [initialized, setInitialized] = createSignal(false);
  const [activeStreams, setActiveStreams] = createSignal<Map<string, StreamProgress>>(new Map());
  const [fileAvailable, setFileAvailable] = createSignal<FileAvailableEvent | null>(null);
  const [error, setError] = createSignal<string | null>(null);

  let fileAvailableUnlisten: (() => void) | null = null;
  let progressUnlisten: (() => void) | null = null;
  let completeUnlisten: (() => void) | null = null;

  onMount(async () => {
    try {
      // Initialize streaming extractor
      await invoke("stream_extract_init");
      setInitialized(true);

      // Listen for file available events
      fileAvailableUnlisten = await listen<FileAvailableEvent>(
        "file-available",
        (event) => {
          setFileAvailable(event.payload);
          log.debug(`File available: ${event.payload.filePath}`);
        }
      );

      // Listen for progress updates
      progressUnlisten = await listen<StreamProgress>(
        "stream-progress",
        (event) => {
          setActiveStreams((map) => {
            const newMap = new Map(map);
            newMap.set(event.payload.streamId, event.payload);
            return newMap;
          });
        }
      );

      // Listen for stream completion
      completeUnlisten = await listen<StreamProgress>(
        "stream-complete",
        (event) => {
          log.debug(`Stream complete: ${event.payload.streamId}`);
          setActiveStreams((map) => {
            const newMap = new Map(map);
            newMap.set(event.payload.streamId, event.payload);
            return newMap;
          });
        }
      );
    } catch (err) {
      setError(String(err));
    }
  });

  onCleanup(() => {
    if (fileAvailableUnlisten) fileAvailableUnlisten();
    if (progressUnlisten) progressUnlisten();
    if (completeUnlisten) completeUnlisten();
  });

  const startStream = async (
    streamId: string,
    jobs: StreamExtractionJob[],
    maxConcurrent: number = 4
  ): Promise<void> => {
    try {
      setError(null);
      await invoke("stream_extract_start", {
        streamId,
        jobs,
        maxConcurrent,
      });
    } catch (err) {
      setError(String(err));
      throw err;
    }
  };

  const getProgress = async (streamId: string): Promise<StreamProgress> => {
    try {
      return await invoke<StreamProgress>("stream_extract_get_progress", {
        streamId,
      });
    } catch (err) {
      setError(String(err));
      throw err;
    }
  };

  const cancelStream = async (streamId: string): Promise<void> => {
    try {
      await invoke("stream_extract_cancel", { streamId });
    } catch (err) {
      setError(String(err));
      throw err;
    }
  };

  const getActiveStreamIds = async (): Promise<string[]> => {
    try {
      return await invoke<string[]>("stream_extract_get_active");
    } catch (err) {
      setError(String(err));
      return [];
    }
  };

  const getJobStatuses = async (streamId: string): Promise<JobStatus[]> => {
    try {
      return await invoke<JobStatus[]>("stream_extract_get_job_statuses", {
        streamId,
      });
    } catch (err) {
      setError(String(err));
      return [];
    }
  };

  // Computed: Overall statistics across all streams
  const overallStats = createMemo(() => {
    const streams = Array.from(activeStreams().values());
    return {
      totalStreams: streams.length,
      totalJobs: streams.reduce((sum, s) => sum + s.totalJobs, 0),
      activeJobs: streams.reduce((sum, s) => sum + s.activeJobs, 0),
      completedJobs: streams.reduce((sum, s) => sum + s.completedJobs, 0),
      failedJobs: streams.reduce((sum, s) => sum + s.failedJobs, 0),
      totalBytes: streams.reduce((sum, s) => sum + s.totalBytes, 0),
      extractedBytes: streams.reduce((sum, s) => sum + s.extractedBytes, 0),
      overallThroughput: streams.reduce((sum, s) => sum + s.overallThroughputMbps, 0),
    };
  });

  // Helper formatters
  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
  };

  const formatTime = (seconds: number): string => {
    if (seconds < 60) return `${seconds}s`;
    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = seconds % 60;
    if (minutes < 60) return `${minutes}m ${remainingSeconds}s`;
    const hours = Math.floor(minutes / 60);
    const remainingMinutes = minutes % 60;
    return `${hours}h ${remainingMinutes}m`;
  };

  const formatThroughput = (mbps: number): string => {
    if (mbps < 1) return `${(mbps * 1024).toFixed(2)} KB/s`;
    return `${mbps.toFixed(2)} MB/s`;
  };

  return {
    // State
    initialized,
    activeStreams,
    fileAvailable,
    error,
    overallStats,

    // Actions
    startStream,
    getProgress,
    cancelStream,
    getActiveStreamIds,
    getJobStatuses,

    // Formatters
    formatBytes,
    formatTime,
    formatThroughput,
  };
}
