import { Component, Show, For, createSignal } from "solid-js";
import { useStreamingExtractor, type StreamProgress, type JobStatus } from "../hooks/useStreamingExtractor";
import { HiOutlineArrowPath, HiOutlineXMark, HiOutlineCheckCircle, HiOutlineExclamationCircle } from "solid-icons/hi";

export const StreamingExtractionPanel: Component = () => {
  const extractor = useStreamingExtractor();
  const [selectedStream, setSelectedStream] = createSignal<string | null>(null);
  const [jobStatuses, setJobStatuses] = createSignal<JobStatus[]>([]);

  const handleViewDetails = async (streamId: string) => {
    setSelectedStream(streamId);
    const statuses = await extractor.getJobStatuses(streamId);
    setJobStatuses(statuses);
  };

  const handleCancelStream = async (streamId: string) => {
    await extractor.cancelStream(streamId);
    setSelectedStream(null);
  };

  return (
    <div class="flex flex-col h-full bg-bg">
      {/* Header */}
      <div class="flex items-center justify-between p-4 border-b border-border">
        <div class="flex items-center gap-small">
          <HiOutlineArrowPath class="w-icon-lg h-icon-lg text-accent" />
          <h2 class="text-lg font-semibold text-txt">Streaming Extraction</h2>
        </div>
      </div>

      {/* Error Display */}
      <Show when={extractor.error()}>
        <div class="m-4 p-3 bg-error/10 border border-error/20 rounded-md">
          <p class="text-error text-sm">{extractor.error()}</p>
        </div>
      </Show>

      {/* Overall Stats */}
      <Show when={extractor.overallStats().totalStreams > 0}>
        <div class="grid grid-cols-4 gap-4 p-4 border-b border-border">
          <div class="bg-bg-panel rounded-md p-3 border border-border">
            <div class="text-xs text-txt-muted uppercase tracking-wide mb-1">Active Streams</div>
            <div class="text-2xl font-bold text-accent">{extractor.overallStats().totalStreams}</div>
          </div>

          <div class="bg-bg-panel rounded-md p-3 border border-border">
            <div class="text-xs text-txt-muted uppercase tracking-wide mb-1">Active Jobs</div>
            <div class="text-2xl font-bold text-warning">{extractor.overallStats().activeJobs}</div>
            <div class="text-xs text-txt-secondary mt-1">
              {extractor.overallStats().completedJobs} / {extractor.overallStats().totalJobs} complete
            </div>
          </div>

          <div class="bg-bg-panel rounded-md p-3 border border-border">
            <div class="text-xs text-txt-muted uppercase tracking-wide mb-1">Data Extracted</div>
            <div class="text-2xl font-bold text-success">
              {extractor.formatBytes(extractor.overallStats().extractedBytes)}
            </div>
            <div class="text-xs text-txt-secondary mt-1">
              of {extractor.formatBytes(extractor.overallStats().totalBytes)}
            </div>
          </div>

          <div class="bg-bg-panel rounded-md p-3 border border-border">
            <div class="text-xs text-txt-muted uppercase tracking-wide mb-1">Throughput</div>
            <div class="text-2xl font-bold text-txt">
              {extractor.formatThroughput(extractor.overallStats().overallThroughput)}
            </div>
          </div>
        </div>
      </Show>

      {/* File Available Notification */}
      <Show when={extractor.fileAvailable()}>
        {(file) => (
          <div class="m-4 p-3 bg-success/10 border border-success/20 rounded-md flex items-center gap-small">
            <HiOutlineCheckCircle class="w-icon-base h-icon-base text-success flex-shrink-0" />
            <div class="flex-1 min-w-0">
              <p class="text-success text-sm font-medium">File Available</p>
              <p class="text-txt-secondary text-xs truncate">{file().filePath}</p>
            </div>
            <div class="text-xs text-txt-muted">
              {extractor.formatBytes(file().sizeBytes)}
            </div>
          </div>
        )}
      </Show>

      {/* Active Streams List */}
      <div class="flex-1 overflow-y-auto p-4">
        <Show
          when={extractor.activeStreams().size > 0}
          fallback={
            <div class="flex flex-col items-center justify-center h-full text-txt-muted">
              <HiOutlineArrowPath class="w-16 h-16 mb-4 opacity-50" />
              <p class="text-lg">No active streams</p>
              <p class="text-sm mt-2">Start a streaming extraction to see progress</p>
            </div>
          }
        >
          <div class="space-y-3">
            <For each={Array.from(extractor.activeStreams().entries())}>
              {([streamId, progress]) => (
                <StreamCard
                  streamId={streamId}
                  progress={progress}
                  isSelected={selectedStream() === streamId}
                  jobStatuses={selectedStream() === streamId ? jobStatuses() : []}
                  onViewDetails={() => handleViewDetails(streamId)}
                  onCancel={() => handleCancelStream(streamId)}
                  extractor={extractor}
                />
              )}
            </For>
          </div>
        </Show>
      </div>
    </div>
  );
};

