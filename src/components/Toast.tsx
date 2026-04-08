// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { ParentComponent } from "solid-js";
import { ToastProvider as SharedToastProvider, useToast } from "@core-suite/components";
import { getPreference } from "./preferences";

export type { Toast, ToastAction, ToastContextValue, ToastProviderProps, ToastType } from "@core-suite/components";
export { useToast };

export const ToastProvider: ParentComponent = (props) => (
  <SharedToastProvider notificationsEnabled={() => getPreference("enableNotifications")}>
    {props.children}
  </SharedToastProvider>
);
