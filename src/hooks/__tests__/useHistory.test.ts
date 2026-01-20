// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, vi } from "vitest";
import { createRoot } from "solid-js";
import { useHistory, createStateCommand, createBatchCommand, type Command } from "../useHistory";

describe("useHistory", () => {
  // Helper to run tests in a reactive context
  function testWithRoot<T>(fn: () => T): T {
    let result!: T;
    createRoot((dispose) => {
      result = fn();
      dispose();
    });
    return result;
  }

  describe("initial state", () => {
    it("should start with empty history", () => {
      testWithRoot(() => {
        const [state] = useHistory();
        expect(state.historyLength()).toBe(0);
        expect(state.currentIndex()).toBe(-1);
        expect(state.canUndo()).toBe(false);
        expect(state.canRedo()).toBe(false);
      });
    });

    it("should have null undo/redo descriptions initially", () => {
      testWithRoot(() => {
        const [state] = useHistory();
        expect(state.undoDescription()).toBeNull();
        expect(state.redoDescription()).toBeNull();
      });
    });
  });

  describe("execute", () => {
    it("should add command to history when executed", async () => {
      await testWithRoot(async () => {
        const [state, actions] = useHistory();
        
        const command: Command<number> = {
          type: "test",
          description: "Test command",
          previousState: 0,
          newState: 1,
          timestamp: Date.now(),
          execute: vi.fn(),
          undo: vi.fn(),
        };

        await actions.execute(command);

        expect(state.historyLength()).toBe(1);
        expect(state.currentIndex()).toBe(0);
        expect(state.canUndo()).toBe(true);
        expect(state.canRedo()).toBe(false);
        expect(command.execute).toHaveBeenCalledOnce();
      });
    });

    it("should call onExecute callback", async () => {
      await testWithRoot(async () => {
        const onExecute = vi.fn();
        const [, actions] = useHistory({ onExecute });
        
        const command: Command<number> = {
          type: "test",
          description: "Test",
          previousState: 0,
          newState: 1,
          timestamp: Date.now(),
          execute: vi.fn(),
          undo: vi.fn(),
        };

        await actions.execute(command);

        expect(onExecute).toHaveBeenCalledWith(command);
      });
    });

    it("should truncate forward history when executing new command", async () => {
      await testWithRoot(async () => {
        const [state, actions] = useHistory();
        
        const createCommand = (n: number): Command<number> => ({
          type: "test",
          description: `Command ${n}`,
          previousState: n - 1,
          newState: n,
          timestamp: Date.now(),
          execute: vi.fn(),
          undo: vi.fn(),
        });

        // Execute 3 commands
        await actions.execute(createCommand(1));
        await actions.execute(createCommand(2));
        await actions.execute(createCommand(3));

        // Undo twice (now at command 1)
        await actions.undo();
        await actions.undo();

        expect(state.currentIndex()).toBe(0);
        expect(state.historyLength()).toBe(3);

        // Execute new command - should truncate commands 2 and 3
        await actions.execute(createCommand(4));

        expect(state.historyLength()).toBe(2); // Only command 1 and 4
        expect(state.currentIndex()).toBe(1);
        expect(state.canRedo()).toBe(false);
      });
    });

    it("should respect maxHistory option", async () => {
      await testWithRoot(async () => {
        const [state, actions] = useHistory({ maxHistory: 3 });
        
        const createCommand = (n: number): Command<number> => ({
          type: "test",
          description: `Command ${n}`,
          previousState: n - 1,
          newState: n,
          timestamp: Date.now(),
          execute: vi.fn(),
          undo: vi.fn(),
        });

        // Execute 5 commands with max of 3
        for (let i = 1; i <= 5; i++) {
          await actions.execute(createCommand(i));
        }

        expect(state.historyLength()).toBe(3);
        // Should keep the last 3 commands (3, 4, 5)
        expect(state.history()[0].description).toBe("Command 3");
      });
    });
  });

  describe("undo", () => {
    it("should undo the last command", async () => {
      await testWithRoot(async () => {
        const [state, actions] = useHistory();
        
        const undoFn = vi.fn();
        const command: Command<number> = {
          type: "test",
          description: "Test",
          previousState: 0,
          newState: 1,
          timestamp: Date.now(),
          execute: vi.fn(),
          undo: undoFn,
        };

        await actions.execute(command);
        const result = await actions.undo();

        expect(result).toBe(true);
        expect(undoFn).toHaveBeenCalledOnce();
        expect(state.currentIndex()).toBe(-1);
        expect(state.canUndo()).toBe(false);
        expect(state.canRedo()).toBe(true);
      });
    });

    it("should return false when nothing to undo", async () => {
      await testWithRoot(async () => {
        const [, actions] = useHistory();
        
        const result = await actions.undo();

        expect(result).toBe(false);
      });
    });

    it("should call onUndo callback", async () => {
      await testWithRoot(async () => {
        const onUndo = vi.fn();
        const [, actions] = useHistory({ onUndo });
        
        const command: Command<number> = {
          type: "test",
          description: "Test",
          previousState: 0,
          newState: 1,
          timestamp: Date.now(),
          execute: vi.fn(),
          undo: vi.fn(),
        };

        await actions.execute(command);
        await actions.undo();

        expect(onUndo).toHaveBeenCalledWith(command);
      });
    });
  });

  describe("redo", () => {
    it("should redo the last undone command", async () => {
      await testWithRoot(async () => {
        const [state, actions] = useHistory();
        
        const executeFn = vi.fn();
        const command: Command<number> = {
          type: "test",
          description: "Test",
          previousState: 0,
          newState: 1,
          timestamp: Date.now(),
          execute: executeFn,
          undo: vi.fn(),
        };

        await actions.execute(command);
        await actions.undo();
        const result = await actions.redo();

        expect(result).toBe(true);
        expect(executeFn).toHaveBeenCalledTimes(2); // Initial + redo
        expect(state.currentIndex()).toBe(0);
        expect(state.canUndo()).toBe(true);
        expect(state.canRedo()).toBe(false);
      });
    });

    it("should return false when nothing to redo", async () => {
      await testWithRoot(async () => {
        const [, actions] = useHistory();
        
        const command: Command<number> = {
          type: "test",
          description: "Test",
          previousState: 0,
          newState: 1,
          timestamp: Date.now(),
          execute: vi.fn(),
          undo: vi.fn(),
        };

        await actions.execute(command);
        const result = await actions.redo();

        expect(result).toBe(false);
      });
    });

    it("should call onRedo callback", async () => {
      await testWithRoot(async () => {
        const onRedo = vi.fn();
        const [, actions] = useHistory({ onRedo });
        
        const command: Command<number> = {
          type: "test",
          description: "Test",
          previousState: 0,
          newState: 1,
          timestamp: Date.now(),
          execute: vi.fn(),
          undo: vi.fn(),
        };

        await actions.execute(command);
        await actions.undo();
        await actions.redo();

        expect(onRedo).toHaveBeenCalledWith(command);
      });
    });
  });

  describe("clear", () => {
    it("should clear all history", async () => {
      await testWithRoot(async () => {
        const [state, actions] = useHistory();
        
        const command: Command<number> = {
          type: "test",
          description: "Test",
          previousState: 0,
          newState: 1,
          timestamp: Date.now(),
          execute: vi.fn(),
          undo: vi.fn(),
        };

        await actions.execute(command);
        await actions.execute(command);
        await actions.execute(command);

        actions.clear();

        expect(state.historyLength()).toBe(0);
        expect(state.currentIndex()).toBe(-1);
        expect(state.canUndo()).toBe(false);
        expect(state.canRedo()).toBe(false);
      });
    });
  });

  describe("goTo", () => {
    it("should navigate to a specific point in history", async () => {
      await testWithRoot(async () => {
        const [state, actions] = useHistory();
        
        const commands: Command<number>[] = [];
        for (let i = 1; i <= 5; i++) {
          const cmd: Command<number> = {
            type: "test",
            description: `Command ${i}`,
            previousState: i - 1,
            newState: i,
            timestamp: Date.now(),
            execute: vi.fn(),
            undo: vi.fn(),
          };
          commands.push(cmd);
          await actions.execute(cmd);
        }

        // At index 4, go to index 1
        await actions.goTo(1);

        expect(state.currentIndex()).toBe(1);
        // Commands 5, 4, 3 should have been undone
        expect(commands[4].undo).toHaveBeenCalled();
        expect(commands[3].undo).toHaveBeenCalled();
        expect(commands[2].undo).toHaveBeenCalled();
      });
    });

    it("should redo commands when going forward", async () => {
      await testWithRoot(async () => {
        const [state, actions] = useHistory();
        
        const commands: Command<number>[] = [];
        for (let i = 1; i <= 3; i++) {
          const cmd: Command<number> = {
            type: "test",
            description: `Command ${i}`,
            previousState: i - 1,
            newState: i,
            timestamp: Date.now(),
            execute: vi.fn(),
            undo: vi.fn(),
          };
          commands.push(cmd);
          await actions.execute(cmd);
        }

        // Undo all
        await actions.goTo(-1);
        expect(state.currentIndex()).toBe(-1);

        // Go forward to index 2
        await actions.goTo(2);

        expect(state.currentIndex()).toBe(2);
        // All commands should have been re-executed
        expect(commands[0].execute).toHaveBeenCalledTimes(2);
        expect(commands[1].execute).toHaveBeenCalledTimes(2);
        expect(commands[2].execute).toHaveBeenCalledTimes(2);
      });
    });
  });

  describe("descriptions", () => {
    it("should return correct undo description", async () => {
      await testWithRoot(async () => {
        const [state, actions] = useHistory();
        
        const command: Command<number> = {
          type: "test",
          description: "Add item",
          previousState: 0,
          newState: 1,
          timestamp: Date.now(),
          execute: vi.fn(),
          undo: vi.fn(),
        };

        await actions.execute(command);

        expect(state.undoDescription()).toBe("Add item");
      });
    });

    it("should return correct redo description", async () => {
      await testWithRoot(async () => {
        const [state, actions] = useHistory();
        
        const command: Command<number> = {
          type: "test",
          description: "Add item",
          previousState: 0,
          newState: 1,
          timestamp: Date.now(),
          execute: vi.fn(),
          undo: vi.fn(),
        };

        await actions.execute(command);
        await actions.undo();

        expect(state.redoDescription()).toBe("Add item");
      });
    });
  });
});

