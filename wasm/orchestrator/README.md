# Zang Orchestrator

TypeScript orchestrator for the multi-agent AI development system.

## Overview

Creates a hierarchical AI agent organization using tmux:

```
Director (hands-off management)
    ├── EM-Forge (manages type system workers)
    │       └── Workers 1-5 (forge squad)
    └── EM-Anvil (manages output workers)
            └── Workers 1-5 (anvil squad)
```

## Installation

```bash
cd wasm/orchestrator
npm install
npm run build
```

## Usage

```bash
# Start with Claude (default)
node dist/cli.js

# Start with OpenAI Codex
node dist/cli.js --codex

# Kill session (saves state)
node dist/cli.js --kill

# Resume from saved state
node dist/cli.js --resume

# Fresh start (reset all branches)
node dist/cli.js --fresh

# Show help
node dist/cli.js --help
```

## Development

```bash
# Run in development mode
npm run dev

# Run tests
npm test

# Watch mode tests
npm run test:watch

# Type checking
npm run typecheck

# Build
npm run build
```

## Architecture

The orchestrator is composed of several modules:

- **TmuxClient** - Handles all tmux session, window, and pane operations
- **AgentManager** - Spawns and manages AI agents (Claude/Codex)
- **IdleMonitor** - Tracks pane activity and pokes idle agents
- **StateManager** - Saves and restores session state for resume
- **WorktreeManager** - Manages git worktrees for each agent
- **Config** - Typed configuration with environment variable support

## Configuration

Configuration can be set via environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `SESSION` | `zang-org` | tmux session name |
| `DIRECTOR_IDLE_SECONDS` | `600` | Director idle threshold |
| `EM_IDLE_SECONDS` | `60` | EM idle threshold |
| `WORKER_IDLE_SECONDS` | `300` | Worker idle threshold |
| `AUTO_FETCH` | `1` | Auto-fetch from origin |
| `AUTO_RESTART_AGENT` | `1` | Auto-restart crashed agents |
| `AUTO_MONITOR` | `1` | Enable idle monitoring |
| `AUTO_ATTACH` | `1` | Auto-attach after start |

## Tests

127 tests covering:
- Utility functions (hash, exec, shell escape)
- Configuration creation and validation
- TmuxClient operations
- AgentManager prompt generation
- IdleMonitor pane tracking
- StateManager save/load
- WorktreeManager path generation
- Orchestrator module structure
