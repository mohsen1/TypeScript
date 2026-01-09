/**
 * Tests for WorktreeManager
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { WorktreeManager } from '../src/worktree/index.js';
import type { OrchestratorConfig, SquadName } from '../src/types.js';

// Mock exec module
vi.mock('../src/utils/exec.js', () => ({
  runCommand: vi.fn().mockResolvedValue({ stdout: '', stderr: '', exitCode: 0 }),
  runCommandStrict: vi.fn().mockResolvedValue(''),
}));

// Mock fs modules
vi.mock('node:fs', () => ({
  existsSync: vi.fn().mockReturnValue(true),
  mkdirSync: vi.fn(),
  copyFileSync: vi.fn(),
  readFileSync: vi.fn().mockReturnValue(''),
  writeFileSync: vi.fn(),
}));

vi.mock('node:fs/promises', () => ({
  writeFile: vi.fn().mockResolvedValue(undefined),
}));

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
    rootDir: '/test/TypeScript',
    worktreeBase: '/test',
    stateFile: '/test/state.json',
    squads: [
      { name: 'forge', workerCount: 5, focusAreas: ['solver/', 'checker/'] },
      { name: 'anvil', workerCount: 5, focusAreas: ['thin_emitter/', 'transforms/'] },
    ],
  };
}

describe('WorktreeManager', () => {
  let manager: WorktreeManager;
  let mockRunCommand: ReturnType<typeof vi.fn>;

  beforeEach(async () => {
    manager = new WorktreeManager(createTestConfig());
    const execModule = await import('../src/utils/exec.js');
    mockRunCommand = vi.mocked(execModule.runCommand);
    mockRunCommand.mockReset();
    mockRunCommand.mockResolvedValue({ stdout: '', stderr: '', exitCode: 0 });
  });

  describe('getWorkerWorktreePath', () => {
    it('returns correct path for worker', () => {
      const path = manager.getWorkerWorktreePath('forge', 1);
      expect(path).toBe('/test/TypeScript-forge-1-track');
    });

    it('returns correct path for different workers', () => {
      expect(manager.getWorkerWorktreePath('anvil', 3)).toBe(
        '/test/TypeScript-anvil-3-track'
      );
      expect(manager.getWorkerWorktreePath('forge', 5)).toBe(
        '/test/TypeScript-forge-5-track'
      );
    });
  });

  describe('getEmWorktreePath', () => {
    it('returns correct path for EM', () => {
      expect(manager.getEmWorktreePath('forge')).toBe('/test/TypeScript-em-forge');
      expect(manager.getEmWorktreePath('anvil')).toBe('/test/TypeScript-em-anvil');
    });
  });

  describe('getWorkerBranch', () => {
    it('returns correct branch name', () => {
      expect(manager.getWorkerBranch('forge', 1)).toBe('worker/forge-1');
      expect(manager.getWorkerBranch('anvil', 5)).toBe('worker/anvil-5');
    });
  });

  describe('getEmBranch', () => {
    it('returns correct branch name', () => {
      expect(manager.getEmBranch('forge')).toBe('em/forge');
      expect(manager.getEmBranch('anvil')).toBe('em/anvil');
    });
  });

  describe('getSquadBranch', () => {
    it('returns correct branch name', () => {
      expect(manager.getSquadBranch('forge')).toBe('squad/forge');
      expect(manager.getSquadBranch('anvil')).toBe('squad/anvil');
    });
  });

  describe('fetch', () => {
    it('fetches when autoFetch is true', async () => {
      await manager.fetch();
      expect(mockRunCommand).toHaveBeenCalledWith(
        'git fetch --prune origin',
        expect.any(Object)
      );
    });

    it('skips fetch when autoFetch is false', async () => {
      const noFetchConfig = { ...createTestConfig(), autoFetch: false };
      const noFetchManager = new WorktreeManager(noFetchConfig);

      await noFetchManager.fetch();
      expect(mockRunCommand).not.toHaveBeenCalled();
    });
  });

  describe('ensureRustBranch', () => {
    it('creates branch from origin if local does not exist', async () => {
      // First call: check local (not found)
      // Second call: check remote (found)
      // Third call: create branch
      mockRunCommand
        .mockResolvedValueOnce({ stdout: '', stderr: '', exitCode: 1 })
        .mockResolvedValueOnce({ stdout: '', stderr: '', exitCode: 0 })
        .mockResolvedValueOnce({ stdout: '', stderr: '', exitCode: 0 });

      await manager.ensureRustBranch();

      expect(mockRunCommand).toHaveBeenCalledWith(
        expect.stringContaining('refs/heads/rust'),
        expect.any(Object)
      );
      expect(mockRunCommand).toHaveBeenCalledWith(
        expect.stringContaining('git branch --track rust origin/rust'),
        expect.any(Object)
      );
    });

    it('does nothing if rust branch already exists', async () => {
      mockRunCommand.mockResolvedValue({ stdout: '', stderr: '', exitCode: 0 });

      await manager.ensureRustBranch();

      // Should only check for local branch existence
      expect(mockRunCommand).toHaveBeenCalledTimes(1);
    });
  });

  describe('isWorktree', () => {
    it('returns true when path is in worktree list', async () => {
      mockRunCommand.mockResolvedValue({
        stdout: '/test/TypeScript-forge-1-track\n/test/TypeScript-anvil-1-track',
        stderr: '',
        exitCode: 0,
      });

      const result = await manager.isWorktree('/test/TypeScript-forge-1-track');
      expect(result).toBe(true);
    });

    it('returns false when path is not in worktree list', async () => {
      mockRunCommand.mockResolvedValue({
        stdout: '/other/path',
        stderr: '',
        exitCode: 0,
      });

      const result = await manager.isWorktree('/test/TypeScript-forge-1-track');
      expect(result).toBe(false);
    });
  });

  describe('ensureAllWorktrees', () => {
    it('creates worktrees for all workers and EMs', async () => {
      const fsModule = await import('node:fs');
      vi.mocked(fsModule.existsSync).mockReturnValue(false);

      const worktrees = await manager.ensureAllWorktrees();

      // Should have 2 EMs + 10 workers = 12 worktrees
      expect(worktrees.size).toBe(12);

      // Check EM entries
      expect(worktrees.has('em-forge')).toBe(true);
      expect(worktrees.has('em-anvil')).toBe(true);

      // Check worker entries
      for (const squad of ['forge', 'anvil'] as SquadName[]) {
        for (let i = 1; i <= 5; i++) {
          expect(worktrees.has(`${squad}-${i}`)).toBe(true);
        }
      }
    });
  });

  describe('resetAllBranches', () => {
    it('resets squad and worker branches', async () => {
      await manager.resetAllBranches();

      // Should delete and recreate squad branches
      expect(mockRunCommand).toHaveBeenCalledWith(
        expect.stringContaining('git branch -D "squad/forge"'),
        expect.any(Object)
      );
      expect(mockRunCommand).toHaveBeenCalledWith(
        expect.stringContaining('git branch "squad/forge" origin/rust'),
        expect.any(Object)
      );

      // Should delete and recreate worker branches
      expect(mockRunCommand).toHaveBeenCalledWith(
        expect.stringContaining('git branch -D "worker/forge-1"'),
        expect.any(Object)
      );
    });
  });

  describe('setupRoleAgents', () => {
    it('creates .role directories and copies AGENTS.md', async () => {
      const fsModule = await import('node:fs');
      // Ensure existsSync returns true for both directory and source file checks
      vi.mocked(fsModule.existsSync).mockReturnValue(true);

      await manager.setupRoleAgents();

      // Should create directories
      expect(fsModule.mkdirSync).toHaveBeenCalled();

      // Should copy files (source files exist so copyFileSync is called)
      expect(fsModule.copyFileSync).toHaveBeenCalled();
    });
  });

  describe('setupSquadSpecs', () => {
    it('creates squad directories', async () => {
      const fsModule = await import('node:fs');
      vi.mocked(fsModule.existsSync).mockReturnValue(false);

      await manager.setupSquadSpecs();

      expect(fsModule.mkdirSync).toHaveBeenCalledWith(
        expect.stringContaining('squads/forge'),
        expect.any(Object)
      );
      expect(fsModule.mkdirSync).toHaveBeenCalledWith(
        expect.stringContaining('squads/anvil'),
        expect.any(Object)
      );
    });
  });
});
