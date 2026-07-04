import { isTauri } from "../../utils/platform";
import type { ExportToast } from "./types";

export const DESKTOP_EXPORT_ENGINE_MESSAGE =
  "Export, imaging, memory capture, and triage engines are available in the desktop app. Browser preview can configure fields but cannot run native acquisition tools.";

export function canUseDesktopExportEngine(toast: ExportToast): boolean {
  if (isTauri) return true;
  toast.error("Desktop Runtime Required", DESKTOP_EXPORT_ENGINE_MESSAGE);
  return false;
}
