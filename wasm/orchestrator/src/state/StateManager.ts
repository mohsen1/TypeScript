/**
 * StateManager - Handles saving and restoring session state
 *
 * Captures pane content and paths to enable resuming sessions
 * after a kill. State is saved to a JSON file.
 */

import { readFile, writeFile, unlink } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import { TmuxClient } from '../tmux/index.js';
import type {
  OrchestratorConfig,
  SessionState,
  PaneState,
  PaneId,
} from '../types.js';

export interface SaveOptions {
  captureLines?: number;
  contextLines?: number;
}

const DEFAULT_SAVE_OPTIONS: SaveOptions = {
  captureLines: 500,
  contextLines: 200,
};

export class StateManager {
  private readonly tmux: TmuxClient;
  private readonly config: OrchestratorConfig;

  constructor(tmux: TmuxClient, config: OrchestratorConfig) {
    this.tmux = tmux;
    this.config = config;
  }

  /**
   * Get the state file path
   */
  get stateFile(): string {
    return this.config.stateFile;
  }

  /**
   * Check if a saved state exists
   */
  stateExists(): boolean {
    return existsSync(this.stateFile);
  }

  /**
   * Save the current session state
   */
  async saveState(options: SaveOptions = {}): Promise<void> {
    const opts = { ...DEFAULT_SAVE_OPTIONS, ...options };
    const windows = ['director', 'forge', 'anvil'];
    const panes: Record<string, PaneState> = {};

    for (const window of windows) {
      // Check if window exists
      const windowList = await this.tmux.listWindows(this.config.session);
      if (!windowList.includes(window)) {
        continue;
      }

      const paneCount = await this.tmux.getPaneCount(
        this.config.session,
        window
      );

      for (let pane = 0; pane < paneCount; pane++) {
        const paneId: PaneId = {
          session: this.config.session,
          window,
          pane,
        };

        const key = `${window}_${pane}`;
        const [content, path] = await Promise.all([
          this.tmux.capturePane(paneId, opts.captureLines),
          this.tmux.getPanePath(paneId),
        ]);

        // Take only the last N lines for context
        const lines = content.content.split('\n');
        const contextContent = lines
          .slice(-(opts.contextLines ?? 200))
          .join('\n');

        panes[key] = {
          path,
          context: contextContent,
        };
      }
    }

    const state: SessionState = {
      timestamp: new Date().toISOString(),
      panes,
    };

    await writeFile(this.stateFile, JSON.stringify(state, null, 2));
  }

  /**
   * Load saved state
   */
  async loadState(): Promise<SessionState | null> {
    if (!this.stateExists()) {
      return null;
    }

    try {
      const content = await readFile(this.stateFile, 'utf-8');
      return JSON.parse(content) as SessionState;
    } catch {
      return null;
    }
  }

  /**
   * Get resume prompt for a pane from saved state
   */
  async getResumePrompt(paneKey: string): Promise<string | null> {
    const state = await this.loadState();
    if (!state) {
      return null;
    }

    const paneState = state.panes[paneKey];
    if (!paneState?.context) {
      return null;
    }

    // Build resume prompt with context
    return `RESUMING SESSION - Here's what was happening before the session ended:
---
${paneState.context}
---
Continue where you left off. Check your current git status and plan file, then resume your work.`;
  }

  /**
   * Delete saved state
   */
  async deleteState(): Promise<void> {
    if (this.stateExists()) {
      await unlink(this.stateFile);
    }
  }

  /**
   * Get age of saved state in seconds
   */
  async getStateAge(): Promise<number | null> {
    const state = await this.loadState();
    if (!state) {
      return null;
    }

    const savedTime = new Date(state.timestamp).getTime();
    const now = Date.now();
    return (now - savedTime) / 1000;
  }

  /**
   * Format the saved state for display
   */
  async formatStateInfo(): Promise<string> {
    const state = await this.loadState();
    if (!state) {
      return 'No saved state found';
    }

    const age = await this.getStateAge();
    const ageStr = age !== null ? this.formatDuration(age) : 'unknown';
    const paneCount = Object.keys(state.panes).length;

    return `State file: ${this.stateFile}
Saved: ${state.timestamp}
Age: ${ageStr}
Panes captured: ${paneCount}`;
  }

  /**
   * Format duration in human-readable format
   */
  private formatDuration(seconds: number): string {
    if (seconds < 60) {
      return `${Math.round(seconds)}s`;
    }
    if (seconds < 3600) {
      return `${Math.round(seconds / 60)}m`;
    }
    if (seconds < 86400) {
      return `${Math.round(seconds / 3600)}h`;
    }
    return `${Math.round(seconds / 86400)}d`;
  }
}

export default StateManager;
