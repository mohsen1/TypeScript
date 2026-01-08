/**
 * Tests for AgentManager
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { AgentManager } from '../src/agent/index.js';
import { TmuxClient } from '../src/tmux/index.js';
import type { OrchestratorConfig, PaneId } from '../src/types.js';

// Create a minimal config for testing
function createTestConfig(
  overrides: Partial<OrchestratorConfig> = {}
): OrchestratorConfig {
  return {
    session: 'test-session',
    agentType: 'claude',
    agentArgs: '--dangerously-skip-permissions',
    agentUpdateCmd: 'npm install -g @anthropic-ai/claude-code',
    autoFetch: true,
    autoRestartAgent: true,
    autoMonitor: true,
    autoAttach: true,
    agentAutoUpdate: true,
    idleThresholds: { director: 600, em: 60, worker: 300 },
    pokeMessages: {
      director: 'Director poke',
      em: 'EM poke',
      worker: 'Worker poke',
    },
    startupPrompts: {
      director: 'Director startup',
      em: 'EM startup for $SQUAD_NAME',
      worker: 'Worker $WORKER_NUM in $SQUAD_NAME',
    },
    timing: {
      startPause: 0, // Use 0 for tests to speed them up
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
    ...overrides,
  };
}

describe('AgentManager', () => {
  let tmux: TmuxClient;
  let agent: AgentManager;

  beforeEach(() => {
    // Mock TmuxClient
    tmux = {
      sendKeys: vi.fn().mockResolvedValue(undefined),
      sendEscape: vi.fn().mockResolvedValue(undefined),
    } as unknown as TmuxClient;

    agent = new AgentManager(tmux, createTestConfig());
  });

  describe('getAgentCommand', () => {
    it('returns claude for claude agent type', () => {
      expect(agent.getAgentCommand()).toBe('claude');
    });

    it('returns codex for codex agent type', () => {
      const codexAgent = new AgentManager(
        tmux,
        createTestConfig({ agentType: 'codex' })
      );
      expect(codexAgent.getAgentCommand()).toBe('codex');
    });
  });

  describe('getAgentRunCommand', () => {
    it('includes restart loop when autoRestartAgent is true', () => {
      const cmd = agent.getAgentRunCommand();
      expect(cmd).toContain('while true');
      expect(cmd).toContain('sleep');
    });

    it('returns simple command when autoRestartAgent is false', () => {
      const agentNoRestart = new AgentManager(
        tmux,
        createTestConfig({ autoRestartAgent: false })
      );
      const cmd = agentNoRestart.getAgentRunCommand();
      expect(cmd).not.toContain('while true');
      expect(cmd).toBe('claude --dangerously-skip-permissions');
    });
  });

  describe('getStartupPrompt', () => {
    it('returns director prompt unchanged', () => {
      const prompt = agent.getStartupPrompt('director');
      expect(prompt).toBe('Director startup');
    });

    it('substitutes squad name for EM', () => {
      const prompt = agent.getStartupPrompt('em', 'forge');
      expect(prompt).toBe('EM startup for forge');
    });

    it('substitutes worker num and squad for worker', () => {
      const prompt = agent.getStartupPrompt('worker', 'anvil', 3);
      expect(prompt).toBe('Worker 3 in anvil');
    });

    it('handles ${WORKER_NUM} syntax', () => {
      const customAgent = new AgentManager(
        tmux,
        createTestConfig({
          startupPrompts: {
            director: 'd',
            em: 'e',
            worker: 'Worker ${WORKER_NUM} here',
          },
        })
      );
      const prompt = customAgent.getStartupPrompt('worker', 'forge', 5);
      expect(prompt).toBe('Worker 5 here');
    });
  });

  describe('getPokeMessage', () => {
    it('returns correct poke message for each role', () => {
      expect(agent.getPokeMessage('director')).toBe('Director poke');
      expect(agent.getPokeMessage('em')).toBe('EM poke');
      expect(agent.getPokeMessage('worker')).toBe('Worker poke');
    });
  });

  describe('sendPrompt', () => {
    it('sends prompt text then enter', async () => {
      const pane: PaneId = { session: 's', window: 'w', pane: 0 };
      await agent.sendPrompt(pane, 'test prompt');

      expect(tmux.sendKeys).toHaveBeenCalledTimes(2);
      expect(tmux.sendKeys).toHaveBeenNthCalledWith(1, pane, 'test prompt');
      expect(tmux.sendKeys).toHaveBeenNthCalledWith(2, pane, '', true);
    });
  });

  describe('cancelOperation', () => {
    it('sends escape to pane', async () => {
      const pane: PaneId = { session: 's', window: 'w', pane: 0 };
      await agent.cancelOperation(pane);

      expect(tmux.sendEscape).toHaveBeenCalledWith(pane);
    });
  });

  describe('pokeAgent', () => {
    it('sends appropriate poke message for role', async () => {
      const pane: PaneId = { session: 's', window: 'w', pane: 0 };

      await agent.pokeAgent(pane, 'em');

      expect(tmux.sendKeys).toHaveBeenCalledWith(pane, 'EM poke');
    });
  });
});
