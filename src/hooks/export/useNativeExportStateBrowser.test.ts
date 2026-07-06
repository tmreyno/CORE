import { createRoot, createSignal } from "solid-js";
import { afterEach, describe, expect, it, vi } from "vitest";
import { useNativeExportState } from "./useNativeExportState";
import { DESKTOP_EXPORT_ENGINE_MESSAGE } from "./desktopRuntimeGuard";
import type { ToolsTabId } from "../../components/export/ToolsMode";

vi.mock("../../api/archiveCreate", () => ({
  createArchive: vi.fn(),
  listenToProgress: vi.fn(),
  estimateSize: vi.fn(),
  formatBytes: (value: number) => `${value} B`,
  CompressionLevel: { Store: 0 },
  testArchive: vi.fn(),
  repairArchive: vi.fn(),
  validateArchive: vi.fn(),
  extractSplitArchive: vi.fn(),
  listenToRepairProgress: vi.fn(),
  listenToSplitExtractProgress: vi.fn(),
}));

vi.mock("../../api/lzmaApi", () => ({
  compressToLzma: vi.fn(),
  compressToLzma2: vi.fn(),
  decompressLzma: vi.fn(),
  decompressLzma2: vi.fn(),
}));

vi.mock("../../api/fileExport", () => ({
  exportFiles: vi.fn(),
  cancelExport: vi.fn(),
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

describe("useNativeExportState in browser preview", () => {
  afterEach(() => {
    vi.clearAllMocks();
  });

  it("blocks native export and archive tool engines without invoking desktop APIs", async () => {
    const archiveApi = await import("../../api/archiveCreate");
    const lzmaApi = await import("../../api/lzmaApi");
    const fileExportApi = await import("../../api/fileExport");

    await createRoot(async (dispose) => {
      const [sources, setSources] = createSignal(["/typed/source.E01"]);
      const [destination, setDestination] = createSignal("/typed/out");
      const [mode, setMode] = createSignal("native");
      const [nativeExportTab, setNativeExportTab] = createSignal("archive");
      const [processing, setProcessing] = createSignal(false);
      const [acquiring, setAcquiring] = createSignal(false);

      const toast = {
        success: vi.fn(),
        error: vi.fn(),
        warning: vi.fn(),
        info: vi.fn(),
      };

      const state = useNativeExportState({
        toast,
        common: {
          sources,
          setSources,
          destination,
          setDestination,
          mode,
          setMode,
          nativeExportTab,
          setNativeExportTab,
          isProcessing: processing,
          setIsProcessing: setProcessing,
          isAcquiring: acquiring,
          setIsAcquiring: setAcquiring,
          clearAllSources: vi.fn(),
        } as any,
      });

      await state.handleCreateArchive();
      await state.handleCopyOrExport();

      const tabs: ToolsTabId[] = ["test", "repair", "validate", "extract", "compress", "decompress"];
      for (const tab of tabs) {
        state.setToolsTab(tab);
        await state.handleToolAction();
      }

      expect(toast.error).toHaveBeenCalledTimes(8);
      expect(toast.error).toHaveBeenCalledWith(
        "Desktop Runtime Required",
        DESKTOP_EXPORT_ENGINE_MESSAGE,
      );

      expect(archiveApi.estimateSize).not.toHaveBeenCalled();
      expect(archiveApi.createArchive).not.toHaveBeenCalled();
      expect(archiveApi.listenToProgress).not.toHaveBeenCalled();
      expect(archiveApi.testArchive).not.toHaveBeenCalled();
      expect(archiveApi.repairArchive).not.toHaveBeenCalled();
      expect(archiveApi.validateArchive).not.toHaveBeenCalled();
      expect(archiveApi.extractSplitArchive).not.toHaveBeenCalled();
      expect(archiveApi.listenToRepairProgress).not.toHaveBeenCalled();
      expect(archiveApi.listenToSplitExtractProgress).not.toHaveBeenCalled();
      expect(fileExportApi.exportFiles).not.toHaveBeenCalled();
      expect(lzmaApi.compressToLzma).not.toHaveBeenCalled();
      expect(lzmaApi.compressToLzma2).not.toHaveBeenCalled();
      expect(lzmaApi.decompressLzma).not.toHaveBeenCalled();
      expect(lzmaApi.decompressLzma2).not.toHaveBeenCalled();
      expect(processing()).toBe(false);
      expect(acquiring()).toBe(false);

      dispose();
    });
  });
});
