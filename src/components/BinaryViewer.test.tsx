// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, beforeEach } from "vitest";
import { render } from "solid-js/web";
import { BinaryViewer } from "./BinaryViewer";
import { mockInvoke } from "../__tests__/setup";

// Helper to render and return the container
function renderComponent(component: () => any) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const dispose = render(component, container);
  return { container, dispose };
}

// Wait for async updates
const tick = (ms = 50) => new Promise(resolve => setTimeout(resolve, ms));

// Mock PE binary data (matches BinaryInfo interface)
const mockPeData = {
  path: "/tmp/program.exe",
  format: "PE64",
  architecture: "x86_64",
  is_64bit: true,
  entry_point: 4096,
  file_size: 102400,
  is_stripped: false,
  has_debug_info: true,
  has_code_signing: false,
  pe_timestamp: 1705312200,
  pe_checksum: 54321,
  pe_subsystem: "WindowsConsole",
  macho_cpu_type: null,
  macho_filetype: null,
  sections: [
    { name: ".text", virtual_address: 4096, virtual_size: 32768, raw_size: 32768, characteristics: "CNT_CODE | MEM_EXECUTE | MEM_READ" },
    { name: ".rdata", virtual_address: 40960, virtual_size: 8192, raw_size: 8192, characteristics: "CNT_INITIALIZED_DATA | MEM_READ" },
    { name: ".data", virtual_address: 49152, virtual_size: 4096, raw_size: 2048, characteristics: "CNT_INITIALIZED_DATA | MEM_READ | MEM_WRITE" },
  ],
  imports: [
    { library: "KERNEL32.dll", functions: ["CreateFileW", "ReadFile", "CloseHandle", "GetLastError"], function_count: 4 },
    { library: "USER32.dll", functions: ["MessageBoxW", "GetWindowTextW"], function_count: 2 },
  ],
  exports: [
    { name: "DllMain", ordinal: 1, address: 4096 },
    { name: "PluginInit", ordinal: 2, address: 4200 },
  ],
};

// Mock ELF binary data
const mockElfData = {
  path: "/tmp/program.elf",
  format: "ELF64",
  architecture: "x86_64",
  is_64bit: true,
  entry_point: 4096,
  file_size: 81920,
  is_stripped: true,
  has_debug_info: false,
  has_code_signing: false,
  pe_timestamp: null,
  pe_checksum: null,
  pe_subsystem: null,
  macho_cpu_type: null,
  macho_filetype: null,
  sections: [
    { name: ".text", virtual_address: 4096, virtual_size: 16384, raw_size: 16384, characteristics: "ALLOC | EXECINSTR" },
  ],
  imports: [],
  exports: [],
};

// Mock Mach-O data
const mockMachoData = {
  path: "/tmp/program.dylib",
  format: "MachO64",
  architecture: "ARM64",
  is_64bit: true,
  entry_point: null,
  file_size: 65536,
  is_stripped: false,
  has_debug_info: true,
  has_code_signing: true,
  pe_timestamp: null,
  pe_checksum: null,
  pe_subsystem: null,
  macho_cpu_type: "ARM64",
  macho_filetype: "Execute",
  sections: [],
  imports: [],
  exports: [],
};

