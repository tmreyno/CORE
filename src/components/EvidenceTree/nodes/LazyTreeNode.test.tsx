// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render } from "solid-js/web";
import { LazyTreeNode } from "./LazyTreeNode";
import type { NestedContainerEntry } from "../../../types";
import type { LazyTreeEntry } from "../../../types/lazy-loading";

function renderComponent(component: () => any) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const dispose = render(component, container);
  return { container, dispose };
}

function makeLazyEntry(overrides: Partial<LazyTreeEntry> = {}): LazyTreeEntry {
  return {
    id: "entry-1",
    name: "nested.zip",
    path: "nested.zip",
    is_dir: false,
    size: 1024,
    entry_type: "file",
    child_count: 0,
    children_loaded: false,
    hash: null,
    modified: null,
    metadata: null,
    ...overrides,
  };
}

function makeNestedEntry(overrides: Partial<NestedContainerEntry> = {}): NestedContainerEntry {
  return {
    path: "inside.txt",
    name: "inside.txt",
    isDir: false,
    size: 512,
    hash: null,
    modified: null,
    sourceType: "zip",
    isNestedContainer: false,
    nestedType: null,
    ...overrides,
  };
}

describe("LazyTreeNode", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
  });

  function baseProps(overrides: Partial<Parameters<typeof LazyTreeNode>[0]> = {}) {
    return {
      entry: makeLazyEntry(),
      containerPath: "/evidence/sample.l01",
      depth: 0,
      isExpanded: vi.fn().mockReturnValue(false),
      isLoading: vi.fn().mockReturnValue(false),
      isSelected: vi.fn().mockReturnValue(false),
      getChildren: vi.fn().mockReturnValue([]),
      hasMoreChildren: vi.fn().mockReturnValue(false),
      getLoadedCount: vi.fn().mockReturnValue(0),
      getTotalCount: vi.fn().mockReturnValue(0),
      onToggle: vi.fn(),
      onClick: vi.fn(),
      onLoadMore: vi.fn(),
      isNestedExpanded: vi.fn().mockReturnValue(false),
      isNestedLoading: vi.fn().mockReturnValue(false),
      getNestedEntries: vi.fn().mockReturnValue([]),
      getNestedChildren: vi.fn().mockReturnValue([]),
      onToggleNested: vi.fn().mockResolvedValue(undefined),
      onNestedClick: vi.fn(),
      ...overrides,
    };
  }

  it("uses lazy-prefixed keys for selection and loading", () => {
    const isSelected = vi.fn().mockReturnValue(false);
    const isLoading = vi.fn().mockReturnValue(false);
    const props = baseProps({ isSelected, isLoading });

    const { dispose } = renderComponent(() => <LazyTreeNode {...props} />);

    expect(isSelected).toHaveBeenCalledWith("/evidence/sample.l01::lazy::nested.zip");
    expect(isLoading).toHaveBeenCalledWith("/evidence/sample.l01::lazy::nested.zip");

    dispose();
  });

  it("renders nested container contents for expanded lazy-tree container files", () => {
    const nestedEntries = [makeNestedEntry({ name: "inside.txt", path: "inside.txt" })];
    const props = baseProps({
      isNestedExpanded: vi.fn().mockReturnValue(true),
      getNestedEntries: vi.fn().mockReturnValue(nestedEntries),
    });

    const { container, dispose } = renderComponent(() => <LazyTreeNode {...props} />);

    expect(container.textContent).toContain("nested.zip");
    expect(container.textContent).toContain("inside.txt");

    dispose();
  });

  it("routes expand toggles for nested container files through onToggleNested", () => {
    const onToggle = vi.fn();
    const onToggleNested = vi.fn().mockResolvedValue(undefined);
    const props = baseProps({ onToggle, onToggleNested });

    const { container, dispose } = renderComponent(() => <LazyTreeNode {...props} />);
    const toggle = container.querySelector('[aria-hidden="true"]');

    expect(toggle).toBeTruthy();
    toggle!.dispatchEvent(new MouseEvent("click", { bubbles: true }));

    expect(onToggleNested).toHaveBeenCalledWith("/evidence/sample.l01", "nested.zip");
    expect(onToggle).not.toHaveBeenCalled();

    dispose();
  });
});