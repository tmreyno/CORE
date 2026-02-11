// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, beforeEach } from "vitest";
import { render } from "solid-js/web";
import { EmailViewer } from "./EmailViewer";
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

// Mock email data
const mockEmlData = {
  path: "/tmp/test.eml",
  message_id: "<abc123@example.com>",
  subject: "Test Subject",
  from: [{ name: "Alice", address: "alice@example.com" }],
  to: [{ name: "Bob", address: "bob@example.com" }],
  cc: [{ name: null, address: "cc@example.com" }],
  bcc: [],
  date: "2024-01-15T10:30:00Z",
  body_text: "Hello, this is a test email.",
  body_html: null,
  attachments: [
    { filename: "report.pdf", content_type: "application/pdf", size: 1024, is_inline: false },
  ],
  headers: [
    { name: "Return-Path", value: "alice@example.com" },
    { name: "X-Mailer", value: "TestMailer 1.0" },
  ],
  size: 2048,
};

const mockMboxData = [
  {
    ...mockEmlData,
    subject: "First Message",
    message_id: "<msg1@example.com>",
  },
  {
    ...mockEmlData,
    subject: "Second Message",
    message_id: "<msg2@example.com>",
    from: [{ name: "Charlie", address: "charlie@example.com" }],
  },
];

describe("EmailViewer", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
    mockInvoke.mockReset();
  });

  describe("EML file rendering", () => {
    it("renders email subject and sender for .eml files", async () => {
      mockInvoke.mockResolvedValueOnce(mockEmlData);

      const { container } = renderComponent(() => (
        <EmailViewer path="/tmp/test.eml" />
      ));
      await tick();

      expect(container.textContent).toContain("Test Subject");
      expect(container.textContent).toContain("alice@example.com");
    });

    it("calls email_parse_eml for .eml files", async () => {
      mockInvoke.mockResolvedValueOnce(mockEmlData);

      renderComponent(() => <EmailViewer path="/tmp/test.eml" />);
      await tick();

      expect(mockInvoke).toHaveBeenCalledWith("email_parse_eml", { path: "/tmp/test.eml" });
    });

    it("displays recipient information", async () => {
      mockInvoke.mockResolvedValueOnce(mockEmlData);

      const { container } = renderComponent(() => (
        <EmailViewer path="/tmp/test.eml" />
      ));
      await tick();

      expect(container.textContent).toContain("bob@example.com");
    });

    it("shows attachment count and details", async () => {
      mockInvoke.mockResolvedValueOnce(mockEmlData);

      const { container } = renderComponent(() => (
        <EmailViewer path="/tmp/test.eml" />
      ));
      await tick();

      expect(container.textContent).toContain("report.pdf");
      expect(container.textContent).toContain("Attachments (1)");
    });

    it("renders email body text", async () => {
      mockInvoke.mockResolvedValueOnce(mockEmlData);

      const { container } = renderComponent(() => (
        <EmailViewer path="/tmp/test.eml" />
      ));
      await tick();

      expect(container.textContent).toContain("Hello, this is a test email.");
    });
  });

  describe("MBOX file rendering", () => {
    it("calls email_parse_mbox for .mbox files", async () => {
      mockInvoke.mockResolvedValueOnce(mockMboxData);

      renderComponent(() => <EmailViewer path="/tmp/mailbox.mbox" />);
      await tick();

      expect(mockInvoke).toHaveBeenCalledWith("email_parse_mbox", {
        path: "/tmp/mailbox.mbox",
        maxMessages: 200,
      });
    });

    it("renders message list for mbox files", async () => {
      mockInvoke.mockResolvedValueOnce(mockMboxData);

      const { container } = renderComponent(() => (
        <EmailViewer path="/tmp/mailbox.mbox" />
      ));
      await tick();

      expect(container.textContent).toContain("First Message");
      expect(container.textContent).toContain("Second Message");
    });
  });

  describe("Loading and error states", () => {
    it("shows loading state initially", () => {
      mockInvoke.mockReturnValue(new Promise(() => {})); // Never resolves

      const { container } = renderComponent(() => (
        <EmailViewer path="/tmp/test.eml" />
      ));

      expect(container.textContent).toContain("Parsing");
    });

    it("shows error when parsing fails", async () => {
      mockInvoke.mockRejectedValueOnce(new Error("Invalid email format"));

      const { container } = renderComponent(() => (
        <EmailViewer path="/tmp/bad.eml" />
      ));
      await tick();

      expect(container.textContent).toContain("Invalid email format");
    });
  });

  describe("Edge cases", () => {
    it("handles email with no subject", async () => {
      mockInvoke.mockResolvedValueOnce({
        ...mockEmlData,
        subject: null,
      });

      const { container } = renderComponent(() => (
        <EmailViewer path="/tmp/test.eml" />
      ));
      await tick();

      expect(container.textContent).toContain("(No Subject)");
    });

    it("handles email with no attachments", async () => {
      mockInvoke.mockResolvedValueOnce({
        ...mockEmlData,
        attachments: [],
      });

      const { container } = renderComponent(() => (
        <EmailViewer path="/tmp/test.eml" />
      ));
      await tick();

      // Should not show attachments section
      expect(container.textContent).not.toContain("attachment");
    });

    it("handles email with HTML body", async () => {
      mockInvoke.mockResolvedValueOnce({
        ...mockEmlData,
        body_text: null,
        body_html: "<p>Rich text email</p>",
      });

      const { container } = renderComponent(() => (
        <EmailViewer path="/tmp/test.eml" />
      ));
      await tick();

      // Should render something indicating HTML content
      expect(container.innerHTML).toBeTruthy();
    });
  });
});
