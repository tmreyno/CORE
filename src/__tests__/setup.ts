// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { vi, beforeEach, afterEach, expect } from "vitest";

// Custom matcher for toBeInTheDocument (simplified version)
expect.extend({
  toBeInTheDocument(received) {
    const pass = received !== null && received !== undefined && document.body.contains(received);
    return {
      pass,
      message: () => pass 
        ? `expected element not to be in the document`
        : `expected element to be in the document`,
    };
  },
  toHaveAttribute(received, attr: string, value?: string) {
    const hasAttr = received?.hasAttribute?.(attr);
    const attrValue = received?.getAttribute?.(attr);
    const pass = value !== undefined ? attrValue === value : hasAttr;
    return {
      pass,
      message: () => pass
        ? `expected element not to have attribute "${attr}"${value !== undefined ? ` with value "${value}"` : ''}`
        : `expected element to have attribute "${attr}"${value !== undefined ? ` with value "${value}"` : ''}, but got "${attrValue}"`,
    };
  },
  toHaveStyle(received, styles: Record<string, string>) {
    const computedStyle = window.getComputedStyle(received);
    const pass = Object.entries(styles).every(([prop, value]) => {
      return computedStyle.getPropertyValue(prop) === value || received.style[prop] === value;
    });
    return {
      pass,
      message: () => pass
        ? `expected element not to have specified styles`
        : `expected element to have specified styles`,
    };
  },
});

// Mock Tauri APIs for testing
const mockInvoke = vi.fn();
const mockListen = vi.fn(() => Promise.resolve(() => {}));
const mockEmit = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mockInvoke,
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: mockListen,
  emit: mockEmit,
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(),
  save: vi.fn(),
  message: vi.fn(),
  ask: vi.fn(),
  confirm: vi.fn(),
}));

// Export mocks for use in tests
export { mockInvoke, mockListen, mockEmit };

// Reset mocks before each test
beforeEach(() => {
  mockInvoke.mockReset().mockResolvedValue(undefined);
  mockListen.mockReset().mockReturnValue(Promise.resolve(() => {}));
  mockEmit.mockReset();
});

// Clean up after each test
afterEach(() => {
  vi.clearAllMocks();
});
