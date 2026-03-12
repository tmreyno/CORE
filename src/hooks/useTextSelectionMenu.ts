// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useTextSelectionMenu — Provides a right-click context menu for text selected
 * inside document viewers. Offers three actions:
 *   1. Bookmark the selected text
 *   2. Create a note from the selected text
 *   3. Search for the selected text across all evidence
 *
 * Usage:
 *   const selectionMenu = useTextSelectionMenu({ onBookmark, onNote, onSearch });
 *   <div onContextMenu={selectionMenu.handleContextMenu}> ... </div>
 *   <ContextMenu items={selectionMenu.menu.items()} ... />
 */

import { createContextMenu, type ContextMenuItem } from "../components/ContextMenu";

export interface TextSelectionActions {
  /** Called with selected text + entry path/name to create a bookmark */
  onBookmarkSelection?: (text: string) => void;
  /** Called with selected text + entry path/name to create a note */
  onNoteFromSelection?: (text: string) => void;
  /** Called with selected text to trigger a search across evidence */
  onSearchSelection?: (text: string) => void;
}

/**
 * Returns a context menu manager + a `handleContextMenu` function that should
 * be set as `onContextMenu` on the viewer wrapper element. When the user
 * right-clicks with text selected, a custom menu appears. When no text is
 * selected the browser default context menu shows instead.
 */
export function useTextSelectionMenu(actions: TextSelectionActions) {
  const menu = createContextMenu();

  const handleContextMenu = (e: MouseEvent) => {
    const selection = window.getSelection();
    const selectedText = selection?.toString().trim() ?? "";

    if (!selectedText) {
      // No text selected — let browser default context menu through
      return;
    }

    // Build context menu items for the selected text
    const items: ContextMenuItem[] = [];

    if (actions.onBookmarkSelection) {
      items.push({
        id: "bookmark-selection",
        label: "Bookmark Selection",
        icon: "📑",
        onSelect: () => actions.onBookmarkSelection!(selectedText),
      });
    }

    if (actions.onNoteFromSelection) {
      items.push({
        id: "note-from-selection",
        label: "Note from Selection",
        icon: "📝",
        onSelect: () => actions.onNoteFromSelection!(selectedText),
      });
    }

    if (actions.onSearchSelection) {
      items.push({
        id: "search-selection",
        label: "Search for Selection",
        icon: "🔍",
        onSelect: () => actions.onSearchSelection!(selectedText),
      });
    }

    // Separator + copy (always available when text is selected)
    items.push(
      { id: "sep-copy", label: "", separator: true },
      {
        id: "copy-selection",
        label: "Copy",
        icon: "📋",
        shortcut: "cmd+c",
        onSelect: () => {
          navigator.clipboard.writeText(selectedText);
        },
      },
    );

    if (items.length > 0) {
      menu.open(e, items);
    }
  };

  return { menu, handleContextMenu };
}
