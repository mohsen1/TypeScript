/**
 * IdleMonitor - Tracks pane activity and pokes idle agents
 *
 * Monitors tmux panes for changes in output. When a pane hasn't
 * changed for longer than its threshold, sends a "poke" message
 * to prompt the agent to continue working.
 */

import { TmuxClient } from '../tmux/index.js';
import { AgentManager } from '../agent/index.js';
import type { OrchestratorConfig, AgentRole, PaneId } from '../types.js';

interface PaneTrackingState {
  lastHash: string;
  lastChangeTime: number;
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
   * Check if a pane is idle
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

    return idleSeconds >= threshold;
  }

  /**
   * Update tracking state for a pane
   */
  async updatePaneState(pane: PaneId): Promise<void> {
    const content = await this.tmux.capturePane(pane);
    const key = this.paneKey(pane);
    const state = this.paneStates.get(key);
    const now = Date.now();

    if (!state) {
      // First time seeing this pane
      this.paneStates.set(key, {
        lastHash: content.hash,
        lastChangeTime: now,
      });
    } else if (state.lastHash !== content.hash) {
      // Content changed
      this.paneStates.set(key, {
        lastHash: content.hash,
        lastChangeTime: now,
      });
    }
    // If hash is same, keep existing lastChangeTime (pane is idle)
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
