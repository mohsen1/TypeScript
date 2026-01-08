/**
 * Config - Typed configuration management for the orchestrator
 *
 * Provides default values and environment variable overrides,
 * matching the bash script's configuration options.
 */

import { homedir } from 'node:os';
import { join, dirname, resolve } from 'node:path';
import { existsSync } from 'node:fs';
import type {
  OrchestratorConfig,
  AgentType,
  IdleThresholds,
  PokeMessages,
  StartupPrompts,
  TimingConfig,
  SquadConfig,
  MutablePartial,
} from '../types.js';

/**
 * Default squad configuration - two squads with 5 workers each
 */
const DEFAULT_SQUADS: readonly SquadConfig[] = [
  {
    name: 'forge',
    workerCount: 5,
    focusAreas: ['solver/', 'checker/', 'binder/', 'types/'],
  },
  {
    name: 'anvil',
    workerCount: 5,
    focusAreas: ['thin_emitter/', 'transforms/', 'cli/', 'lsp/'],
  },
];

export interface ConfigInput {
  agentType?: AgentType;
  rootDir?: string;
  worktreeBase?: string;
  stateFile?: string;
  session?: string;
  squads?: SquadConfig[];
  overrides?: MutablePartial<OrchestratorConfig>;
}

const DEFAULT_SESSION = 'zang-org';

const DEFAULT_IDLE_THRESHOLDS: IdleThresholds = {
  director: 600, // 10 minutes - Director is hands-off
  em: 60,        // 1 minute - EMs should be responsive
  worker: 300,   // 5 minutes - Workers need time to work
};

const DEFAULT_POKE_MESSAGES: PokeMessages = {
  director:
    'Run `git fetch origin` and check for new commits on squad branches. Merge any squad branches that have new commits into rust. No need to coordinate with EMs or workers.',
  em: 'Check worker status: 1) Run `git fetch origin` and merge any new commits from worker branches (origin/worker/$SQUAD_NAME-*) into your squad branch. 2) Check tmux panes for idle workers waiting for tasks. 3) If any worker is idle, assign them a task from the backlog in wasm/specs/squads/$SQUAD_NAME/. Update their worker plan file.',
  worker:
    'How is your task going? If stuck, describe the issue.',
};

const DEFAULT_STARTUP_PROMPTS: StartupPrompts = {
  director:
    "Read .role/AGENTS.md for your instructions. Merge EM branches (em/forge, em/anvil) into rust when they have blocker fixes. Be hands-off otherwise.",
  em: "You are EM for squad $SQUAD_NAME. Read .role/AGENTS.md for your instructions. You have your own worktree on branch em/$SQUAD_NAME. FIRST: run ./wasm/test.sh 2>&1 | head -50 to check build. If build fails, FIX IT YOURSELF before assigning worker tasks.",
  worker:
    "You are worker $WORKER_NUM in squad $SQUAD_NAME. Read .role/AGENTS.md then your plan at wasm/specs/squads/$SQUAD_NAME/worker-${WORKER_NUM}_plan.md. Switch to branch worker/$SQUAD_NAME-$WORKER_NUM and work on your current assignment.",
};

const DEFAULT_TIMING: TimingConfig = {
  startPause: 10,          // Seconds to wait for agents to boot
  sendEnterPause: 1,       // Seconds between sending text and Enter
  staggerPause: 2,         // Seconds between starting each worker
  agentRestartDelay: 2,    // Seconds before restarting a crashed agent
  mergePauseSeconds: 240,  // 4 minutes for coordinated merge
};

/**
 * Get agent-specific configuration based on type
 */
function getAgentConfig(agentType: AgentType): {
  cmd: string;
  args: string;
  updateCmd: string;
} {
  switch (agentType) {
    case 'codex':
      return {
        cmd: 'codex',
        args: '--dangerously-bypass-approvals-and-sandbox',
        updateCmd: 'npm install -g @openai/codex',
      };
    case 'claude':
    default:
      return {
        cmd: 'claude',
        args: '--dangerously-skip-permissions',
        updateCmd: 'npm install -g @anthropic-ai/claude-code',
      };
  }
}

/**
 * Resolve the root directory from various possible locations
 */
function resolveRootDir(providedPath?: string): string {
  if (providedPath && existsSync(providedPath)) {
    return resolve(providedPath);
  }

  // Try to find TypeScript directory
  const cwd = process.cwd();

  // Check if we're in TypeScript directory
  if (existsSync(join(cwd, 'wasm', 'src'))) {
    return cwd;
  }

  // Check if we're in a subdirectory
  const parent = dirname(cwd);
  if (existsSync(join(parent, 'TypeScript', 'wasm', 'src'))) {
    return join(parent, 'TypeScript');
  }

  // Check common locations
  const commonPaths = [
    join(homedir(), 'code', 'TypeScript'),
    join(homedir(), 'projects', 'TypeScript'),
    '/code/TypeScript',
  ];

  for (const path of commonPaths) {
    if (existsSync(join(path, 'wasm', 'src'))) {
      return path;
    }
  }

  throw new Error(
    'Could not find TypeScript root directory. Please provide rootDir in config.'
  );
}

/**
 * Resolve worktree base directory
 */
function resolveWorktreeBase(rootDir: string, providedPath?: string): string {
  if (providedPath) {
    return resolve(providedPath);
  }
  // Worktrees are siblings of the main repo
  return dirname(rootDir);
}

/**
 * Create a complete configuration from input
 */
