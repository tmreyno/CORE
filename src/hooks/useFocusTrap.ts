// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, onMount, onCleanup, Accessor } from "solid-js";

/**
 * Creates a focus trap that keeps focus within a container element.
 * Useful for modals, dialogs, and other overlay components.
 */
export function useFocusTrap(
  containerRef: Accessor<HTMLElement | undefined>,
  isActive: Accessor<boolean>
) {
  const [previouslyFocused, setPreviouslyFocused] = createSignal<HTMLElement | null>(null);

  // Get all focusable elements within the container
  const getFocusableElements = (): HTMLElement[] => {
    const container = containerRef();
    if (!container) return [];
    
    const focusableSelectors = [
      'button:not([disabled])',
      'input:not([disabled])',
      'select:not([disabled])',
      'textarea:not([disabled])',
      'a[href]',
      '[tabindex]:not([tabindex="-1"])',
    ].join(', ');
    
    return Array.from(container.querySelectorAll<HTMLElement>(focusableSelectors));
  };

  // Handle Tab key to trap focus
  const handleKeyDown = (e: KeyboardEvent) => {
    if (!isActive() || e.key !== 'Tab') return;
    
    const focusable = getFocusableElements();
    if (focusable.length === 0) return;
    
    const firstElement = focusable[0];
    const lastElement = focusable[focusable.length - 1];
    
    if (e.shiftKey) {
      // Shift+Tab: if on first element, go to last
      if (document.activeElement === firstElement) {
        e.preventDefault();
        lastElement.focus();
      }
    } else {
      // Tab: if on last element, go to first
      if (document.activeElement === lastElement) {
        e.preventDefault();
        firstElement.focus();
      }
    }
  };

  // Activate trap
  const activate = () => {
    // Store currently focused element
    setPreviouslyFocused(document.activeElement as HTMLElement);
    
    // Focus first focusable element
    const focusable = getFocusableElements();
    if (focusable.length > 0) {
      focusable[0].focus();
    }
    
    // Add keydown listener
    document.addEventListener('keydown', handleKeyDown);
  };

  // Deactivate trap
  const deactivate = () => {
    document.removeEventListener('keydown', handleKeyDown);
    
    // Restore focus to previously focused element
    const prev = previouslyFocused();
    if (prev && typeof prev.focus === 'function') {
      prev.focus();
    }
  };

  // Watch for active state changes
  onMount(() => {
    if (isActive()) {
      activate();
    }
  });

  // Cleanup
  onCleanup(() => {
    deactivate();
  });

  // Return control functions
  return {
    activate,
    deactivate,
    getFocusableElements,
  };
}

/**
 * Simplified focus trap that auto-activates based on a signal.
 * Returns a ref setter and automatically manages focus.
 */
export function createFocusTrap(isActive: Accessor<boolean>) {
  const [containerRef, setContainerRef] = createSignal<HTMLElement | undefined>();
  
  let previouslyFocused: HTMLElement | null = null;
  
  const getFocusableElements = (): HTMLElement[] => {
    const container = containerRef();
    if (!container) return [];
    
    const focusableSelectors = [
      'button:not([disabled])',
      'input:not([disabled])',
      'select:not([disabled])',
      'textarea:not([disabled])',
      'a[href]',
      '[tabindex]:not([tabindex="-1"])',
    ].join(', ');
    
    return Array.from(container.querySelectorAll<HTMLElement>(focusableSelectors));
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (!isActive() || e.key !== 'Tab') return;
    
    const focusable = getFocusableElements();
    if (focusable.length === 0) return;
    
    const firstElement = focusable[0];
    const lastElement = focusable[focusable.length - 1];
    
    if (e.shiftKey) {
      if (document.activeElement === firstElement) {
        e.preventDefault();
        lastElement.focus();
      }
    } else {
      if (document.activeElement === lastElement) {
        e.preventDefault();
        firstElement.focus();
      }
    }
  };

  // Effect to watch active state
  onMount(() => {
    const checkActive = () => {
      if (isActive()) {
        previouslyFocused = document.activeElement as HTMLElement;
        document.addEventListener('keydown', handleKeyDown);
        
        // Focus first element after a tick
        requestAnimationFrame(() => {
          const focusable = getFocusableElements();
          if (focusable.length > 0) {
            focusable[0].focus();
          }
        });
      } else {
        document.removeEventListener('keydown', handleKeyDown);
        if (previouslyFocused) {
          previouslyFocused.focus();
          previouslyFocused = null;
        }
      }
    };
    
    // Initial check
    checkActive();
  });

  onCleanup(() => {
    document.removeEventListener('keydown', handleKeyDown);
  });

  return {
    ref: setContainerRef,
    containerRef,
  };
}
