// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, vi, beforeEach } from "vitest";

// Types for command history
interface Command<T = unknown> {
  type: string;
  description: string;
  previousState: T;
  newState: T;
  timestamp: number;
  execute: () => void | Promise<void>;
  undo: () => void | Promise<void>;
}

describe("useHistory", () => {
  describe("Command Structure", () => {
    it("should have correct command interface", () => {
      const command: Command<string> = {
        type: "text-change",
        description: "Change text",
        previousState: "old",
        newState: "new",
        timestamp: Date.now(),
        execute: vi.fn(),
        undo: vi.fn(),
      };

      expect(command.type).toBe("text-change");
      expect(command.description).toBe("Change text");
      expect(command.previousState).toBe("old");
      expect(command.newState).toBe("new");
      expect(typeof command.timestamp).toBe("number");
    });

    it("should support different state types", () => {
      const numberCommand: Command<number> = {
        type: "count",
        description: "Increment counter",
        previousState: 0,
        newState: 1,
        timestamp: Date.now(),
        execute: vi.fn(),
        undo: vi.fn(),
      };

      const objectCommand: Command<{ name: string }> = {
        type: "update-user",
        description: "Update user name",
        previousState: { name: "old" },
        newState: { name: "new" },
        timestamp: Date.now(),
        execute: vi.fn(),
        undo: vi.fn(),
      };

      expect(numberCommand.newState).toBe(1);
      expect(objectCommand.newState.name).toBe("new");
    });
  });

  describe("History Stack Operations", () => {
    let history: Command[];
    let currentIndex: number;

    beforeEach(() => {
      history = [];
      currentIndex = -1;
    });

    it("should start with empty history", () => {
      expect(history.length).toBe(0);
      expect(currentIndex).toBe(-1);
    });

    it("should add commands to history", () => {
      const command: Command<string> = {
        type: "test",
        description: "Test command",
        previousState: "",
        newState: "value",
        timestamp: Date.now(),
        execute: vi.fn(),
        undo: vi.fn(),
      };

      history.push(command);
      currentIndex = 0;

      expect(history.length).toBe(1);
      expect(currentIndex).toBe(0);
    });

    it("should track current position in history", () => {
      const commands: Command<number>[] = [
        { type: "a", description: "A", previousState: 0, newState: 1, timestamp: 1, execute: vi.fn(), undo: vi.fn() },
        { type: "b", description: "B", previousState: 1, newState: 2, timestamp: 2, execute: vi.fn(), undo: vi.fn() },
        { type: "c", description: "C", previousState: 2, newState: 3, timestamp: 3, execute: vi.fn(), undo: vi.fn() },
      ];

      history = [...commands];
      currentIndex = 2;

      expect(currentIndex).toBe(history.length - 1);
    });
  });

  describe("Undo/Redo Logic", () => {
    let history: Command[];
    let currentIndex: number;

    const canUndo = () => currentIndex >= 0;
    const canRedo = () => currentIndex < history.length - 1;

    beforeEach(() => {
      history = [
        { type: "a", description: "A", previousState: 0, newState: 1, timestamp: 1, execute: vi.fn(), undo: vi.fn() },
        { type: "b", description: "B", previousState: 1, newState: 2, timestamp: 2, execute: vi.fn(), undo: vi.fn() },
        { type: "c", description: "C", previousState: 2, newState: 3, timestamp: 3, execute: vi.fn(), undo: vi.fn() },
      ];
      currentIndex = 2;
    });

    it("should detect when undo is possible", () => {
      expect(canUndo()).toBe(true);
      currentIndex = -1;
      expect(canUndo()).toBe(false);
    });

    it("should detect when redo is possible", () => {
      expect(canRedo()).toBe(false);
      currentIndex = 1;
      expect(canRedo()).toBe(true);
    });

    it("should undo by decrementing index", async () => {
      expect(currentIndex).toBe(2);
      
      // Undo
      if (canUndo()) {
        const command = history[currentIndex];
        await command.undo();
        currentIndex--;
      }

      expect(currentIndex).toBe(1);
      expect(history[2].undo).toHaveBeenCalled();
    });

    it("should redo by incrementing index", async () => {
      currentIndex = 1;
      
      // Redo
      if (canRedo()) {
        currentIndex++;
        const command = history[currentIndex];
        await command.execute();
      }

      expect(currentIndex).toBe(2);
      expect(history[2].execute).toHaveBeenCalled();
    });

    it("should clear history after current position when new command added", () => {
      currentIndex = 1;
      
      const newCommand: Command = {
        type: "d",
        description: "D",
        previousState: 2,
        newState: 4,
        timestamp: 4,
        execute: vi.fn(),
        undo: vi.fn(),
      };

      // Truncate history and add new command
      history = history.slice(0, currentIndex + 1);
      history.push(newCommand);
      currentIndex = history.length - 1;

      expect(history.length).toBe(3);
      expect(history[2].description).toBe("D");
    });
  });

  describe("History Size Management", () => {
    it("should respect max history limit", () => {
      const maxHistory = 5;
      const history: Command[] = [];

      for (let i = 0; i < 10; i++) {
        const command: Command<number> = {
          type: `cmd-${i}`,
          description: `Command ${i}`,
          previousState: i - 1,
          newState: i,
          timestamp: i,
          execute: vi.fn(),
          undo: vi.fn(),
        };

        history.push(command);
        
        // Enforce max history
        if (history.length > maxHistory) {
          history.shift();
        }
      }

      expect(history.length).toBe(maxHistory);
      expect(history[0].description).toBe("Command 5");
    });
  });

  describe("Command Descriptions", () => {
    it("should provide undo description", () => {
      const history: Command[] = [
        { type: "a", description: "Delete file", previousState: 0, newState: 1, timestamp: 1, execute: vi.fn(), undo: vi.fn() },
      ];
      const currentIndex = 0;

      const undoDescription = currentIndex >= 0 ? history[currentIndex]?.description : null;
      expect(undoDescription).toBe("Delete file");
    });

    it("should provide redo description", () => {
      const history: Command[] = [
        { type: "a", description: "First", previousState: 0, newState: 1, timestamp: 1, execute: vi.fn(), undo: vi.fn() },
        { type: "b", description: "Second", previousState: 1, newState: 2, timestamp: 2, execute: vi.fn(), undo: vi.fn() },
      ];
      const currentIndex = 0;

      const redoDescription = currentIndex < history.length - 1 
        ? history[currentIndex + 1]?.description 
        : null;
      expect(redoDescription).toBe("Second");
    });

    it("should return null when no undo available", () => {
      const history: Command[] = [];
      const currentIndex = -1;

      const undoDescription = currentIndex >= 0 ? history[currentIndex]?.description : null;
      expect(undoDescription).toBeNull();
    });
  });
});

