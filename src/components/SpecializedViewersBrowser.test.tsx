import { describe, expect, it, vi, beforeEach } from "vitest";
import { render } from "solid-js/web";
import { mockInvoke } from "../__tests__/setup";

vi.mock("../utils/platform", () => ({
  isTauri: false,
}));

vi.mock("pdfjs-dist", () => ({
  GlobalWorkerOptions: { workerSrc: "" },
}));

const mockLoadPdfDocument = vi.fn();

vi.mock("./pdf/pdfHelpers", () => ({
  loadPdfDocument: (...args: any[]) => mockLoadPdfDocument(...args),
  renderPdfPage: vi.fn(),
  generateThumbnailsBatch: vi.fn(),
}));

vi.mock("@solid-primitives/resize-observer", () => ({
  createResizeObserver: vi.fn(),
}));

vi.mock("@solid-primitives/scheduled", () => ({
  debounce: (fn: () => void) => fn,
}));

vi.mock("@solid-primitives/event-listener", () => ({
  makeEventListener: vi.fn(),
}));

import { BinaryViewer } from "./BinaryViewer";
import { DatabaseViewer } from "./DatabaseViewer";
import { EmailViewer } from "./EmailViewer";
import { PdfViewer } from "./PdfViewer";
import { PlistViewer } from "./PlistViewer";
import { PstViewer } from "./PstViewer";
import { RegistryViewer } from "./RegistryViewer";
import { SpreadsheetViewer } from "./SpreadsheetViewer";

const tick = () => new Promise((resolve) => setTimeout(resolve, 0));

function renderComponent(component: () => any) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const dispose = render(component, container);
  return { container, dispose };
}

describe("specialized evidence viewers browser runtime guards", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
    mockInvoke.mockReset();
    mockLoadPdfDocument.mockReset();
  });

  it.each([
    {
      name: "database",
      component: () => <DatabaseViewer path="/evidence/data.sqlite" />,
      message: "Database evidence viewing is available in the desktop app.",
    },
    {
      name: "spreadsheet",
      component: () => <SpreadsheetViewer path="/evidence/data.xlsx" />,
      message: "Spreadsheet evidence viewing is available in the desktop app.",
    },
    {
      name: "email",
      component: () => <EmailViewer path="/evidence/message.eml" />,
      message: "Email evidence viewing is available in the desktop app.",
    },
    {
      name: "binary",
      component: () => <BinaryViewer path="/evidence/program.exe" />,
      message: "Binary evidence analysis is available in the desktop app.",
    },
    {
      name: "registry",
      component: () => <RegistryViewer path="/evidence/NTUSER.DAT" />,
      message: "Registry evidence viewing is available in the desktop app.",
    },
    {
      name: "plist",
      component: () => <PlistViewer path="/evidence/Info.plist" />,
      message: "Plist evidence viewing is available in the desktop app.",
    },
    {
      name: "pst",
      component: () => <PstViewer path="/evidence/mail.pst" />,
      message: "PST evidence viewing is available in the desktop app.",
    },
  ])("$name viewer does not call native commands", async ({ component, message }) => {
    const { container, dispose } = renderComponent(component);

    await tick();

    expect(mockInvoke).not.toHaveBeenCalled();
    expect(container.textContent).toContain(message);
    dispose();
  });

  it("pdf viewer does not load native PDF bytes in browser preview", async () => {
    const { container, dispose } = renderComponent(() => (
      <PdfViewer path="/evidence/report.pdf" />
    ));

    await tick();

    expect(mockLoadPdfDocument).not.toHaveBeenCalled();
    expect(container.textContent).toContain(
      "PDF evidence viewing is available in the desktop app.",
    );
    dispose();
  });
});
