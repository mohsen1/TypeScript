/**
 * AgentManager - Handles spawning and managing AI agents (Claude/Codex)
 *
 * Responsible for starting agents in tmux panes with appropriate
 * environment variables and startup prompts.
 */

import { TmuxClient } from '../tmux/index.js';
import { commandExists, shellEscape } from '../utils/index.js';
import type {
  AgentType,
  AgentRole,
  OrchestratorConfig,
  PaneId,
  SquadName,
} from '../types.js';

export interface AgentStartOptions {
  pane: PaneId;
  role: AgentRole;
  squad?: SquadName | undefined;
  workerNum?: number | undefined;
  cwd: string;
  resumePrompt?: string | undefined;
}

export class AgentManager {
  private readonly tmux: TmuxClient;
  private readonly config: OrchestratorConfig;

  constructor(tmux: TmuxClient, config: OrchestratorConfig) {
    this.tmux = tmux;
    this.config = config;
  }

  /**
   * Get the agent command (claude or codex)
   */
  getAgentCommand(): string {
    return this.config.agentType === 'codex' ? 'codex' : 'claude';
  }

  /**
   * Get the full command to run the agent (with restart loop if enabled)
   */
  getAgentRunCommand(): string {
    const baseCmd = `${this.getAgentCommand()} ${this.config.agentArgs}`;

    if (this.config.autoRestartAgent) {
      return `while true; do ${baseCmd}; sleep ${this.config.timing.agentRestartDelay}; done`;
    }

    return baseCmd;
  }

  /**
   * Check if the agent CLI is available
   */
  async isAgentAvailable(): Promise<boolean> {
    return commandExists(this.getAgentCommand());
  }

  /**
   * Update the agent CLI
   */
  async updateAgent(): Promise<boolean> {
    if (!this.config.agentAutoUpdate) {
      return true;
    }

    const { runCommand } = await import('../utils/index.js');
    const result = await runCommand(`bash -lc "${this.config.agentUpdateCmd}"`);
    return result.exitCode === 0;
  }

  /**
   * Start an agent in a pane
   */
  async startAgent(options: AgentStartOptions): Promise<void> {
    const { pane, role, squad, workerNum, cwd, resumePrompt } = options;

    // Build environment variables for the agent
    const envVars = this.buildEnvVars(role, squad, workerNum, cwd);

    // Build the command with environment and agent
    const envExports = Object.entries(envVars)
      .map(([k, v]) => `export ${k}=${shellEscape(v)}`);

    const cdCommand = `cd ${shellEscape(cwd)}`;
    const agentCmd = this.getAgentRunCommand();

    // Combine: cd, set env (if any), run agent in bash -lc (login shell for PATH)
    const commandParts = [cdCommand, ...envExports, `bash -lc '${agentCmd}'`];
    const fullCommand = commandParts.join(' && ');

    // Send the command to start the agent
    await this.tmux.sendKeys(pane, fullCommand, true);

    // Wait for agent to boot
    await this.sleep(this.config.timing.startPause * 1000);

    // Send startup prompt
    const prompt = resumePrompt ?? this.getStartupPrompt(role, squad, workerNum);
    await this.sendPrompt(pane, prompt);
  }

  /**
   * Send a prompt to an agent
   */
  async sendPrompt(pane: PaneId, prompt: string): Promise<void> {
    await this.tmux.sendKeys(pane, prompt);
    await this.sleep(this.config.timing.sendEnterPause * 1000);
    await this.tmux.sendKeys(pane, '', true); // Press Enter
  }

  /**
   * Cancel the current operation in a pane
   */
  async cancelOperation(pane: PaneId): Promise<void> {
    await this.tmux.sendEscape(pane);
  }

  /**
   * Poke an idle agent
   */
  async pokeAgent(pane: PaneId, role: AgentRole): Promise<void> {
    const message = this.getPokeMessage(role);
    await this.sendPrompt(pane, message);
  }

  /**
   * Get the startup prompt for a role
   */
  getStartupPrompt(
    role: AgentRole,
    squad?: SquadName,
    workerNum?: number
  ): string {
    let prompt = this.config.startupPrompts[role];

    // Substitute variables
    if (squad) {
      prompt = prompt.replace(/\$SQUAD_NAME/g, squad);
    }
    if (workerNum !== undefined) {
      prompt = prompt.replace(/\$WORKER_NUM/g, String(workerNum));
      prompt = prompt.replace(/\$\{WORKER_NUM\}/g, String(workerNum));
    }

    return prompt;
  }

  /**
   * Get the poke message for a role
   */
  getPokeMessage(role: AgentRole): string {
    return this.config.pokeMessages[role];
  }

  /**
   * Build environment variables for an agent
   */
  private buildEnvVars(
    role: AgentRole,
    squad?: SquadName,
    workerNum?: number,
    worktreeDir?: string
  ): Record<string, string> {
    const vars: Record<string, string> = {};

    if (squad) {
      vars['SQUAD_NAME'] = squad;
    }

    if (workerNum !== undefined) {
      vars['WORKER_NUM'] = String(workerNum);
    }

    if (role === 'em' && worktreeDir) {
      vars['EM_WORKTREE'] = worktreeDir;
    }

    return vars;
  }

  /**
   * Sleep helper
   */
  private sleep(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }
}

export default AgentManager;
