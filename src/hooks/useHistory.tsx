// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, onMount, onCleanup, Accessor } from "solid-js";

// ============================================================================
// Types
// ============================================================================

export interface Command<T = unknown> {
  /** Unique identifier for this command type */
  type: string;
  /** Human-readable description */
  description: string;
  /** The data/state before the command was executed */
  previousState: T;
  /** The data/state after the command was executed */
  newState: T;
  /** Timestamp when the command was executed */
  timestamp: number;
  /** Execute the command (apply newState) */
  execute: () => void | Promise<void>;
  /** Undo the command (revert to previousState) */
  undo: () => void | Promise<void>;
}

export interface HistoryState {
  /** Can we undo? */
  canUndo: Accessor<boolean>;
  /** Can we redo? */
  canRedo: Accessor<boolean>;
  /** Current position in history */
  currentIndex: Accessor<number>;
  /** Total number of commands in history */
  historyLength: Accessor<number>;
  /** Description of the command that would be undone */
  undoDescription: Accessor<string | null>;
  /** Description of the command that would be redone */
  redoDescription: Accessor<string | null>;
  /** The full history stack */
  history: Accessor<Command[]>;
}

export interface HistoryActions {
  /** Execute a new command and add it to history */
  execute: <T>(command: Command<T>) => Promise<void>;
  /** Undo the last command */
  undo: () => Promise<boolean>;
  /** Redo the next command */
  redo: () => Promise<boolean>;
  /** Clear all history */
  clear: () => void;
  /** Go to a specific point in history */
  goTo: (index: number) => Promise<void>;
}

export interface UseHistoryOptions {
  /** Maximum number of commands to keep in history */
  maxHistory?: number;
  /** Called when undo is performed */
  onUndo?: (command: Command) => void;
  /** Called when redo is performed */
  onRedo?: (command: Command) => void;
  /** Called when a new command is executed */
  onExecute?: (command: Command) => void;
}

// ============================================================================
// Hook
// ============================================================================

/**
 * Hook for managing undo/redo command history
 */
