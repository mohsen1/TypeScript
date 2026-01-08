/**
 * Tests for IdleMonitor
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { IdleMonitor } from '../src/monitor/index.js';
import { TmuxClient } from '../src/tmux/index.js';
import { AgentManager } from '../src/agent/index.js';
import type { OrchestratorConfig, PaneId, PaneContent } from '../src/types.js';

function createTestConfig(): OrchestratorConfig {
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
    idleThresholds: { director: 10, em: 5, worker: 8 },
    pokeMessages: {
      director: 'Director poke',
      em: 'EM poke',
      worker: 'Worker poke',
    },
    startupPrompts: { director: 'd', em: 'e', worker: 'w' },
    timing: {
      startPause: 0,
      sendEnterPause: 0,
      staggerPause: 0,
      agentRestartDelay: 2,
      mergePauseSeconds: 240,
    },
    rootDir: '/test/root',
    worktreeBase: '/test/worktrees',
    stateFile: '/test/state.json',
    squads: [
      { name: 'forge', workerCount: 5, focusAreas: ['solver/', 'checker/'] },
      { name: 'anvil', workerCount: 5, focusAreas: ['thin_emitter/', 'transforms/'] },
    ],
  };
}

describe('IdleMonitor', () => {
  let tmux: TmuxClient;
  let agent: AgentManager;
  let monitor: IdleMonitor;
  let config: OrchestratorConfig;

  beforeEach(() => {
    config = createTestConfig();

    tmux = {
      sessionExists: vi.fn().mockResolvedValue(true),
      capturePane: vi.fn().mockResolvedValue({
        paneId: { session: 's', window: 'w', pane: 0 },
        content: 'test content',
        hash: 'hash123',
      } as PaneContent),
    } as unknown as TmuxClient;

    agent = {
      pokeAgent: vi.fn().mockResolvedValue(undefined),
    } as unknown as AgentManager;

    monitor = new IdleMonitor(tmux, agent, config);
  });

  afterEach(() => {
    monitor.stop();
    monitor.reset();
  });

  describe('getThreshold', () => {
    it('returns correct threshold for each role', () => {
      expect(monitor.getThreshold('director')).toBe(10);
      expect(monitor.getThreshold('em')).toBe(5);
      expect(monitor.getThreshold('worker')).toBe(8);
    });
  });

  describe('registerPanes', () => {
    it('registers panes for monitoring', () => {
      const pane: PaneId = { session: 's', window: 'w', pane: 0 };
      monitor.registerPanes([{ pane, role: 'worker' }]);

      // Internal state - we can verify via behavior
      expect(() => monitor.addPane(pane, 'em')).not.toThrow();
    });
  });

  describe('addPane', () => {
    it('adds single pane', () => {
      const pane: PaneId = { session: 's', window: 'w', pane: 0 };
      expect(() => monitor.addPane(pane, 'worker')).not.toThrow();
    });
  });

  describe('updatePaneState', () => {
    it('initializes state for new pane', async () => {
      const pane: PaneId = { session: 's', window: 'w', pane: 0 };

      await monitor.updatePaneState(pane);

      const state = monitor.getPaneState(pane);
      expect(state).toBeDefined();
      expect(state?.lastHash).toBe('hash123');
      expect(state?.lastChangeTime).toBeGreaterThan(0);
    });

    it('updates change time when content changes', async () => {
      const pane: PaneId = { session: 's', window: 'w', pane: 0 };

      // First capture
      await monitor.updatePaneState(pane);
      const state1 = monitor.getPaneState(pane);

      // Wait a tiny bit
      await new Promise((r) => setTimeout(r, 10));

      // Change content hash
      vi.mocked(tmux.capturePane).mockResolvedValueOnce({
        paneId: pane,
        content: 'new content',
        hash: 'hash456',
      });

      await monitor.updatePaneState(pane);
      const state2 = monitor.getPaneState(pane);

      expect(state2?.lastHash).toBe('hash456');
      expect(state2!.lastChangeTime).toBeGreaterThanOrEqual(state1!.lastChangeTime);
    });

    it('preserves change time when content unchanged', async () => {
      const pane: PaneId = { session: 's', window: 'w', pane: 0 };

      // First capture
      await monitor.updatePaneState(pane);
      const state1 = monitor.getPaneState(pane);

      // Wait a bit
      await new Promise((r) => setTimeout(r, 50));

      // Same content
      await monitor.updatePaneState(pane);
      const state2 = monitor.getPaneState(pane);

      // Change time should be the same (pane is idle)
      expect(state2?.lastChangeTime).toBe(state1?.lastChangeTime);
    });
  });

  describe('checkPaneIdle', () => {
    it('returns false for new pane (no state)', async () => {
      const pane: PaneId = { session: 's', window: 'w', pane: 0 };
      const isIdle = await monitor.checkPaneIdle(pane, 'worker');
      expect(isIdle).toBe(false);
    });

    it('returns false when below threshold', async () => {
      const pane: PaneId = { session: 's', window: 'w', pane: 0 };

      // Initialize state (lastChangeTime = now)
      await monitor.updatePaneState(pane);

      // Check immediately - should not be idle
      const isIdle = await monitor.checkPaneIdle(pane, 'worker');
      expect(isIdle).toBe(false);
    });

    it('returns false when threshold is 0', async () => {
      const zeroConfig = {
        ...config,
        idleThresholds: { director: 0, em: 0, worker: 0 },
      };
      const zeroMonitor = new IdleMonitor(tmux, agent, zeroConfig);

      const pane: PaneId = { session: 's', window: 'w', pane: 0 };
      await zeroMonitor.updatePaneState(pane);

      // Even with old state, threshold 0 means no idle check
      const isIdle = await zeroMonitor.checkPaneIdle(pane, 'worker');
      expect(isIdle).toBe(false);
    });
  });

  describe('cycle', () => {
    it('stops if session no longer exists', async () => {
      vi.mocked(tmux.sessionExists).mockResolvedValue(false);

      monitor.start(100);
      expect(monitor.isRunning()).toBe(true);

      await monitor.cycle();

      expect(monitor.isRunning()).toBe(false);
    });

    it('updates state for all registered panes', async () => {
      const pane1: PaneId = { session: 's', window: 'w', pane: 0 };
      const pane2: PaneId = { session: 's', window: 'w', pane: 1 };

      monitor.registerPanes([
        { pane: pane1, role: 'worker' },
        { pane: pane2, role: 'em' },
      ]);

      await monitor.cycle();

      expect(monitor.getPaneState(pane1)).toBeDefined();
      expect(monitor.getPaneState(pane2)).toBeDefined();
    });
  });

  describe('start/stop', () => {
    it('starts monitoring', () => {
      expect(monitor.isRunning()).toBe(false);
      monitor.start(100);
      expect(monitor.isRunning()).toBe(true);
    });

    it('stops monitoring', () => {
      monitor.start(100);
      monitor.stop();
      expect(monitor.isRunning()).toBe(false);
    });

    it('does not start twice', () => {
      monitor.start(100);
      monitor.start(100); // Should not throw
      expect(monitor.isRunning()).toBe(true);
    });
  });

  describe('reset', () => {
    it('clears all state', async () => {
      const pane: PaneId = { session: 's', window: 'w', pane: 0 };
      monitor.addPane(pane, 'worker');
      await monitor.updatePaneState(pane);

      expect(monitor.getPaneState(pane)).toBeDefined();

      monitor.reset();

      expect(monitor.getPaneState(pane)).toBeUndefined();
    });
  });
});
