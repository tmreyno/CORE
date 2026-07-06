import { createRoot, createSignal } from "solid-js";
import { afterEach, describe, expect, it, vi } from "vitest";
import { useAff4ExportState } from "./useAff4ExportState";
import { DESKTOP_EXPORT_ENGINE_MESSAGE } from "./desktopRuntimeGuard";
import { useEwfExportState } from "./useEwfExportState";
import { useL01ExportState } from "./useL01ExportState";
import { useMemoryDumpState } from "./useMemoryDumpState";
import { useRawExportState } from "./useRawExportState";
import { useTriageState } from "./useTriageState";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

vi.mock("../../api/archiveCreate", () => ({
  formatBytes: (value: number) => `${value} B`,
}));

vi.mock("../../api/ewfExport", () => ({
  createE01Image: vi.fn(),
  buildEwfExportOptions: vi.fn((options) => options),
}));

vi.mock("../../api/l01Export", () => ({
  createL01Image: vi.fn(),
  buildL01ExportOptions: vi.fn((options) => options),
}));

vi.mock("../../api/rawExport", () => ({
  createRawImage: vi.fn(),
  buildRawExportOptions: vi.fn((options) => options),
}));

vi.mock("../../api/aff4Export", () => ({
  createAff4Image: vi.fn(),
}));

vi.mock("../../api/memory", () => ({
  getMemoryCaptureInfo: vi.fn(),
  captureMemory: vi.fn(),
  cancelMemoryCapture: vi.fn(),
  listenMemoryCaptureProgress: vi.fn(),
}));

vi.mock("../../api/triage", () => ({
  getTriageProfiles: vi.fn(),
  triageCollect: vi.fn(),
  triageCancel: vi.fn(),
  listenTriageProgress: vi.fn(),
}));

vi.mock("../project/useProjectDbSync", () => ({
  dbSync: {
    insertExport: vi.fn(),
    updateExport: vi.fn(),
  },
}));

vi.mock("./companionHelper", () => ({
  startAcquisitionRecord: vi.fn(() => ({ collectionId: "collection-1", itemId: "item-1" })),
  handleAcquisitionComplete: vi.fn(),
}));

function createCommonState() {
  const [sources, setSources] = createSignal(["/typed/source.E01"]);
  const [destination, setDestination] = createSignal("/typed/out");
  const [processing, setProcessing] = createSignal(false);
  const [acquiring, setAcquiring] = createSignal(false);

  return {
    sources,
    setSources,
    destination,
    setDestination,
    isProcessing: processing,
    setIsProcessing: setProcessing,
    isAcquiring: acquiring,
    setIsAcquiring: setAcquiring,
    hasDriveSources: () => false,
    mountDrivesReadOnly: () => false,
    restoreAllDriveMounts: vi.fn(),
    clearAllSources: vi.fn(),
    processing,
    acquiring,
  };
}

describe("acquisition export engines in browser preview", () => {
  afterEach(() => {
    vi.clearAllMocks();
  });

  it("blocks native imaging, memory, and triage commands before desktop APIs are invoked", async () => {
    const ewfApi = await import("../../api/ewfExport");
    const l01Api = await import("../../api/l01Export");
    const rawApi = await import("../../api/rawExport");
    const aff4Api = await import("../../api/aff4Export");
    const memoryApi = await import("../../api/memory");
    const triageApi = await import("../../api/triage");

    await createRoot(async (dispose) => {
      const common = createCommonState();
      const toast = {
        success: vi.fn(),
        error: vi.fn(),
        warning: vi.fn(),
        info: vi.fn(),
      };
      const options = { toast, common: common as any };

      const ewf = useEwfExportState(options);
      const l01 = useL01ExportState(options);
      const raw = useRawExportState(options);
      const aff4 = useAff4ExportState(options);
      const memory = useMemoryDumpState(options);
      const triage = useTriageState(options);

      await ewf.handleCreateE01Image();
      await l01.handleCreateL01Image();
      await raw.handleCreateRawImage();
      await aff4.handleCreateAff4Image();
      await memory.loadMemoryInfo();
      await memory.handleCaptureMemory();
      await memory.handleCancelMemoryCapture();
      await triage.loadTriageProfiles();
      await triage.handleTriageCollect();
      await triage.handleCancelTriage();

      expect(toast.error).toHaveBeenCalledTimes(10);
      expect(toast.error).toHaveBeenCalledWith(
        "Desktop Runtime Required",
        DESKTOP_EXPORT_ENGINE_MESSAGE,
      );

      expect(ewfApi.createE01Image).not.toHaveBeenCalled();
      expect(l01Api.createL01Image).not.toHaveBeenCalled();
      expect(rawApi.createRawImage).not.toHaveBeenCalled();
      expect(aff4Api.createAff4Image).not.toHaveBeenCalled();
      expect(memoryApi.getMemoryCaptureInfo).not.toHaveBeenCalled();
      expect(memoryApi.captureMemory).not.toHaveBeenCalled();
      expect(memoryApi.cancelMemoryCapture).not.toHaveBeenCalled();
      expect(memoryApi.listenMemoryCaptureProgress).not.toHaveBeenCalled();
      expect(triageApi.getTriageProfiles).not.toHaveBeenCalled();
      expect(triageApi.triageCollect).not.toHaveBeenCalled();
      expect(triageApi.triageCancel).not.toHaveBeenCalled();
      expect(triageApi.listenTriageProgress).not.toHaveBeenCalled();
      expect(common.processing()).toBe(false);
      expect(common.acquiring()).toBe(false);

      dispose();
    });
  });
});