describe("createStateCommand", () => {
  it("should create a command that updates signal", () => {
    testWithRoot(() => {
      const setter = (_v: number) => { /* no-op for test */ };
      
      const command = createStateCommand(
        "Set value to 10",
        setter,
        0,
        10
      );

      expect(command.type).toBe("state-change");
      expect(command.description).toBe("Set value to 10");
      expect(command.previousState).toBe(0);
      expect(command.newState).toBe(10);
    });
  });

  it("should execute and undo correctly", () => {
    testWithRoot(() => {
      let currentValue = 0;
      const setter = (v: number) => { currentValue = v; };
      
      const command = createStateCommand(
        "Set value",
        setter,
        0,
        10
      );

      command.execute();
      expect(currentValue).toBe(10);

      command.undo();
      expect(currentValue).toBe(0);
    });
  });
});

describe("createBatchCommand", () => {
  it("should combine multiple commands", async () => {
    await testWithRoot(async () => {
      const cmd1: Command<number> = {
        type: "test",
        description: "First",
        previousState: 0,
        newState: 1,
        timestamp: Date.now(),
        execute: vi.fn(),
        undo: vi.fn(),
      };
      
      const cmd2: Command<number> = {
        type: "test",
        description: "Second",
        previousState: 1,
        newState: 2,
        timestamp: Date.now(),
        execute: vi.fn(),
        undo: vi.fn(),
      };

      const batch = createBatchCommand("Batch operation", [cmd1, cmd2]);

      expect(batch.type).toBe("batch");
      expect(batch.description).toBe("Batch operation");

      await batch.execute();
      expect(cmd1.execute).toHaveBeenCalled();
      expect(cmd2.execute).toHaveBeenCalled();

      await batch.undo();
      // Should undo in reverse order
      expect(cmd2.undo).toHaveBeenCalled();
      expect(cmd1.undo).toHaveBeenCalled();
    });
  });
});

// Helper for running tests with root
function testWithRoot<T>(fn: () => T | Promise<T>): T | Promise<T> {
  let result!: T | Promise<T>;
  createRoot((dispose) => {
    result = fn();
    if (result instanceof Promise) {
      result.finally(dispose);
    } else {
      dispose();
    }
  });
  return result;
}
