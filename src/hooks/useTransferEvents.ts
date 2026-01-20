// =============================================================================
// useTransferEvents - Transfer progress event listeners
// =============================================================================

import { onMount, onCleanup, Setter } from "solid-js";
import { onTransferProgress, onTransferComplete } from "../transfer";
import type { TransferJob } from "../components";

/**
 * Hook to manage transfer progress and completion event listeners
 * Sets up global event listeners that persist across tab switches
 */
export function useTransferEvents(setTransferJobs: Setter<TransferJob[]>): void {
  let unsubProgress: (() => void) | null = null;
  let unsubComplete: (() => void) | null = null;

  onMount(async () => {
    // Listen for transfer progress events
    unsubProgress = await onTransferProgress((progress) => {
      setTransferJobs(jobs => jobs.map(job => 
        job.id === progress.operation_id 
          ? { ...job, progress, status: "running" as const } 
          : job
      ));
    });
    
    // Listen for transfer complete events
    unsubComplete = await onTransferComplete((result) => {
      setTransferJobs(jobs => jobs.map(job => {
        if (job.id === result.operation_id) {
          return {
            ...job,
            status: result.success ? "completed" as const : "failed" as const,
            result,
            endTime: new Date(),
          };
        }
        return job;
      }));
    });
  });

  onCleanup(() => {
    unsubProgress?.();
    unsubComplete?.();
  });
}
