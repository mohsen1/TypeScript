/**
 * Core types for the Zang Orchestrator
 */

export type AgentType = 'claude' | 'codex';

export type AgentRole = 'director' | 'em' | 'worker';

// Squad name is now a string to support dynamic squad configuration
export type SquadName = string;

/**
 * Squad configuration - defines a team of workers
 */
export interface SquadConfig {
  readonly name: string;
  readonly workerCount: number;
  readonly focusAreas: string[];
}

export interface PaneId {
  readonly session: string;
  readonly window: string;
  readonly pane: number;
}

export interface WorkerIdentity {
  readonly squad: SquadName;
  readonly number: number;
}

export interface EMIdentity {
  readonly squad: SquadName;
}

export interface AgentIdentity {
  readonly role: AgentRole;
  readonly worker?: WorkerIdentity;
  readonly em?: EMIdentity;
}

export interface IdleThresholds {
  readonly director: number;
  readonly em: number;
  readonly worker: number;
}

export interface PokeMessages {
  readonly director: string;
  readonly em: string;
  readonly worker: string;
}

export interface StartupPrompts {
  readonly director: string;
  readonly em: string;
  readonly worker: string;
}

export interface TimingConfig {
  readonly startPause: number;
  readonly sendEnterPause: number;
  readonly staggerPause: number;
  readonly agentRestartDelay: number;
  readonly mergePauseSeconds: number;
}

export interface OrchestratorConfig {
  readonly session: string;
  readonly agentType: AgentType;
  readonly agentArgs: string;
  readonly agentUpdateCmd: string;
  readonly autoFetch: boolean;
  readonly autoRestartAgent: boolean;
  readonly autoMonitor: boolean;
  readonly autoAttach: boolean;
  readonly agentAutoUpdate: boolean;
  readonly idleThresholds: IdleThresholds;
  readonly pokeMessages: PokeMessages;
  readonly startupPrompts: StartupPrompts;
  readonly timing: TimingConfig;
  readonly rootDir: string;
  readonly worktreeBase: string;
  readonly stateFile: string;
  readonly squads: readonly SquadConfig[];
}

export interface PaneState {
  readonly path: string;
  readonly context: string;
}

export interface SessionState {
  readonly timestamp: string;
  readonly panes: Record<string, PaneState>;
}

export interface PaneContent {
  readonly paneId: PaneId;
  readonly content: string;
  readonly hash: string;
}

export interface CommandResult {
  readonly stdout: string;
  readonly stderr: string;
  readonly exitCode: number;
}

export interface WorktreeInfo {
  readonly path: string;
  readonly branch: string;
  readonly exists: boolean;
}

export type OperationMode = 'start' | 'kill' | 'resume' | 'fresh' | 'graceful-exit';

/**
 * Utility type to make all properties mutable (remove readonly)
 */
export type Mutable<T> = { -readonly [P in keyof T]: T[P] };

/**
 * Deep partial type for nested objects
 */
export type DeepPartial<T> = {
  [P in keyof T]?: T[P] extends object ? DeepPartial<T[P]> : T[P];
};

/**
 * Mutable deep partial for config building
 */
export type MutablePartial<T> = {
  -readonly [P in keyof T]?: T[P] extends object ? MutablePartial<T[P]> : T[P];
};
