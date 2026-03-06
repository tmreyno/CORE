// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

export { EmailViewerComponent as EmailViewer } from "./EmailViewerComponent";
export type { EmailViewerProps, EmailInfo, EmailAddress, EmailAttachment, EmailHeader } from "./types";
export { formatEmailAddress, formatAddressList, formatEmailDate, isEml, isMbox, isMsg } from "./helpers";
export { useEmailData } from "./useEmailData";
export { EmailMessage } from "./EmailMessage";
export { MboxSidebar } from "./MboxSidebar";
