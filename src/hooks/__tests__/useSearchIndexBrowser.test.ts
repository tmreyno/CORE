import { createRoot, createSignal } from "solid-js";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { mockInvoke } from "../../__tests__/setup";
import { useSearchIndex } from "../useSearchIndex";

vi.mock("../../utils/platform", () => ({
  isTauri: false,
}));

const tick = () => new Promise((resolve) => setTimeout(resolve, 0));

describe("useSearchIndex browser runtime guard", () => {
  beforeEach(() => {
    mockInvoke.mockClear();
  });

  it("does not open or auto-index a native search index", async () => {
    let hook!: ReturnType<typeof useSearchIndex>;
    let dispose!: () => void;
    const [hasProject, setHasProject] = createSignal(false);
    const [paths, setPaths] = createSignal<string[]>([]);

    createRoot((d) => {
      dispose = d;
      hook = useSearchIndex({
        hasProject,
        projectPath: () => "/case/project.cffx",
        discoveredFilePaths: paths,
      });
    });

    setHasProject(true);
    setPaths(["/case/disk.E01"]);
    await tick();
    await tick();

    await hook.indexSingleContainer("/case/disk.E01");
    await hook.indexAllDiscovered();
    await hook.rebuildIndex();
    await hook.refreshStats();

    expect(hook.indexReady()).toBe(false);
    expect(hook.indexing()).toBe(false);
    expect(hook.stats()).toBeNull();
    expect(mockInvoke).not.toHaveBeenCalled();
    dispose();
  });
});