export function createConfig(input: ConfigInput = {}): OrchestratorConfig {
  const agentType = input.agentType ?? 'claude';
  const agentConfig = getAgentConfig(agentType);
  const rootDir = resolveRootDir(input.rootDir);
  const worktreeBase = resolveWorktreeBase(rootDir, input.worktreeBase);
  const stateFile = input.stateFile ?? join(homedir(), '.zang-org-state.json');
  const session = input.session ?? DEFAULT_SESSION;

  const squads = input.squads ?? [...DEFAULT_SQUADS];

  const baseConfig: OrchestratorConfig = {
    session,
    agentType,
    agentArgs: agentConfig.args,
    agentUpdateCmd: agentConfig.updateCmd,
    autoFetch: true,
    autoRestartAgent: true,
    autoMonitor: true,
    autoAttach: true,
    agentAutoUpdate: true,
    idleThresholds: { ...DEFAULT_IDLE_THRESHOLDS },
    pokeMessages: { ...DEFAULT_POKE_MESSAGES },
    startupPrompts: { ...DEFAULT_STARTUP_PROMPTS },
    timing: { ...DEFAULT_TIMING },
    rootDir,
    worktreeBase,
    stateFile,
    squads,
  };

  // Apply overrides
  if (input.overrides) {
    return mergeConfig(baseConfig, input.overrides);
  }

  return baseConfig;
}

/**
 * Merge partial config into base config
 */
function mergeConfig(
  base: OrchestratorConfig,
  overrides: MutablePartial<OrchestratorConfig>
): OrchestratorConfig {
  return {
    ...base,
    ...overrides,
    idleThresholds: {
      ...base.idleThresholds,
      ...overrides.idleThresholds,
    },
    pokeMessages: {
      ...base.pokeMessages,
      ...overrides.pokeMessages,
    },
    startupPrompts: {
      ...base.startupPrompts,
      ...overrides.startupPrompts,
    },
    timing: {
      ...base.timing,
      ...overrides.timing,
    },
    // Squads are replaced entirely if provided in overrides (filter out undefined)
    squads: (overrides.squads?.filter((s): s is SquadConfig => s !== undefined) ?? base.squads) as readonly SquadConfig[],
  };
}

/**
 * Read configuration from environment variables
 */
export function configFromEnv(): MutablePartial<OrchestratorConfig> {
  const env = process.env;

  const partial: MutablePartial<OrchestratorConfig> = {};

  if (env['SESSION']) partial.session = env['SESSION'];
  if (env['AUTO_FETCH'] === '0') partial.autoFetch = false;
  if (env['AUTO_RESTART_AGENT'] === '0') partial.autoRestartAgent = false;
  if (env['AUTO_MONITOR'] === '0') partial.autoMonitor = false;
  if (env['AUTO_ATTACH'] === '0') partial.autoAttach = false;
  if (env['AGENT_AUTO_UPDATE'] === '0') partial.agentAutoUpdate = false;

  const idleThresholds: MutablePartial<IdleThresholds> = {};
  if (env['DIRECTOR_IDLE_SECONDS']) {
    idleThresholds.director = parseInt(env['DIRECTOR_IDLE_SECONDS'], 10);
  }
  if (env['EM_IDLE_SECONDS']) {
    idleThresholds.em = parseInt(env['EM_IDLE_SECONDS'], 10);
  }
  if (env['WORKER_IDLE_SECONDS']) {
    idleThresholds.worker = parseInt(env['WORKER_IDLE_SECONDS'], 10);
  }
  if (Object.keys(idleThresholds).length > 0) {
    partial.idleThresholds = idleThresholds as IdleThresholds;
  }

  const pokeMessages: MutablePartial<PokeMessages> = {};
  if (env['DIRECTOR_POKE']) pokeMessages.director = env['DIRECTOR_POKE'];
  if (env['EM_POKE']) pokeMessages.em = env['EM_POKE'];
  if (env['WORKER_POKE']) pokeMessages.worker = env['WORKER_POKE'];
  if (Object.keys(pokeMessages).length > 0) {
    partial.pokeMessages = pokeMessages as PokeMessages;
  }

  const timing: MutablePartial<TimingConfig> = {};
  if (env['START_PAUSE']) timing.startPause = parseInt(env['START_PAUSE'], 10);
  if (env['SEND_ENTER_PAUSE']) {
    timing.sendEnterPause = parseInt(env['SEND_ENTER_PAUSE'], 10);
  }
  if (env['STAGGER_PAUSE']) {
    timing.staggerPause = parseInt(env['STAGGER_PAUSE'], 10);
  }
  if (env['AGENT_RESTART_DELAY']) {
    timing.agentRestartDelay = parseInt(env['AGENT_RESTART_DELAY'], 10);
  }
  if (env['MERGE_PAUSE_SECONDS']) {
    timing.mergePauseSeconds = parseInt(env['MERGE_PAUSE_SECONDS'], 10);
  }
  if (Object.keys(timing).length > 0) {
    partial.timing = timing as TimingConfig;
  }

  return partial;
}

/**
 * Validate a configuration
 */
export function validateConfig(config: OrchestratorConfig): string[] {
  const errors: string[] = [];

  if (!existsSync(config.rootDir)) {
    errors.push(`Root directory does not exist: ${config.rootDir}`);
  }

  if (!existsSync(config.worktreeBase)) {
    errors.push(`Worktree base does not exist: ${config.worktreeBase}`);
  }

  if (config.idleThresholds.director < 0) {
    errors.push('Director idle threshold must be non-negative');
  }

  if (config.idleThresholds.em < 0) {
    errors.push('EM idle threshold must be non-negative');
  }

  if (config.idleThresholds.worker < 0) {
    errors.push('Worker idle threshold must be non-negative');
  }

  return errors;
}