// Stream card component
const StreamCard: Component<{
  streamId: string;
  progress: StreamProgress;
  isSelected: boolean;
  jobStatuses: JobStatus[];
  onViewDetails: () => void;
  onCancel: () => void;
  extractor: ReturnType<typeof useStreamingExtractor>;
}> = (props) => {
  const percentComplete = () => {
    if (props.progress.totalBytes === 0) return 0;
    return (props.progress.extractedBytes / props.progress.totalBytes) * 100;
  };

  const isComplete = () =>
    props.progress.completedJobs + props.progress.failedJobs === props.progress.totalJobs;

  return (
    <div class="bg-bg-panel rounded-md border border-border overflow-hidden">
      {/* Card Header */}
      <div class="p-3 border-b border-border">
        <div class="flex items-center justify-between mb-2">
          <div class="flex items-center gap-small">
            <span class="text-sm font-medium text-txt">Stream {props.streamId.slice(0, 8)}...</span>
            <Show when={isComplete()}>
              <span class="px-2 py-0.5 bg-success/20 text-success rounded text-[10px] font-medium">
                COMPLETE
              </span>
            </Show>
          </div>

          <div class="flex items-center gap-compact">
            <button
              onClick={props.onViewDetails}
              class="px-2 py-1 bg-bg-secondary text-txt hover:bg-bg-hover rounded text-xs border border-border"
            >
              {props.isSelected ? "Hide Details" : "View Details"}
            </button>
            <Show when={!isComplete()}>
              <button
                onClick={props.onCancel}
                class="p-1 text-error hover:bg-error/10 rounded"
              >
                <HiOutlineXMark class="w-icon-sm h-icon-sm" />
              </button>
            </Show>
          </div>
        </div>

        {/* Progress Stats */}
        <div class="grid grid-cols-4 gap-4 text-xs">
          <div>
            <span class="text-txt-muted">Jobs:</span>
            <span class="ml-1 text-txt font-medium">
              {props.progress.completedJobs} / {props.progress.totalJobs}
            </span>
          </div>
          <div>
            <span class="text-txt-muted">Active:</span>
            <span class="ml-1 text-warning font-medium">{props.progress.activeJobs}</span>
          </div>
          <div>
            <span class="text-txt-muted">Failed:</span>
            <span class="ml-1 text-error font-medium">{props.progress.failedJobs}</span>
          </div>
          <div>
            <span class="text-txt-muted">Throughput:</span>
            <span class="ml-1 text-txt font-medium">
              {props.extractor.formatThroughput(props.progress.overallThroughputMbps)}
            </span>
          </div>
        </div>
      </div>

      {/* Progress Bar */}
      <div class="px-3 py-2 bg-bg">
        <div class="flex items-center justify-between mb-1 text-xs">
          <span class="text-txt-secondary">
            {props.extractor.formatBytes(props.progress.extractedBytes)} / {props.extractor.formatBytes(props.progress.totalBytes)}
          </span>
          <span class="text-txt">{percentComplete().toFixed(1)}%</span>
        </div>
        <div class="w-full bg-bg-secondary rounded-full h-2">
          <div
            class="bg-accent h-2 rounded-full transition-all duration-300"
            style={{ width: `${percentComplete()}%` }}
          />
        </div>
        <Show when={props.progress.etaSeconds}>
          <div class="text-xs text-txt-muted mt-1">
            ETA: {props.extractor.formatTime(props.progress.etaSeconds!)}
          </div>
        </Show>
      </div>

      {/* Job Details (expanded) */}
      <Show when={props.isSelected && props.jobStatuses.length > 0}>
        <div class="border-t border-border">
          <div class="max-h-64 overflow-y-auto">
            <For each={props.jobStatuses}>
              {(job) => (
                <div class="flex items-center justify-between p-2 px-3 text-xs border-b border-border last:border-b-0 hover:bg-bg-hover">
                  <div class="flex items-center gap-small flex-1 min-w-0">
                    <JobStatusIcon status={job.status} />
                    <span class="font-mono text-txt-secondary truncate text-[10px]">
                      {job.id.slice(0, 12)}...
                    </span>
                  </div>

                  <div class="flex items-center gap-4 text-txt-muted">
                    <span>{props.extractor.formatBytes(job.totalBytes)}</span>
                    <Show when={job.throughputMbps > 0}>
                      <span>{props.extractor.formatThroughput(job.throughputMbps)}</span>
                    </Show>
                    <Show when={job.elapsedMs > 0}>
                      <span>{(job.elapsedMs / 1000).toFixed(1)}s</span>
                    </Show>
                  </div>

                  <Show when={job.error}>
                    <div class="ml-2 text-error text-[10px] truncate max-w-32">
                      {job.error}
                    </div>
                  </Show>
                </div>
              )}
            </For>
          </div>
        </div>
      </Show>
    </div>
  );
};

// Job status icon
const JobStatusIcon: Component<{ status: string }> = (props) => {
  switch (props.status) {
    case "complete":
      return <HiOutlineCheckCircle class="w-icon-sm h-icon-sm text-success" />;
    case "failed":
      return <HiOutlineExclamationCircle class="w-icon-sm h-icon-sm text-error" />;
    case "extracting":
    case "verifying":
      return <HiOutlineArrowPath class="w-icon-sm h-icon-sm text-warning animate-spin" />;
    default:
      return <div class="w-icon-sm h-icon-sm rounded-full border-2 border-txt-muted" />;
  }
};
