// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createContext, useContext, createSignal, For, Show, onCleanup, type ParentComponent } from "solid-js";
import { Portal } from "solid-js/web";
import { 
  HiOutlineCheckCircle,
  HiOutlineXCircle,
  HiOutlineExclamationTriangle,
  HiOutlineInformationCircle,
  HiOutlineArrowPath,
  HiOutlineXMark
} from "./icons";
import { getPreference } from "./preferences";

// Toast types
export type ToastType = "success" | "error" | "warning" | "info" | "loading";

export interface Toast {
  id: string;
  type: ToastType;
  title: string;
  message?: string;
  duration?: number;
  dismissible?: boolean;
  progress?: number; // 0-100 for loading toasts
  action?: {
    label: string;
    onClick: () => void;
  };
}

interface ToastContextValue {
  toasts: () => Toast[];
  addToast: (toast: Omit<Toast, "id">) => string;
  removeToast: (id: string) => void;
  updateToast: (id: string, updates: Partial<Toast>) => void;
  clearAll: () => void;
  success: (title: string, message?: string) => string;
  error: (title: string, message?: string) => string;
  warning: (title: string, message?: string) => string;
  info: (title: string, message?: string) => string;
  loading: (title: string, message?: string) => string;
  promise: <T>(
    promise: Promise<T>,
    messages: { loading: string; success: string; error: string }
  ) => Promise<T>;
}

const ToastContext = createContext<ToastContextValue>();

let toastId = 0;
const timers = new Map<string, number>();

/**
 * Toast Provider - wraps app to provide toast functionality
 */
export const ToastProvider: ParentComponent = (props) => {
  const [toasts, setToasts] = createSignal<Toast[]>([]);

  const addToast = (toast: Omit<Toast, "id">): string => {
    // Check if notifications are enabled (loading toasts always show - they indicate critical operations)
    if (toast.type !== "loading" && !getPreference("enableNotifications")) {
      return ""; // Return empty string - no toast added
    }
    
    const id = `toast-${++toastId}`;
    const newToast: Toast = { 
      ...toast, 
      id,
      dismissible: toast.dismissible ?? (toast.type !== "loading"),
    };
    
    setToasts((prev) => [...prev, newToast]);

    // Auto-remove after duration (default 5s, 0 for loading)
    const duration = toast.duration ?? (toast.type === "loading" ? 0 : 5000);
    if (duration > 0) {
      const timer = window.setTimeout(() => removeToast(id), duration);
      timers.set(id, timer);
    }

    return id;
  };

  const removeToast = (id: string) => {
    const timer = timers.get(id);
    if (timer) {
      clearTimeout(timer);
      timers.delete(id);
    }
    setToasts((prev) => prev.filter((t) => t.id !== id));
  };

  const updateToast = (id: string, updates: Partial<Toast>) => {
    setToasts((prev) =>
      prev.map((t) => (t.id === id ? { ...t, ...updates } : t))
    );
  };

  const clearAll = () => {
    timers.forEach((timer) => clearTimeout(timer));
    timers.clear();
    setToasts([]);
  };

  const success = (title: string, message?: string) => 
    addToast({ type: "success", title, message });

  const error = (title: string, message?: string) => 
    addToast({ type: "error", title, message, duration: 8000 });

  const warning = (title: string, message?: string) => 
    addToast({ type: "warning", title, message });

  const info = (title: string, message?: string) => 
    addToast({ type: "info", title, message });

  const loading = (title: string, message?: string) =>
    addToast({ type: "loading", title, message, duration: 0, dismissible: false });

  const promise = async <T,>(
    promiseToTrack: Promise<T>,
    messages: { loading: string; success: string; error: string }
  ): Promise<T> => {
    const id = loading(messages.loading);
    try {
      const result = await promiseToTrack;
      updateToast(id, { 
        type: "success", 
        title: messages.success, 
        dismissible: true, 
        duration: 5000 
      });
      setTimeout(() => removeToast(id), 5000);
      return result;
    } catch (err) {
      updateToast(id, {
        type: "error",
        title: messages.error,
        message: err instanceof Error ? err.message : "Unknown error",
        dismissible: true,
        duration: 8000,
      });
      setTimeout(() => removeToast(id), 8000);
      throw err;
    }
  };

  // Cleanup timers on unmount
  onCleanup(() => {
    timers.forEach((timer) => clearTimeout(timer));
  });

  const value: ToastContextValue = {
    toasts,
    addToast,
    removeToast,
    updateToast,
    clearAll,
    success,
    error,
    warning,
    info,
    loading,
    promise,
  };

  return (
    <ToastContext.Provider value={value}>
      {props.children}
      <Portal>
        <ToastContainer toasts={toasts()} onDismiss={removeToast} />
      </Portal>
    </ToastContext.Provider>
  );
};

