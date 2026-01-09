/**
 * IdleMonitor - Tracks pane activity and pokes idle agents
 *
 * Monitors tmux panes for changes in output. When a pane hasn't
 * changed for longer than its threshold, sends a "poke" message
 * to prompt the agent to continue working.
 *
 * SMART DETECTION: Agents deep in debugging/fixing bugs can work for
 * 30-60+ minutes without needing interruption. We detect "active work"
 * patterns to avoid disrupting focused agents:
 * - Running tools (Bash, Read, Search, Edit, etc.)
 * - Showing progress spinners or timers
 * - Displaying todo lists with in-progress items
 * - Building/compiling (wasm-pack, cargo, npm, etc.)
 */

import { TmuxClient } from '../tmux/index.js';
import { AgentManager } from '../agent/index.js';
import type { OrchestratorConfig, AgentRole, PaneId } from '../types.js';

/**
 * Patterns that indicate the agent is actively working and should NOT be poked
 */
const ACTIVE_WORK_PATTERNS = [
  // Claude Code tool execution indicators
  /⏺\s*(Bash|Read|Search|Edit|Write|Update|Grep|Glob|Task)\(/i,
  /Running\.\.\./i,
  /\(esc to interrupt/i,           // Active tool with escape hint
  /ctrl\+[a-z] to/i,               // Interactive hints

  // Progress indicators
  /\d+m\s+\d+s/,                   // Timer like "44m 9s"
  /\d+:\d{2}:\d{2}/,               // Duration like "1:23:45"
  /↑\s*[\d.]+k?\s*tokens/i,        // Token counter

  // Build/compile indicators
  /Compiling\s+\w+/i,
  /Building\s+/i,
  /wasm-pack\s+build/i,
  /cargo\s+(build|test|check)/i,
  /npm\s+(run|install|test)/i,
  /\[INFO\]:/,                     // wasm-pack info output

  // Todo list with active work
  /\[in_progress\]/i,
  /☐.*\(in progress\)/i,

  // Waiting for user input (don't poke - waiting is intentional)
  />\s*$/,                         // Prompt waiting for input
  /waiting for.*input/i,
];

/**
 * Patterns that indicate the agent is truly idle and might need a poke
 */
const IDLE_PATTERNS = [
  /How can I help/i,
  /What would you like/i,
  /Let me know if/i,
  /Is there anything else/i,
  /I've completed/i,
  /Task complete/i,
];

interface PaneTrackingState {
  lastHash: string;
  lastChangeTime: number;
  /** Last time we detected active work patterns */
  lastActiveWorkTime: number;
  /** Number of consecutive idle checks (resets on activity) */
  consecutiveIdleChecks: number;
}

export interface MonitoredPane {
  pane: PaneId;
  role: AgentRole;
}

export class IdleMonitor {
  private readonly tmux: TmuxClient;
  private readonly agent: AgentManager;
  private readonly config: OrchestratorConfig;
  private readonly paneStates: Map<string, PaneTrackingState> = new Map();
  private intervalId: ReturnType<typeof setInterval> | null = null;
  private monitoredPanes: MonitoredPane[] = [];

  constructor(
    tmux: TmuxClient,
    agent: AgentManager,
    config: OrchestratorConfig
  ) {
    this.tmux = tmux;
    this.agent = agent;
    this.config = config;
  }

  /**
   * Register panes to monitor
   */
  registerPanes(panes: MonitoredPane[]): void {
    this.monitoredPanes = panes;
  }

  /**
   * Add a single pane to monitor
   */
  addPane(pane: PaneId, role: AgentRole): void {
    this.monitoredPanes.push({ pane, role });
  }

  /**
   * Get idle threshold for a role
   */
  getThreshold(role: AgentRole): number {
    return this.config.idleThresholds[role];
  }

  /**
   * Check if pane content shows signs of active work
   */
  private isShowingActiveWork(content: string): boolean {
    // Check last ~50 lines for active work patterns
    const recentContent = content.split('\n').slice(-50).join('\n');
    return ACTIVE_WORK_PATTERNS.some((pattern) => pattern.test(recentContent));
  }

  /**
   * Check if pane content shows signs of being truly idle
   */
  private isShowingIdleState(content: string): boolean {
    // Check last ~20 lines for idle patterns
    const recentContent = content.split('\n').slice(-20).join('\n');
    return IDLE_PATTERNS.some((pattern) => pattern.test(recentContent));
  }

  /**
   * Check if a pane is idle and should be poked
   *
   * Smart detection: An agent is only considered "pokeable" if:
   * 1. The output hash hasn't changed for threshold seconds, AND
   * 2. The content doesn't show active work patterns, AND
   * 3. Either shows idle patterns OR has been idle for extended time
   *
   * Workers deep in debugging can work for 60+ minutes - don't interrupt them
   * if they're showing active work patterns (tool calls, builds, timers, etc.)
   */
  async checkPaneIdle(pane: PaneId, role: AgentRole): Promise<boolean> {
    const key = this.paneKey(pane);
    const state = this.paneStates.get(key);
    const threshold = this.getThreshold(role);

    if (!state || threshold <= 0) {
      return false;
    }

    const now = Date.now();
    const idleSeconds = (now - state.lastChangeTime) / 1000;

    // If output changed recently, definitely not idle
    if (idleSeconds < threshold) {
      return false;
    }

    // Get current content to check for activity patterns
    const content = await this.tmux.capturePane(pane);

    // If showing active work patterns, NOT idle (even if output is static)
    if (this.isShowingActiveWork(content.content)) {
      // Update lastActiveWorkTime since we see active patterns
      this.paneStates.set(key, {
        ...state,
        lastActiveWorkTime: now,
        consecutiveIdleChecks: 0,
      });
      return false;
    }

    // Check how long since we last saw active work
    const timeSinceActiveWork = (now - state.lastActiveWorkTime) / 1000;

    // If we saw active work recently (within 10 minutes), be patient
    // unless showing explicit idle patterns
    if (timeSinceActiveWork < 600 && !this.isShowingIdleState(content.content)) {
      // Increment consecutive idle checks
      this.paneStates.set(key, {
        ...state,
        consecutiveIdleChecks: state.consecutiveIdleChecks + 1,
      });

      // Only poke after multiple consecutive idle checks (be really sure)
      // Require 3 consecutive checks (at 5s interval = 15+ seconds of apparent idle)
      if (state.consecutiveIdleChecks < 3) {
        return false;
      }
    }

    // Truly idle - either showing idle patterns or no activity for extended time
    return true;
  }

  /**
   * Update tracking state for a pane
   */
  async updatePaneState(pane: PaneId): Promise<void> {
    const content = await this.tmux.capturePane(pane);
    const key = this.paneKey(pane);
    const state = this.paneStates.get(key);
    const now = Date.now();

    // Check if content shows active work
    const showingActiveWork = this.isShowingActiveWork(content.content);

    if (!state) {
      // First time seeing this pane
      this.paneStates.set(key, {
        lastHash: content.hash,
        lastChangeTime: now,
        lastActiveWorkTime: showingActiveWork ? now : now,
        consecutiveIdleChecks: 0,
      });
    } else if (state.lastHash !== content.hash) {
      // Content changed - reset idle tracking
      this.paneStates.set(key, {
        lastHash: content.hash,
        lastChangeTime: now,
        lastActiveWorkTime: showingActiveWork ? now : state.lastActiveWorkTime,
        consecutiveIdleChecks: 0,
      });
    } else if (showingActiveWork) {
      // Hash same but showing active work - update active work time
      this.paneStates.set(key, {
        ...state,
        lastActiveWorkTime: now,
        consecutiveIdleChecks: 0,
      });
    }
    // If hash is same and not showing active work, keep existing state
  }

  /**
   * Run a single monitoring cycle
   */
  async cycle(): Promise<void> {
    // Check if session still exists
    const sessionExists = await this.tmux.sessionExists(this.config.session);
    if (!sessionExists) {
      this.stop();
      return;
    }

    for (const { pane, role } of this.monitoredPanes) {
      // Update state first
      await this.updatePaneState(pane);

      // Check if idle
      const isIdle = await this.checkPaneIdle(pane, role);

      if (isIdle) {
        // Poke the agent
        await this.agent.pokeAgent(pane, role);

        // Reset the change time so we don't poke again immediately
        const key = this.paneKey(pane);
        const state = this.paneStates.get(key);
        if (state) {
          this.paneStates.set(key, {
            ...state,
            lastChangeTime: Date.now(),
          });
        }
      }
    }
  }

  /**
   * Start continuous monitoring
   */
  start(intervalMs = 5000): void {
    if (this.intervalId) {
      return; // Already running
    }

    this.intervalId = setInterval(() => {
      this.cycle().catch((err) => {
        console.error('Monitor cycle error:', err);
      });
    }, intervalMs);
  }

  /**
   * Stop monitoring
   */
  stop(): void {
    if (this.intervalId) {
      clearInterval(this.intervalId);
      this.intervalId = null;
    }
  }

  /**
   * Check if monitor is running
   */
  isRunning(): boolean {
    return this.intervalId !== null;
  }

  /**
   * Get the state of a pane (for testing/debugging)
   */
  getPaneState(pane: PaneId): PaneTrackingState | undefined {
    return this.paneStates.get(this.paneKey(pane));
  }

  /**
   * Reset all tracking state
   */
  reset(): void {
    this.paneStates.clear();
    this.monitoredPanes = [];
  }

  /**
   * Generate a unique key for a pane
   */
  private paneKey(pane: PaneId): string {
    return `${pane.session}_${pane.window}_${pane.pane}`;
  }
}

export default IdleMonitor;