export function useHistory(options: UseHistoryOptions = {}): [HistoryState, HistoryActions] {
  const maxHistory = options.maxHistory ?? 100;
  
  // History stack and current position
  const [history, setHistory] = createSignal<Command[]>([]);
  const [currentIndex, setCurrentIndex] = createSignal(-1);
  const [isExecuting, setIsExecuting] = createSignal(false);

  // Computed values
  const canUndo = () => currentIndex() >= 0 && !isExecuting();
  const canRedo = () => currentIndex() < history().length - 1 && !isExecuting();
  const historyLength = () => history().length;
  
  const undoDescription = () => {
    if (!canUndo()) return null;
    return history()[currentIndex()]?.description ?? null;
  };
  
  const redoDescription = () => {
    if (!canRedo()) return null;
    return history()[currentIndex() + 1]?.description ?? null;
  };

  // Actions
  const execute = async <T,>(command: Command<T>): Promise<void> => {
    if (isExecuting()) return;
    
    setIsExecuting(true);
    try {
      // Execute the command
      await command.execute();
      
      // If we're not at the end of history, truncate forward history
      const idx = currentIndex();
      const currentHistory = history();
      let newHistory = idx < currentHistory.length - 1 
        ? currentHistory.slice(0, idx + 1) 
        : [...currentHistory];
      
      // Add new command
      newHistory.push(command);
      
      // Trim if exceeding max history
      if (newHistory.length > maxHistory) {
        newHistory = newHistory.slice(newHistory.length - maxHistory);
      }
      
      setHistory(newHistory);
      setCurrentIndex(newHistory.length - 1);
      
      options.onExecute?.(command);
    } finally {
      setIsExecuting(false);
    }
  };

  const undo = async (): Promise<boolean> => {
    if (!canUndo() || isExecuting()) return false;
    
    setIsExecuting(true);
    try {
      const command = history()[currentIndex()];
      await command.undo();
      setCurrentIndex((i) => i - 1);
      options.onUndo?.(command);
      return true;
    } finally {
      setIsExecuting(false);
    }
  };

  const redo = async (): Promise<boolean> => {
    if (!canRedo() || isExecuting()) return false;
    
    setIsExecuting(true);
    try {
      const command = history()[currentIndex() + 1];
      await command.execute();
      setCurrentIndex((i) => i + 1);
      options.onRedo?.(command);
      return true;
    } finally {
      setIsExecuting(false);
    }
  };

  const clear = () => {
    setHistory([]);
    setCurrentIndex(-1);
  };

  const goTo = async (targetIndex: number): Promise<void> => {
    if (isExecuting()) return;
    if (targetIndex < -1 || targetIndex >= history().length) return;
    
    const current = currentIndex();
    if (targetIndex === current) return;
    
    setIsExecuting(true);
    try {
      if (targetIndex < current) {
        // Undo commands from current to target+1
        for (let i = current; i > targetIndex; i--) {
          await history()[i].undo();
        }
      } else {
        // Redo commands from current+1 to target
        for (let i = current + 1; i <= targetIndex; i++) {
          await history()[i].execute();
        }
      }
      setCurrentIndex(targetIndex);
    } finally {
      setIsExecuting(false);
    }
  };

  // Set up keyboard shortcuts
  onMount(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Cmd+Z / Ctrl+Z = Undo
      if ((e.metaKey || e.ctrlKey) && e.key === "z" && !e.shiftKey) {
        e.preventDefault();
        undo();
      }
      // Cmd+Shift+Z / Ctrl+Shift+Z or Cmd+Y = Redo
      if ((e.metaKey || e.ctrlKey) && ((e.key === "z" && e.shiftKey) || e.key === "y")) {
        e.preventDefault();
        redo();
      }
    };
    
    document.addEventListener("keydown", handleKeyDown);
    onCleanup(() => document.removeEventListener("keydown", handleKeyDown));
  });

  const state: HistoryState = {
    canUndo,
    canRedo,
    currentIndex,
    historyLength,
    undoDescription,
    redoDescription,
    history,
  };

  const actions: HistoryActions = {
    execute,
    undo,
    redo,
    clear,
    goTo,
  };

  return [state, actions];
}

// ============================================================================
// Helper: Create a simple state command
// ============================================================================

/**
 * Create a simple command that updates a signal's value
 */
export function createStateCommand<T>(
  description: string,
  setter: (value: T) => void,
  previousState: T,
  newState: T
): Command<T> {
  return {
    type: "state-change",
    description,
    previousState,
    newState,
    timestamp: Date.now(),
    execute: () => setter(newState),
    undo: () => setter(previousState),
  };
}

/**
 * Create a batch command that executes multiple commands as one
 */
export function createBatchCommand(
  description: string,
  commands: Command[]
): Command<Command[]> {
  return {
    type: "batch",
    description,
    previousState: commands,
    newState: commands,
    timestamp: Date.now(),
    execute: async () => {
      for (const cmd of commands) {
        await cmd.execute();
      }
    },
    undo: async () => {
      // Undo in reverse order
      for (let i = commands.length - 1; i >= 0; i--) {
        await commands[i].undo();
      }
    },
  };
}

// ============================================================================
// Context Provider (optional, for global undo/redo)
// ============================================================================

import { createContext, useContext, JSX } from "solid-js";

interface HistoryContextValue {
  state: HistoryState;
  actions: HistoryActions;
}

const HistoryContext = createContext<HistoryContextValue>();

export function HistoryProvider(props: { 
  children: JSX.Element;
  options?: UseHistoryOptions;
}) {
  const [state, actions] = useHistory(props.options);
  
  return (
    <HistoryContext.Provider value={{ state, actions }}>
      {props.children}
    </HistoryContext.Provider>
  );
}

export function useHistoryContext(): HistoryContextValue {
  const context = useContext(HistoryContext);
  if (!context) {
    throw new Error("useHistoryContext must be used within a HistoryProvider");
  }
  return context;
}

export default useHistory;
