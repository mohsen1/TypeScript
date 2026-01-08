/**
 * TmuxClient - Handles all tmux session, window, and pane operations
 *
 * This module provides a typed interface to tmux commands, allowing
 * the orchestrator to create sessions, manage panes, send keys, and
 * capture pane output.
 */

import { runCommand, runCommandStrict, shellEscape } from '../utils/index.js';
import type { PaneId, PaneContent } from '../types.js';
import { contentHash } from '../utils/hash.js';

export interface TmuxClientOptions {
  tmuxPath?: string;
}

export interface SplitPaneOptions {
  horizontal?: boolean;
  cwd?: string;
  percent?: number;
}

export interface WindowLayout {
  name: string;
  paneCount: number;
}

export class TmuxClient {
  private readonly tmuxPath: string;

  constructor(options: TmuxClientOptions = {}) {
    this.tmuxPath = options.tmuxPath ?? 'tmux';
  }

  /**
   * Check if tmux is available
   */
  async isAvailable(): Promise<boolean> {
    const result = await runCommand(`command -v ${this.tmuxPath}`);
    return result.exitCode === 0;
  }

  /**
   * Check if a session exists
   */
  async sessionExists(session: string): Promise<boolean> {
    const result = await runCommand(
      `${this.tmuxPath} has-session -t ${shellEscape(session)}`
    );
    return result.exitCode === 0;
  }

  /**
   * Create a new session with a window
   */
  async createSession(
    session: string,
    windowName: string,
    cwd: string
  ): Promise<void> {
    await runCommandStrict(
      `${this.tmuxPath} new-session -d -s ${shellEscape(session)} -n ${shellEscape(windowName)} -c ${shellEscape(cwd)}`
    );
  }

  /**
   * Kill a session
   */
  async killSession(session: string): Promise<void> {
    await runCommand(`${this.tmuxPath} kill-session -t ${shellEscape(session)}`);
  }

  /**
   * Create a new window in a session
   */
  async createWindow(
    session: string,
    windowName: string,
    cwd: string
  ): Promise<void> {
    await runCommandStrict(
      `${this.tmuxPath} new-window -t ${shellEscape(session)} -n ${shellEscape(windowName)} -c ${shellEscape(cwd)}`
    );
  }

  /**
   * Select a window
   */
  async selectWindow(session: string, window: string): Promise<void> {
    await runCommand(
      `${this.tmuxPath} select-window -t ${shellEscape(session)}:${shellEscape(window)}`
    );
  }

  /**
   * Split a pane
   */
  async splitPane(pane: PaneId, options: SplitPaneOptions = {}): Promise<void> {
    const target = this.formatPaneTarget(pane);
    const splitFlag = options.horizontal ? '-h' : '-v';
    const cwdArg = options.cwd ? `-c ${shellEscape(options.cwd)}` : '';
    const percentArg = options.percent ? `-p ${options.percent}` : '';

    await runCommandStrict(
      `${this.tmuxPath} split-window ${splitFlag} -t ${target} ${cwdArg} ${percentArg}`.trim()
    );
  }

  /**
   * Apply a layout to a window
   */
  async selectLayout(
    session: string,
    window: string,
    layout: 'tiled' | 'even-horizontal' | 'even-vertical' | 'main-horizontal' | 'main-vertical'
  ): Promise<void> {
    await runCommand(
      `${this.tmuxPath} select-layout -t ${shellEscape(session)}:${shellEscape(window)} ${layout}`
    );
  }

  /**
   * Set pane title
   */
  async setPaneTitle(pane: PaneId, title: string): Promise<void> {
    const target = this.formatPaneTarget(pane);
    await runCommand(
      `${this.tmuxPath} select-pane -t ${target} -T ${shellEscape(title)}`
    );
  }

  /**
   * Send keys to a pane
   */
  async sendKeys(pane: PaneId, keys: string, pressEnter = false): Promise<void> {
    const target = this.formatPaneTarget(pane);
    await runCommand(
      `${this.tmuxPath} send-keys -t ${target} ${shellEscape(keys)}`
    );
    if (pressEnter) {
      await runCommand(`${this.tmuxPath} send-keys -t ${target} C-m`);
    }
  }

  /**
   * Send Escape key to a pane (cancel current operation)
   */
  async sendEscape(pane: PaneId): Promise<void> {
    const target = this.formatPaneTarget(pane);
    await runCommand(`${this.tmuxPath} send-keys -t ${target} Escape`);
  }

  /**
   * Capture pane content
   */
  async capturePane(pane: PaneId, lines = 200): Promise<PaneContent> {
    const target = this.formatPaneTarget(pane);
    const result = await runCommand(
      `${this.tmuxPath} capture-pane -p -t ${target} -S -${lines}`
    );

    const content = result.exitCode === 0 ? result.stdout : '';
    return {
      paneId: pane,
      content,
      hash: contentHash(content),
    };
  }

  /**
   * Get current directory of a pane
   */
  async getPanePath(pane: PaneId): Promise<string> {
    const target = this.formatPaneTarget(pane);
    const result = await runCommand(
      `${this.tmuxPath} display-message -p -t ${target} '#{pane_current_path}'`
    );
    return result.exitCode === 0 ? result.stdout : '';
  }

  /**
   * List windows in a session
   */
  async listWindows(session: string): Promise<string[]> {
    const result = await runCommand(
      `${this.tmuxPath} list-windows -t ${shellEscape(session)} -F '#{window_name}'`
    );
    if (result.exitCode !== 0) {
      return [];
    }
    return result.stdout.split('\n').filter(Boolean);
  }

  /**
   * Get pane count in a window
   */
  async getPaneCount(session: string, window: string): Promise<number> {
    const result = await runCommand(
      `${this.tmuxPath} list-panes -t ${shellEscape(session)}:${shellEscape(window)}`
    );
    if (result.exitCode !== 0) {
      return 0;
    }
    return result.stdout.split('\n').filter(Boolean).length;
  }

  /**
   * Run a shell command in the background (detached from session)
   */
  async runInBackground(script: string): Promise<void> {
    await runCommand(`${this.tmuxPath} run-shell -b ${shellEscape(script)}`);
  }

  /**
   * Attach to a session (used for interactive mode)
   */
  async attach(session: string): Promise<void> {
    // Note: This will block the process
    await runCommand(`${this.tmuxPath} attach -t ${shellEscape(session)}`);
  }

  /**
   * Check if we're already inside tmux
   */
  isInsideTmux(): boolean {
    return process.env['TMUX'] !== undefined;
  }

  /**
   * Format a pane target string
   */
  private formatPaneTarget(pane: PaneId): string {
    return `${shellEscape(pane.session)}:${shellEscape(pane.window)}.${pane.pane}`;
  }

  /**
   * Create a PaneId from components
   */
  static paneId(session: string, window: string, pane: number): PaneId {
    return { session, window, pane };
  }
}

export default TmuxClient;