/**
 * Hook to access toast functionality
 */
export function useToast(): ToastContextValue {
  const context = useContext(ToastContext);
  if (!context) {
    throw new Error("useToast must be used within a ToastProvider");
  }
  return context;
}

// Toast icon by type - using Heroicons outline
const ToastIcon = (props: { type: ToastType; class?: string }) => {
  const iconClass = () => `w-5 h-5 shrink-0 ${props.class || ""}`;
  
  switch (props.type) {
    case "success":
      return <HiOutlineCheckCircle class={iconClass()} />;
    case "error":
      return <HiOutlineXCircle class={iconClass()} />;
    case "warning":
      return <HiOutlineExclamationTriangle class={iconClass()} />;
    case "info":
      return <HiOutlineInformationCircle class={iconClass()} />;
    case "loading":
      return <HiOutlineArrowPath class={`${iconClass()} animate-spin`} />;
    default:
      return <HiOutlineInformationCircle class={iconClass()} />;
  }
};

// Toast colors by type (using Tailwind classes)
const toastStyles: Record<ToastType, { bg: string; border: string; icon: string }> = {
  success: { bg: "bg-success-soft", border: "border-success", icon: "text-success" },
  error: { bg: "bg-error-soft", border: "border-error", icon: "text-error" },
  warning: { bg: "bg-warning-soft", border: "border-warning", icon: "text-warning" },
  info: { bg: "bg-accent-soft", border: "border-accent", icon: "text-accent" },
  loading: { bg: "bg-bg-panel", border: "border-border", icon: "text-accent" },
};

/**
 * Single Toast component
 */
function ToastItem(props: { toast: Toast; onDismiss: () => void }) {
  const style = () => toastStyles[props.toast.type];
  const isLoading = () => props.toast.type === "loading";

  return (
    <div
      class={`flex items-start gap-3 px-4 py-3 rounded-lg border shadow-lg bg-bg-card animate-[slideInRight_0.3s_ease-out] ${style().bg} ${style().border}`}
      role="alert"
      aria-live="polite"
    >
      <ToastIcon type={props.toast.type} class={style().icon} />
      <div class="flex-1 min-w-0">
        <div class="text-sm font-semibold text-txt">{props.toast.title}</div>
        <Show when={props.toast.message}>
          <div class="text-xs text-txt-muted mt-0.5">{props.toast.message}</div>
        </Show>
        
        {/* Progress bar for loading toasts */}
        <Show when={isLoading() && props.toast.progress !== undefined}>
          <div class="mt-2 h-1.5 bg-bg-hover rounded-full overflow-hidden">
            <div
              class="h-full bg-accent transition-all duration-300 ease-out"
              style={{ width: `${props.toast.progress}%` }}
            />
          </div>
        </Show>

        {/* Action button */}
        <Show when={props.toast.action}>
          <button
            class="mt-2 text-xs font-medium text-accent hover:text-accent-hover underline hover:no-underline cursor-pointer bg-transparent border-none"
            onClick={props.toast.action!.onClick}
          >
            {props.toast.action!.label}
          </button>
        </Show>
      </div>
      <Show when={props.toast.dismissible !== false}>
        <button 
          class="p-1 text-txt-muted hover:text-txt transition-colors cursor-pointer bg-transparent border-none"
          onClick={props.onDismiss}
          aria-label="Dismiss notification"
        >
          <HiOutlineXMark class="w-4 h-4" />
        </button>
      </Show>
    </div>
  );
}

/**
 * Toast Container - renders all active toasts
 */
function ToastContainer(props: { toasts: Toast[]; onDismiss: (id: string) => void }) {
  return (
    <div class="fixed bottom-4 right-4 z-50 flex flex-col gap-2 max-w-[400px]" aria-live="polite" aria-label="Notifications">
      <For each={props.toasts}>
        {(toast) => (
          <ToastItem toast={toast} onDismiss={() => props.onDismiss(toast.id)} />

        )}
      </For>
    </div>
  );
}