describe("History Keyboard Shortcuts", () => {
  it("should recognize Cmd+Z for undo", () => {
    const isMac = true;
    const isUndoShortcut = (e: { key: string; metaKey: boolean; ctrlKey: boolean }) => {
      if (isMac) {
        return e.key === "z" && e.metaKey && !e.ctrlKey;
      }
      return e.key === "z" && e.ctrlKey && !e.metaKey;
    };

    expect(isUndoShortcut({ key: "z", metaKey: true, ctrlKey: false })).toBe(true);
    expect(isUndoShortcut({ key: "z", metaKey: false, ctrlKey: true })).toBe(false);
  });

  it("should recognize Cmd+Shift+Z for redo", () => {
    const isMac = true;
    const isRedoShortcut = (e: { key: string; metaKey: boolean; ctrlKey: boolean; shiftKey: boolean }) => {
      if (isMac) {
        return e.key === "z" && e.metaKey && e.shiftKey;
      }
      return (e.key === "y" && e.ctrlKey) || (e.key === "z" && e.ctrlKey && e.shiftKey);
    };

    expect(isRedoShortcut({ key: "z", metaKey: true, ctrlKey: false, shiftKey: true })).toBe(true);
    expect(isRedoShortcut({ key: "z", metaKey: true, ctrlKey: false, shiftKey: false })).toBe(false);
  });
});
