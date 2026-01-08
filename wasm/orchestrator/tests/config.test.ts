/**
 * Tests for configuration module
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import {
  createConfig,
  configFromEnv,
  validateConfig,
} from '../src/config/index.js';
import type { OrchestratorConfig } from '../src/types.js';

// Mock fs.existsSync for path validation
vi.mock('node:fs', async () => {
  const actual = await vi.importActual('node:fs');
  return {
    ...actual,
    existsSync: vi.fn().mockReturnValue(true),
  };
});

describe('createConfig', () => {
  it('creates config with defaults', () => {
    const config = createConfig({ rootDir: '/test/path' });

    expect(config.session).toBe('zang-org');
    expect(config.agentType).toBe('claude');
    expect(config.autoFetch).toBe(true);
    expect(config.autoRestartAgent).toBe(true);
    expect(config.autoMonitor).toBe(true);
    expect(config.autoAttach).toBe(true);
  });

  it('uses claude agent settings by default', () => {
    const config = createConfig({ rootDir: '/test/path' });

    expect(config.agentArgs).toBe('--dangerously-skip-permissions');
    expect(config.agentUpdateCmd).toContain('@anthropic-ai/claude-code');
  });

  it('uses codex agent settings when specified', () => {
    const config = createConfig({
      rootDir: '/test/path',
      agentType: 'codex',
    });

    expect(config.agentType).toBe('codex');
    expect(config.agentArgs).toBe('--dangerously-bypass-approvals-and-sandbox');
    expect(config.agentUpdateCmd).toContain('@openai/codex');
  });

  it('sets default idle thresholds', () => {
    const config = createConfig({ rootDir: '/test/path' });

    expect(config.idleThresholds.director).toBe(600);
    expect(config.idleThresholds.em).toBe(60);
    expect(config.idleThresholds.worker).toBe(300);
  });

  it('sets default timing', () => {
    const config = createConfig({ rootDir: '/test/path' });

    expect(config.timing.startPause).toBe(10);
    expect(config.timing.sendEnterPause).toBe(1);
    expect(config.timing.staggerPause).toBe(2);
    expect(config.timing.agentRestartDelay).toBe(2);
    expect(config.timing.mergePauseSeconds).toBe(240);
  });

  it('applies overrides', () => {
    const config = createConfig({
      rootDir: '/test/path',
      overrides: {
        autoFetch: false,
        idleThresholds: {
          director: 900,
          em: 120,
          worker: 180,
        },
      },
    });

    expect(config.autoFetch).toBe(false);
    expect(config.idleThresholds.director).toBe(900);
    expect(config.idleThresholds.em).toBe(120);
    expect(config.idleThresholds.worker).toBe(180);
  });

  it('uses custom session name', () => {
    const config = createConfig({
      rootDir: '/test/path',
      session: 'custom-session',
    });

    expect(config.session).toBe('custom-session');
  });
});

describe('configFromEnv', () => {
  const originalEnv = { ...process.env };

  beforeEach(() => {
    // Clean env vars
    delete process.env['SESSION'];
    delete process.env['AUTO_FETCH'];
    delete process.env['AUTO_RESTART_AGENT'];
    delete process.env['DIRECTOR_IDLE_SECONDS'];
    delete process.env['EM_IDLE_SECONDS'];
    delete process.env['WORKER_IDLE_SECONDS'];
    delete process.env['DIRECTOR_POKE'];
    delete process.env['EM_POKE'];
    delete process.env['WORKER_POKE'];
  });

  afterEach(() => {
    process.env = { ...originalEnv };
  });

  it('returns empty object when no env vars set', () => {
    const result = configFromEnv();
    expect(Object.keys(result)).toHaveLength(0);
  });

  it('reads session from env', () => {
    process.env['SESSION'] = 'my-session';
    const result = configFromEnv();
    expect(result.session).toBe('my-session');
  });

  it('reads AUTO_FETCH=0 as false', () => {
    process.env['AUTO_FETCH'] = '0';
    const result = configFromEnv();
    expect(result.autoFetch).toBe(false);
  });

  it('reads idle thresholds from env', () => {
    process.env['DIRECTOR_IDLE_SECONDS'] = '1200';
    process.env['EM_IDLE_SECONDS'] = '90';
    process.env['WORKER_IDLE_SECONDS'] = '600';

    const result = configFromEnv();
    expect(result.idleThresholds?.director).toBe(1200);
    expect(result.idleThresholds?.em).toBe(90);
    expect(result.idleThresholds?.worker).toBe(600);
  });

  it('reads poke messages from env', () => {
    process.env['DIRECTOR_POKE'] = 'Custom director poke';
    process.env['EM_POKE'] = 'Custom EM poke';
    process.env['WORKER_POKE'] = 'Custom worker poke';

    const result = configFromEnv();
    expect(result.pokeMessages?.director).toBe('Custom director poke');
    expect(result.pokeMessages?.em).toBe('Custom EM poke');
    expect(result.pokeMessages?.worker).toBe('Custom worker poke');
  });
});

describe('validateConfig', () => {
  it('returns no errors for valid config', () => {
    // Create a minimal valid config
    const config: OrchestratorConfig = {
      session: 'test',
      agentType: 'claude',
      agentArgs: '--test',
      agentUpdateCmd: 'echo update',
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
      rootDir: '/tmp', // Use /tmp which exists
      worktreeBase: '/tmp',
      stateFile: '/tmp/state.json',
      squads: [
        { name: 'forge', workerCount: 5, focusAreas: ['solver/', 'checker/'] },
        { name: 'anvil', workerCount: 5, focusAreas: ['thin_emitter/', 'transforms/'] },
      ],
    };

    const errors = validateConfig(config);
    expect(errors).toHaveLength(0);
  });

  it('returns error for negative idle thresholds', () => {
    const config: OrchestratorConfig = {
      session: 'test',
      agentType: 'claude',
      agentArgs: '--test',
      agentUpdateCmd: 'echo',
      autoFetch: true,
      autoRestartAgent: true,
      autoMonitor: true,
      autoAttach: true,
      agentAutoUpdate: true,
      idleThresholds: { director: -1, em: 60, worker: 300 },
      pokeMessages: { director: 'd', em: 'e', worker: 'w' },
      startupPrompts: { director: 'd', em: 'e', worker: 'w' },
      timing: {
        startPause: 10,
        sendEnterPause: 1,
        staggerPause: 2,
        agentRestartDelay: 2,
        mergePauseSeconds: 240,
      },
      rootDir: '/tmp',
      worktreeBase: '/tmp',
      stateFile: '/tmp/state.json',
      squads: [
        { name: 'forge', workerCount: 5, focusAreas: ['solver/', 'checker/'] },
        { name: 'anvil', workerCount: 5, focusAreas: ['thin_emitter/', 'transforms/'] },
      ],
    };

    const errors = validateConfig(config);
    expect(errors.length).toBeGreaterThan(0);
    expect(errors.some((e) => e.includes('Director idle threshold'))).toBe(true);
  });
});
