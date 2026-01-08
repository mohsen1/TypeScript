/**
 * Tests for TmuxClient
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { TmuxClient } from '../src/tmux/index.js';
import type { PaneId } from '../src/types.js';

// Mock the exec module
vi.mock('../src/utils/exec.js', () => ({
  runCommand: vi.fn(),
  runCommandStrict: vi.fn(),
  commandExists: vi.fn(),
  shellEscape: (s: string) => `'${s.replace(/'/g, "'\\''")}'`,
}));

describe('TmuxClient', () => {
  let tmux: TmuxClient;
  let mockRunCommand: ReturnType<typeof vi.fn>;
  let mockRunCommandStrict: ReturnType<typeof vi.fn>;

  beforeEach(async () => {
    tmux = new TmuxClient();
    const execModule = await import('../src/utils/exec.js');
    mockRunCommand = vi.mocked(execModule.runCommand);
    mockRunCommandStrict = vi.mocked(execModule.runCommandStrict);
    mockRunCommand.mockReset();
    mockRunCommandStrict.mockReset();
  });

  describe('paneId', () => {
    it('creates PaneId from components', () => {
      const pane = TmuxClient.paneId('session', 'window', 0);
      expect(pane).toEqual({
        session: 'session',
        window: 'window',
        pane: 0,
      });
    });
  });

  describe('isInsideTmux', () => {
    const originalTMUX = process.env['TMUX'];

    afterEach(() => {
      if (originalTMUX !== undefined) {
        process.env['TMUX'] = originalTMUX;
      } else {
        delete process.env['TMUX'];
      }
    });

    it('returns true when TMUX env var is set', () => {
      process.env['TMUX'] = '/tmp/tmux-123/default,12345,0';
      expect(tmux.isInsideTmux()).toBe(true);
    });

    it('returns false when TMUX env var is not set', () => {
      delete process.env['TMUX'];
      expect(tmux.isInsideTmux()).toBe(false);
    });
  });

  describe('isAvailable', () => {
    it('returns true when tmux command exists', async () => {
      mockRunCommand.mockResolvedValue({
        stdout: '/usr/bin/tmux',
        stderr: '',
        exitCode: 0,
      });

      const result = await tmux.isAvailable();
      expect(result).toBe(true);
    });

    it('returns false when tmux command does not exist', async () => {
      mockRunCommand.mockResolvedValue({
        stdout: '',
        stderr: '',
        exitCode: 1,
      });

      const result = await tmux.isAvailable();
      expect(result).toBe(false);
    });
  });

  describe('sessionExists', () => {
    it('returns true when session exists', async () => {
      mockRunCommand.mockResolvedValue({
        stdout: '',
        stderr: '',
        exitCode: 0,
      });

      const result = await tmux.sessionExists('my-session');
      expect(result).toBe(true);
      expect(mockRunCommand).toHaveBeenCalledWith(
        expect.stringContaining('has-session')
      );
    });

    it('returns false when session does not exist', async () => {
      mockRunCommand.mockResolvedValue({
        stdout: '',
        stderr: "can't find session",
        exitCode: 1,
      });

      const result = await tmux.sessionExists('nonexistent');
      expect(result).toBe(false);
    });
  });

  describe('createSession', () => {
    it('calls tmux new-session with correct args', async () => {
      mockRunCommandStrict.mockResolvedValue('');

      await tmux.createSession('session', 'window', '/path/to/dir');

      expect(mockRunCommandStrict).toHaveBeenCalledWith(
        expect.stringContaining('new-session -d')
      );
      expect(mockRunCommandStrict).toHaveBeenCalledWith(
        expect.stringContaining("-s 'session'")
      );
      expect(mockRunCommandStrict).toHaveBeenCalledWith(
        expect.stringContaining("-n 'window'")
      );
      expect(mockRunCommandStrict).toHaveBeenCalledWith(
        expect.stringContaining("-c '/path/to/dir'")
      );
    });
  });

  describe('sendKeys', () => {
    it('sends keys without enter', async () => {
      mockRunCommand.mockResolvedValue({ stdout: '', stderr: '', exitCode: 0 });

      const pane: PaneId = { session: 's', window: 'w', pane: 0 };
      await tmux.sendKeys(pane, 'hello world');

      expect(mockRunCommand).toHaveBeenCalledTimes(1);
      expect(mockRunCommand).toHaveBeenCalledWith(
        expect.stringContaining("'hello world'")
      );
    });

    it('sends keys with enter when pressEnter=true', async () => {
      mockRunCommand.mockResolvedValue({ stdout: '', stderr: '', exitCode: 0 });

      const pane: PaneId = { session: 's', window: 'w', pane: 0 };
      await tmux.sendKeys(pane, 'hello', true);

      expect(mockRunCommand).toHaveBeenCalledTimes(2);
      expect(mockRunCommand).toHaveBeenLastCalledWith(
        expect.stringContaining('C-m')
      );
    });
  });

  describe('capturePane', () => {
    it('captures pane content and returns hash', async () => {
      mockRunCommand.mockResolvedValue({
        stdout: 'line1\nline2\nline3',
        stderr: '',
        exitCode: 0,
      });

      const pane: PaneId = { session: 's', window: 'w', pane: 0 };
      const result = await tmux.capturePane(pane);

      expect(result.paneId).toEqual(pane);
      expect(result.content).toBe('line1\nline2\nline3');
      expect(result.hash).toBeDefined();
      expect(result.hash.length).toBe(32); // MD5 hex length
    });

    it('returns empty content on failure', async () => {
      mockRunCommand.mockResolvedValue({
        stdout: '',
        stderr: 'error',
        exitCode: 1,
      });

      const pane: PaneId = { session: 's', window: 'w', pane: 0 };
      const result = await tmux.capturePane(pane);

      expect(result.content).toBe('');
    });
  });

  describe('listWindows', () => {
    it('returns list of window names', async () => {
      mockRunCommand.mockResolvedValue({
        stdout: 'director\nforge\nanvil',
        stderr: '',
        exitCode: 0,
      });

      const windows = await tmux.listWindows('session');
      expect(windows).toEqual(['director', 'forge', 'anvil']);
    });

    it('returns empty list on failure', async () => {
      mockRunCommand.mockResolvedValue({
        stdout: '',
        stderr: 'error',
        exitCode: 1,
      });

      const windows = await tmux.listWindows('session');
      expect(windows).toEqual([]);
    });
  });

  describe('getPaneCount', () => {
    it('returns pane count', async () => {
      mockRunCommand.mockResolvedValue({
        stdout: '0: [80x24]\n1: [80x24]\n2: [80x24]',
        stderr: '',
        exitCode: 0,
      });

      const count = await tmux.getPaneCount('session', 'window');
      expect(count).toBe(3);
    });

    it('returns 0 on failure', async () => {
      mockRunCommand.mockResolvedValue({
        stdout: '',
        stderr: 'error',
        exitCode: 1,
      });

      const count = await tmux.getPaneCount('session', 'window');
      expect(count).toBe(0);
    });
  });
});

// Import afterEach for cleanup
import { afterEach } from 'vitest';
