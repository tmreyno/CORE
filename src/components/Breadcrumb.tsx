// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { For, Show, JSX } from "solid-js";
import { HiOutlineFolder, HiOutlineDocument } from "./icons";

export interface BreadcrumbItem {
  label: string;
  path: string;
  icon?: JSX.Element | string;
}

interface BreadcrumbProps {
  items: BreadcrumbItem[];
  onNavigate: (path: string) => void;
  maxItems?: number;
  separator?: string;
}

export function Breadcrumb(props: BreadcrumbProps) {
  const maxItems = () => props.maxItems ?? 4;
  const separator = () => props.separator ?? "/";

  // Truncate middle items if too many
  const displayItems = () => {
    const items = props.items;
    const max = maxItems();
    
    if (items.length <= max) return items;
    
    // Show first, ellipsis, and last (max-1) items
    const firstItem = items[0];
    const lastItems = items.slice(-(max - 1));
    
    return [
      firstItem,
      { label: "...", path: "", icon: undefined } as BreadcrumbItem,
      ...lastItems,
    ];
  };

  return (
    <nav 
      class="flex items-center gap-0.5 text-[10px] text-zinc-400 min-w-0 overflow-hidden"
      aria-label="Breadcrumb"
    >
      <ol class="flex items-center gap-0.5 min-w-0">
        <For each={displayItems()}>
          {(item, index) => (
            <>
              <li class="flex items-center min-w-0">
                <Show when={item.label === "..."}>
                  <span class="px-0.5 text-zinc-500">...</span>
                </Show>
                <Show when={item.label !== "..."}>
                  <button
                    class={`flex items-center gap-0.5 px-1 py-0.5 rounded transition-colors min-w-0 ${
                      index() === displayItems().length - 1
                        ? "text-zinc-200 font-medium cursor-default"
                        : "hover:bg-zinc-800 hover:text-zinc-200"
                    }`}
                    onClick={() => {
                      if (index() < displayItems().length - 1) {
                        props.onNavigate(item.path);
                      }
                    }}
                    disabled={index() === displayItems().length - 1}
                    title={item.path}
                  >
                    <Show when={item.icon}>
                      <span class="shrink-0 w-2.5 h-2.5 flex items-center justify-center [&>svg]:w-2.5 [&>svg]:h-2.5">{item.icon}</span>
                    </Show>
                    <span class="truncate max-w-[120px]">{item.label}</span>
                  </button>
                </Show>
              </li>
              
              {/* Separator */}
              <Show when={index() < displayItems().length - 1}>
                <li class="text-zinc-600 shrink-0" aria-hidden="true">
                  {separator()}
                </li>
              </Show>
            </>
          )}
        </For>
      </ol>
    </nav>
  );
}

// Helper to create breadcrumb items from a file path
export function pathToBreadcrumbs(fullPath: string, rootLabel = "Root"): BreadcrumbItem[] {
  if (!fullPath) return [];
  
  const parts = fullPath.split("/").filter(Boolean);
  const items: BreadcrumbItem[] = [];
  
  // Add root
  items.push({
    label: rootLabel,
    path: "/",
    icon: <HiOutlineFolder class="w-4 h-4 text-yellow-500" />,
  });
  
  // Add each path segment
  let currentPath = "";
  for (let i = 0; i < parts.length; i++) {
    currentPath += "/" + parts[i];
    const isLast = i === parts.length - 1;
    
    items.push({
      label: parts[i],
      path: currentPath,
      icon: isLast ? <HiOutlineDocument class="w-4 h-4" /> : <HiOutlineFolder class="w-4 h-4 text-yellow-500" />,
    });
  }
  
  return items;
}
