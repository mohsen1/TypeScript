#!/usr/bin/env node
/**
 * CLI entry point for the Zang Orchestrator
 *
 * Usage:
 *   zang-org [options]
 *
 * Options:
 *   --kill           Kill the session (saves state)
 *   --resume         Resume from saved state
 *   --fresh          Reset all branches and start fresh
 *   --graceful-exit  Signal agents to finish current work and exit cleanly
 *   --codex          Use OpenAI Codex instead of Claude
 *   --help           Show help
 */

import { Orchestrator } from './Orchestrator.js';
import {
  createConfig,
  configFromEnv,
  validateConfig,
  type ConfigInput,
} from './config/index.js';
import type { AgentType, OperationMode } from './types.js';

interface CliArgs {
  mode: OperationMode;
  agentType: AgentType;
  help: boolean;
}

function parseArgs(args: string[]): CliArgs {
  const result: CliArgs = {
    mode: 'start',
    agentType: 'claude',
    help: false,
  };

  for (const arg of args) {
    switch (arg) {
      case '--kill':
        result.mode = 'kill';
        break;
      case '--resume':
        result.mode = 'resume';
        break;
      case '--fresh':
        result.mode = 'fresh';
        break;
      case '--graceful-exit':
        result.mode = 'graceful-exit';
        break;
      case '--codex':
        result.agentType = 'codex';
        break;
      case '--help':
      case '-h':
        result.help = true;
        break;
    }
  }

  return result;
}

function printHelp(): void {
  console.log(`
Zang Organization Orchestrator

Creates a hierarchical AI agent organization:
  Window 1 (director): Director agent + EMs (one per squad)
  Window N (squad):    Workers for each configured squad

Default Configuration:
  - 2 squads: forge (5 workers), anvil (5 workers)
  - Squads can be customized via configuration

Usage:
  zang-org [options]

Options:
  --kill           Kill the session (saves state for resume)
  --resume         Resume from saved state
  --fresh          Reset all branches to origin/rust and start fresh
  --graceful-exit  Signal agents to finish current work and exit cleanly
  --codex          Use OpenAI Codex instead of Claude
  --help           Show this help message

Environment Variables:
  SESSION               Session name (default: zang-org)
  AUTO_FETCH            Auto-fetch from origin (default: 1)
  AUTO_RESTART_AGENT    Auto-restart crashed agents (default: 1)
  AUTO_MONITOR          Enable idle monitoring (default: 1)
  AUTO_ATTACH           Auto-attach after start (default: 1)
  AGENT_AUTO_UPDATE     Auto-update agent CLI (default: 1)

  DIRECTOR_IDLE_SECONDS Director idle threshold (default: 600)
  EM_IDLE_SECONDS       EM idle threshold (default: 60)
  WORKER_IDLE_SECONDS   Worker idle threshold (default: 300)

  DIRECTOR_POKE         Director poke message
  EM_POKE               EM poke message
  WORKER_POKE           Worker poke message

Examples:
  zang-org                  Start with Claude (default)
  zang-org --codex          Start with OpenAI Codex
  zang-org --kill           Save state and kill session
  zang-org --resume         Continue from saved state
  zang-org --fresh          Reset branches and start fresh
  zang-org --graceful-exit  Finish current work and exit
`);
}

async function main(): Promise<void> {
  const args = parseArgs(process.argv.slice(2));

  if (args.help) {
    printHelp();
    process.exit(0);
  }

  // Build configuration
  const envOverrides = configFromEnv();
  const configInput: ConfigInput = {
    agentType: args.agentType,
    overrides: envOverrides,
  };

  let config;
  try {
    config = createConfig(configInput);
  } catch (error) {
    console.error('Configuration error:', (error as Error).message);
    process.exit(1);
  }

  // Validate configuration
  const errors = validateConfig(config);
  if (errors.length > 0) {
    console.error('Configuration validation errors:');
    for (const error of errors) {
      console.error(`  - ${error}`);
    }
    process.exit(1);
  }

  // Run orchestrator
  const orchestrator = new Orchestrator({
    config,
    mode: args.mode,
  });

  try {
    await orchestrator.run();
  } catch (error) {
    console.error('Error:', (error as Error).message);
    process.exit(1);
  }
}

main().catch((error) => {
  console.error('Fatal error:', error);
  process.exit(1);
});
