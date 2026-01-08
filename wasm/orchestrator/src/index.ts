/**
 * Zang Orchestrator - Multi-agent AI development system
 *
 * @module zang-orchestrator
 */

export { Orchestrator, type OrchestratorOptions } from './Orchestrator.js';
export { TmuxClient, type TmuxClientOptions, type SplitPaneOptions } from './tmux/index.js';
export { AgentManager, type AgentStartOptions } from './agent/index.js';
export { IdleMonitor, type MonitoredPane } from './monitor/index.js';
export { StateManager, type SaveOptions } from './state/index.js';
export { WorktreeManager } from './worktree/index.js';
export {
  createConfig,
  configFromEnv,
  validateConfig,
  type ConfigInput,
} from './config/index.js';
export * from './types.js';
