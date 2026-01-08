/**
 * Integration tests for Orchestrator
 *
 * These tests verify the orchestrator's coordination logic
 * with mocked external dependencies (tmux, git, fs).
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import type { OrchestratorConfig } from '../src/types.js';

// We need to test the orchestrator components individually since
// full orchestrator integration requires actual tmux/git/fs access.
// These tests focus on the logic without actual system calls.

function createTestConfig(): OrchestratorConfig {
  return {
    session: 'test-session',
    agentType: 'claude',
    agentArgs: '--dangerously-skip-permissions',
    agentUpdateCmd: 'echo update',
    autoFetch: false,
    autoRestartAgent: true,
    autoMonitor: false,
    autoAttach: false,
    agentAutoUpdate: false,
    idleThresholds: { director: 600, em: 60, worker: 300 },
    pokeMessages: { director: 'd', em: 'e', worker: 'w' },
    startupPrompts: { director: 'd', em: 'e $SQUAD_NAME', worker: 'w $WORKER_NUM' },
    timing: {
      startPause: 0,
      sendEnterPause: 0,
      staggerPause: 0,
      agentRestartDelay: 2,
      mergePauseSeconds: 240,
    },
    rootDir: '/test/TypeScript',
    worktreeBase: '/test',
    stateFile: '/test/state.json',
    squads: [
      { name: 'forge', workerCount: 5, focusAreas: ['solver/', 'checker/'] },
      { name: 'anvil', workerCount: 5, focusAreas: ['thin_emitter/', 'transforms/'] },
    ],
  };
}

describe('Orchestrator configuration', () => {
  it('creates valid config with all required fields', () => {
    const config = createTestConfig();

    expect(config.session).toBe('test-session');
    expect(config.agentType).toBe('claude');
    expect(config.idleThresholds.director).toBe(600);
    expect(config.idleThresholds.em).toBe(60);
    expect(config.idleThresholds.worker).toBe(300);
  });

  it('supports codex agent type', () => {
    const config = { ...createTestConfig(), agentType: 'codex' as const };
    expect(config.agentType).toBe('codex');
  });
});

describe('Orchestrator module structure', () => {
  it('exports Orchestrator class', async () => {
    const mod = await import('../src/Orchestrator.js');
    expect(mod.Orchestrator).toBeDefined();
    expect(typeof mod.Orchestrator).toBe('function');
  });

  it('exports all required modules from index', async () => {
    const mod = await import('../src/index.js');

    expect(mod.Orchestrator).toBeDefined();
    expect(mod.TmuxClient).toBeDefined();
    expect(mod.AgentManager).toBeDefined();
    expect(mod.IdleMonitor).toBeDefined();
    expect(mod.StateManager).toBeDefined();
    expect(mod.WorktreeManager).toBeDefined();
    expect(mod.createConfig).toBeDefined();
  });
});

describe('Orchestrator options', () => {
  it('accepts start mode', async () => {
    const { Orchestrator } = await import('../src/Orchestrator.js');
    const config = createTestConfig();

    const orchestrator = new Orchestrator({ config, mode: 'start' });
    expect(orchestrator).toBeDefined();
  });

  it('accepts kill mode', async () => {
    const { Orchestrator } = await import('../src/Orchestrator.js');
    const config = createTestConfig();

    const orchestrator = new Orchestrator({ config, mode: 'kill' });
    expect(orchestrator).toBeDefined();
  });

  it('accepts resume mode', async () => {
    const { Orchestrator } = await import('../src/Orchestrator.js');
    const config = createTestConfig();

    const orchestrator = new Orchestrator({ config, mode: 'resume' });
    expect(orchestrator).toBeDefined();
  });

  it('accepts fresh mode', async () => {
    const { Orchestrator } = await import('../src/Orchestrator.js');
    const config = createTestConfig();

    const orchestrator = new Orchestrator({ config, mode: 'fresh' });
    expect(orchestrator).toBeDefined();
  });

  it('defaults to start mode', async () => {
    const { Orchestrator } = await import('../src/Orchestrator.js');
    const config = createTestConfig();

    const orchestrator = new Orchestrator({ config });
    expect(orchestrator).toBeDefined();
  });
});

describe('CLI module', () => {
  it('exports main function (cli.js)', async () => {
    // Just verify the module can be loaded
    const cliPath = '../src/cli.js';
    await expect(import(cliPath)).resolves.toBeDefined();
  });
});

describe('Types exports', () => {
  it('exports all core types', async () => {
    const mod = await import('../src/types.js');

    // Type exports are verified at compile time, but we can check the module loads
    expect(mod).toBeDefined();
  });
});

describe('Config validation', () => {
  it('validateConfig catches invalid thresholds', async () => {
    const { validateConfig } = await import('../src/config/index.js');

    const invalidConfig: OrchestratorConfig = {
      ...createTestConfig(),
      idleThresholds: { director: -1, em: 60, worker: 300 },
    };

    const errors = validateConfig(invalidConfig);
    expect(errors.length).toBeGreaterThan(0);
    expect(errors.some((e) => e.includes('Director'))).toBe(true);
  });

  it('validateConfig passes for valid config with existing paths', async () => {
    const { validateConfig } = await import('../src/config/index.js');

    // Use /tmp which exists on all systems
    const validConfig: OrchestratorConfig = {
      ...createTestConfig(),
      rootDir: '/tmp',
      worktreeBase: '/tmp',
    };

    const errors = validateConfig(validConfig);
    expect(errors).toHaveLength(0);
  });
});

describe('Worktree paths', () => {
  it('generates correct worker worktree paths', async () => {
    const { WorktreeManager } = await import('../src/worktree/index.js');
    const config = createTestConfig();
    const manager = new WorktreeManager(config);

    expect(manager.getWorkerWorktreePath('forge', 1)).toBe(
      '/test/TypeScript-forge-1-track'
    );
    expect(manager.getWorkerWorktreePath('anvil', 5)).toBe(
      '/test/TypeScript-anvil-5-track'
    );
  });

  it('generates correct EM worktree paths', async () => {
    const { WorktreeManager } = await import('../src/worktree/index.js');
    const config = createTestConfig();
    const manager = new WorktreeManager(config);

    expect(manager.getEmWorktreePath('forge')).toBe('/test/TypeScript-em-forge');
    expect(manager.getEmWorktreePath('anvil')).toBe('/test/TypeScript-em-anvil');
  });

  it('generates correct branch names', async () => {
    const { WorktreeManager } = await import('../src/worktree/index.js');
    const config = createTestConfig();
    const manager = new WorktreeManager(config);

    expect(manager.getWorkerBranch('forge', 1)).toBe('worker/forge-1');
    expect(manager.getEmBranch('anvil')).toBe('em/anvil');
    expect(manager.getSquadBranch('forge')).toBe('squad/forge');
  });
});

describe('Agent prompts', () => {
  it('substitutes squad name in EM prompts', async () => {
    const { AgentManager } = await import('../src/agent/index.js');
    const { TmuxClient } = await import('../src/tmux/index.js');

    const config = createTestConfig();
    const tmux = new TmuxClient();
    const agent = new AgentManager(tmux, config);

    const prompt = agent.getStartupPrompt('em', 'forge');
    expect(prompt).toBe('e forge');
  });

  it('substitutes worker number in worker prompts', async () => {
    const { AgentManager } = await import('../src/agent/index.js');
    const { TmuxClient } = await import('../src/tmux/index.js');

    const config = createTestConfig();
    const tmux = new TmuxClient();
    const agent = new AgentManager(tmux, config);

    const prompt = agent.getStartupPrompt('worker', 'anvil', 3);
    expect(prompt).toBe('w 3');
  });
});

describe('Pane identification', () => {
  it('creates valid PaneId objects', async () => {
    const { TmuxClient } = await import('../src/tmux/index.js');

    const pane = TmuxClient.paneId('session', 'window', 0);
    expect(pane.session).toBe('session');
    expect(pane.window).toBe('window');
    expect(pane.pane).toBe(0);
  });
});