describe("BinaryViewer", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
    mockInvoke.mockReset();
  });

  describe("PE binary rendering", () => {
    it("renders PE format badge and overview", async () => {
      mockInvoke.mockResolvedValueOnce(mockPeData);

      const { container } = renderComponent(() => (
        <BinaryViewer path="/tmp/program.exe" />
      ));
      await tick();

      expect(container.textContent).toContain("PE64");
      expect(container.textContent).toContain("x86_64");
    });

    it("calls binary_analyze with the file path", async () => {
      mockInvoke.mockResolvedValueOnce(mockPeData);

      renderComponent(() => <BinaryViewer path="/tmp/program.exe" />);
      await tick();

      expect(mockInvoke).toHaveBeenCalledWith("binary_analyze", { path: "/tmp/program.exe" });
    });

    it("shows PE-specific information", async () => {
      mockInvoke.mockResolvedValueOnce(mockPeData);

      const { container } = renderComponent(() => (
        <BinaryViewer path="/tmp/program.exe" />
      ));
      await tick();

      expect(container.textContent).toContain("WindowsConsole");
    });

    it("renders sections table", async () => {
      mockInvoke.mockResolvedValueOnce(mockPeData);

      const { container } = renderComponent(() => (
        <BinaryViewer path="/tmp/program.exe" />
      ));
      await tick();

      expect(container.textContent).toContain(".text");
      expect(container.textContent).toContain(".rdata");
      expect(container.textContent).toContain(".data");
    });

    it("renders imports list", async () => {
      mockInvoke.mockResolvedValueOnce(mockPeData);

      const { container } = renderComponent(() => (
        <BinaryViewer path="/tmp/program.exe" />
      ));
      await tick();

      expect(container.textContent).toContain("KERNEL32.dll");
      expect(container.textContent).toContain("USER32.dll");
    });

    it("renders exports section header", async () => {
      mockInvoke.mockResolvedValueOnce(mockPeData);

      const { container } = renderComponent(() => (
        <BinaryViewer path="/tmp/program.exe" />
      ));
      await tick();

      // Exports section exists with count (collapsed by default)
      expect(container.textContent).toContain("Exports");
      expect(container.textContent).toContain("2");
    });

    it("shows security indicators", async () => {
      mockInvoke.mockResolvedValueOnce(mockPeData);

      const { container } = renderComponent(() => (
        <BinaryViewer path="/tmp/program.exe" />
      ));
      await tick();

      // Debug info present, not stripped
      expect(container.textContent).toContain("Debug");
    });
  });

  describe("ELF binary rendering", () => {
    it("renders ELF format information", async () => {
      mockInvoke.mockResolvedValueOnce(mockElfData);

      const { container } = renderComponent(() => (
        <BinaryViewer path="/tmp/program.elf" />
      ));
      await tick();

      expect(container.textContent).toContain("ELF64");
      expect(container.textContent).toContain("x86_64");
    });

    it("shows stripped status", async () => {
      mockInvoke.mockResolvedValueOnce(mockElfData);

      const { container } = renderComponent(() => (
        <BinaryViewer path="/tmp/program.elf" />
      ));
      await tick();

      expect(container.textContent).toContain("Stripped");
    });
  });

  describe("Mach-O binary rendering", () => {
    it("renders Mach-O format information", async () => {
      mockInvoke.mockResolvedValueOnce(mockMachoData);

      const { container } = renderComponent(() => (
        <BinaryViewer path="/tmp/program.dylib" />
      ));
      await tick();

      expect(container.textContent).toContain("MachO64");
      expect(container.textContent).toContain("ARM64");
    });

    it("shows code signing status", async () => {
      mockInvoke.mockResolvedValueOnce(mockMachoData);

      const { container } = renderComponent(() => (
        <BinaryViewer path="/tmp/program.dylib" />
      ));
      await tick();

      expect(container.textContent).toContain("Code Signed");
    });
  });

  describe("Loading and error states", () => {
    it("shows loading state initially", () => {
      mockInvoke.mockReturnValue(new Promise(() => {}));

      const { container } = renderComponent(() => (
        <BinaryViewer path="/tmp/program.exe" />
      ));

      expect(container.textContent).toContain("Analyzing");
    });

    it("shows error when analysis fails", async () => {
      mockInvoke.mockRejectedValueOnce(new Error("Unknown binary format"));

      const { container } = renderComponent(() => (
        <BinaryViewer path="/tmp/unknown.bin" />
      ));
      await tick();

      expect(container.textContent).toContain("Unknown binary format");
    });
  });

  describe("Edge cases", () => {
    it("handles binary with no imports or exports", async () => {
      mockInvoke.mockResolvedValueOnce(mockElfData);

      const { container } = renderComponent(() => (
        <BinaryViewer path="/tmp/static.elf" />
      ));
      await tick();

      // Should render without crashing
      expect(container.textContent).toContain("ELF64");
    });

    it("handles binary with no sections", async () => {
      mockInvoke.mockResolvedValueOnce(mockMachoData);

      const { container } = renderComponent(() => (
        <BinaryViewer path="/tmp/minimal.dylib" />
      ));
      await tick();

      // Should render without crashing
      expect(container.innerHTML).toBeTruthy();
    });
  });
});
