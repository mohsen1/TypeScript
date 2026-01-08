/**
 * Tests for StateManager
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { StateManager } from '../src/state/index.js';
import { TmuxClient } from '../src/tmux/index.js';
import type { OrchestratorConfig, PaneContent } from '../src/types.js';
import * as fs from 'node:fs/promises';
import { existsSync } from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';

function createTestConfig(stateFile: string): OrchestratorConfig {
  return {
    session: 'test-session',
    agentType: 'claude',
    agentArgs: '--test',
    agentUpdateCmd: 'echo',
    autoFetch: true,
    autoRestartAgent: true,
    autoMonitor: true,
    autoAttach: true,
    agentAutoUpdate: true,
    idleThresholds: { director: 600, em: 60, worker: 300 },
    pokeMessages: { director: 'd', em: 'e', worker: 'w' },
    startupPrompts: { director: 'd', em: 'e', worker: 'w' },
    timing: {
      startPause: 10,
      sendEnterPause: 1,
      staggerPause: 2,
      agentRestartDelay: 2,
      mergePauseSeconds: 240,
    },
    rootDir: '/test/root',
    worktreeBase: '/test/worktrees',
    stateFile,
    squads: [
      { name: 'forge', workerCount: 5, focusAreas: ['solver/', 'checker/'] },
      { name: 'anvil', workerCount: 5, focusAreas: ['thin_emitter/', 'transforms/'] },
    ],
  };
}

describe('StateManager', () => {
  let tmux: TmuxClient;
  let state: StateManager;
  let testStateFile: string;

  beforeEach(async () => {
    // Create a unique test state file
    testStateFile = join(
      tmpdir(),
      `test-state-${Date.now()}-${Math.random().toString(36).slice(2)}.json`
    );

    tmux = {
      listWindows: vi.fn().mockResolvedValue(['director', 'forge', 'anvil']),
      getPaneCount: vi.fn().mockResolvedValue(3),
      capturePane: vi.fn().mockResolvedValue({
        paneId: { session: 's', window: 'w', pane: 0 },
        content: 'line1\nline2\nline3',
        hash: 'hash123',
      } as PaneContent),
      getPanePath: vi.fn().mockResolvedValue('/test/path'),
    } as unknown as TmuxClient;

    state = new StateManager(tmux, createTestConfig(testStateFile));
  });

  afterEach(async () => {
    // Clean up test state file
    if (existsSync(testStateFile)) {
      await fs.unlink(testStateFile);
    }
  });

  describe('stateFile', () => {
    it('returns the configured state file path', () => {
      expect(state.stateFile).toBe(testStateFile);
    });
  });

  describe('stateExists', () => {
    it('returns false when no state file', () => {
      expect(state.stateExists()).toBe(false);
    });

    it('returns true when state file exists', async () => {
      await fs.writeFile(testStateFile, '{}');
      expect(state.stateExists()).toBe(true);
    });
  });

  describe('saveState', () => {
    it('creates state file', async () => {
      await state.saveState();
      expect(existsSync(testStateFile)).toBe(true);
    });

    it('saves timestamp', async () => {
      await state.saveState();
      const content = await fs.readFile(testStateFile, 'utf-8');
      const parsed = JSON.parse(content);
      expect(parsed.timestamp).toBeDefined();
      expect(new Date(parsed.timestamp).getTime()).not.toBeNaN();
    });

    it('saves pane states', async () => {
      await state.saveState();
      const content = await fs.readFile(testStateFile, 'utf-8');
      const parsed = JSON.parse(content);
      expect(parsed.panes).toBeDefined();
      expect(typeof parsed.panes).toBe('object');
    });

    it('captures content from panes', async () => {
      vi.mocked(tmux.capturePane).mockResolvedValue({
        paneId: { session: 's', window: 'w', pane: 0 },
        content: 'captured content',
        hash: 'hash',
      });

      await state.saveState();
      const content = await fs.readFile(testStateFile, 'utf-8');
      const parsed = JSON.parse(content);

      // Should have captured some panes
      expect(Object.keys(parsed.panes).length).toBeGreaterThan(0);
    });
  });

  describe('loadState', () => {
    it('returns null when no state file', async () => {
      const result = await state.loadState();
      expect(result).toBeNull();
    });

    it('returns parsed state', async () => {
      const testState = {
        timestamp: new Date().toISOString(),
        panes: {
          director_0: { path: '/path', context: 'test context' },
        },
      };
      await fs.writeFile(testStateFile, JSON.stringify(testState));

      const result = await state.loadState();
      expect(result).not.toBeNull();
      expect(result?.timestamp).toBe(testState.timestamp);
      expect(result?.panes['director_0']?.context).toBe('test context');
    });

    it('returns null on parse error', async () => {
      await fs.writeFile(testStateFile, 'invalid json{{{');
      const result = await state.loadState();
      expect(result).toBeNull();
    });
  });

  describe('getResumePrompt', () => {
    it('returns null when no state', async () => {
      const prompt = await state.getResumePrompt('director_0');
      expect(prompt).toBeNull();
    });

    it('returns null for unknown pane', async () => {
      await fs.writeFile(
        testStateFile,
        JSON.stringify({
          timestamp: new Date().toISOString(),
          panes: {},
        })
      );

      const prompt = await state.getResumePrompt('unknown_pane');
      expect(prompt).toBeNull();
    });

    it('returns formatted resume prompt with context', async () => {
      await fs.writeFile(
        testStateFile,
        JSON.stringify({
          timestamp: new Date().toISOString(),
          panes: {
            director_0: { path: '/path', context: 'Previous work here' },
          },
        })
      );

      const prompt = await state.getResumePrompt('director_0');
      expect(prompt).toContain('RESUMING SESSION');
      expect(prompt).toContain('Previous work here');
      expect(prompt).toContain('Continue where you left off');
    });
  });

  describe('deleteState', () => {
    it('deletes state file if exists', async () => {
      await fs.writeFile(testStateFile, '{}');
      expect(existsSync(testStateFile)).toBe(true);

      await state.deleteState();
      expect(existsSync(testStateFile)).toBe(false);
    });

    it('does not throw if file does not exist', async () => {
      await expect(state.deleteState()).resolves.not.toThrow();
    });
  });

  describe('getStateAge', () => {
    it('returns null when no state', async () => {
      const age = await state.getStateAge();
      expect(age).toBeNull();
    });

    it('returns age in seconds', async () => {
      const timestamp = new Date(Date.now() - 60000).toISOString(); // 1 minute ago
      await fs.writeFile(
        testStateFile,
        JSON.stringify({ timestamp, panes: {} })
      );

      const age = await state.getStateAge();
      expect(age).not.toBeNull();
      expect(age).toBeGreaterThanOrEqual(59);
      expect(age).toBeLessThan(70);
    });
  });

  describe('formatStateInfo', () => {
    it('returns message when no state', async () => {
      const info = await state.formatStateInfo();
      expect(info).toBe('No saved state found');
    });

    it('returns formatted info', async () => {
      await fs.writeFile(
        testStateFile,
        JSON.stringify({
          timestamp: new Date().toISOString(),
          panes: { a: {}, b: {}, c: {} },
        })
      );

      const info = await state.formatStateInfo();
      expect(info).toContain('State file:');
      expect(info).toContain('Saved:');
      expect(info).toContain('Age:');
      expect(info).toContain('Panes captured: 3');
    });
  });
});
