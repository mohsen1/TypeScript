# Hook-Based Orchestrator System

A simplified orchestrator using Claude Code hooks instead of custom notification systems.

## Overview

This system replaces the file-based notification watcher with Claude Code's built-in hooks:

- **Workers** use `Stop` hooks to report completion when they finish work
- **Managers** (EMs/Director) use `UserPromptSubmit` hooks to check for updates
- **Shared state** in `.orchestrator/state.json` tracks all agent statuses

No separate notification watcher process needed!

## Architecture

```
┌─────────────┐     Stop Hook      ┌──────────────────┐
│   Worker    │ ──────────────────>│  State File      │
│ (worktree)  │                     │  (.orchestrator/) │
└─────────────┘                     └──────────────────┘
                                              │
                                              │ updates
                                              ▼
┌─────────────┐     UserPromptSubmit ┌──────────────────┐
│   Manager   │ <────────────────────│  Notifications   │
│ (em/director)│   checks on input   │  (pending work)  │
└─────────────┘                     └──────────────────┘
```

## How It Works

### Worker Flow

1. Worker completes a task in their worktree
2. Worker's Claude Code session ends
3. **Stop hook** (`worker-stop.sh`) runs automatically
4. Hook writes to shared state file and creates notification
5. Worker session closes

### Manager Flow

1. Manager receives any input (user prompt or message)
2. **UserPromptSubmit hook** (`manager-check.sh`) runs automatically
3. Hook checks for pending notifications
4. If updates exist, they're injected as context
5. Manager sees the updates and can take action

## File Structure

```
.claude/
├── settings.json           # Hook configuration
└── hooks/
    ├── worker-stop.sh      # Reports worker completion
    └── manager-check.sh    # Checks for updates

.orchestrator/
├── state.json              # Shared agent state
└── notifications/          # Pending notifications
    ├── worker-1.json       # Individual worker notifications
    └── processed/          # Processed notifications
```

## State File Format

`.orchestrator/state.json`:
```json
{
  "version": "1.0",
  "last_updated": "2025-01-11T10:30:00Z",
  "agents": {
    "TypeScript-forge-1-track": {
      "status": "completed",
      "message": "Worker session completed",
      "updated_at": "2025-01-11T10:30:00Z",
      "transcript": "/path/to/transcript.jsonl"
    }
  }
}
```

## Notification Format

`.orchestrator/notifications/*.json`:
```json
{
  "timestamp": "2025-01-11T10:30:00Z",
  "agent": "TypeScript-forge-1-track",
  "squad": "forge",
  "status": "completed",
  "message": "Worker session completed"
}
```

## Installation

### Option 1: Automatic Setup (Recommended)

Run the setup script from the main repository:

```bash
cd /Users/mohsenazimi/code/TypeScript/wasm
./.orchestrator/setup-hooks.sh
```

This will automatically:
- Detect all git worktrees
- Create `.claude/settings.json` in each worktree
- Reference the shared hooks from `.orchestrator/shared-hooks/`

### Option 2: Manual Setup

1. Each worktree needs a `.claude/settings.json` that references the shared hooks:

```bash
# In each worktree
mkdir -p .claude
cat > .claude/settings.json <<'EOF'
{
  "hooks": {
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "\"$CLAUDE_PROJECT_DIR\"/.orchestrator/shared-hooks/worker-stop.sh",
            "timeout": 30
          }
        ]
      }
    ],
    "UserPromptSubmit": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "\"$CLAUDE_PROJECT_DIR\"/.orchestrator/shared-hooks/manager-check.sh",
            "timeout": 30
          }
        ]
      }
    ]
  }
}
EOF
```

2. Set agent name environment variable (optional):
   ```bash
   export CLAUDE_AGENT_NAME="TypeScript-forge-1-track"
   ```

### Verify Installation

```bash
# List all worktrees
./.orchestrator/setup-hooks.sh --list

# Dry run to see what would be installed
./.orchestrator/setup-hooks.sh --dry-run
```

## Usage

### Workers

Workers don't need to do anything special! Just:
1. Complete your work
2. Let Claude Code finish (exit normally)
3. The Stop hook automatically reports completion

### Managers

Managers will see updates automatically when they send any message:
1. Type "check status" or any prompt
2. The UserPromptSubmit hook injects pending updates as context
3. Review the updates and take action

## Hook Configuration

### Worker Worktrees

The `Stop` hook runs when a worker finishes:
```json
{
  "hooks": {
    "Stop": [{
      "hooks": [{
        "type": "command",
        "command": "\"$CLAUDE_PROJECT_DIR\"/.claude/hooks/worker-stop.sh"
      }]
    }]
  }
}
```

### Manager Worktrees

The `UserPromptSubmit` hook checks for updates on each input:
```json
{
  "hooks": {
    "UserPromptSubmit": [{
      "hooks": [{
        "type": "command",
        "command": "\"$CLAUDE_PROJECT_DIR\"/.claude/hooks/manager-check.sh"
      }]
    }]
  }
}
```

## Advantages Over File-Based System

1. **No separate watcher process** - Hooks run inline with Claude Code
2. **Instant notification** - Hooks fire immediately on events
3. **Simpler architecture** - No polling, file watchers, or tmux key injection
4. **Built into Claude Code** - Uses native hook system
5. **Less maintenance** - No separate notification watcher to manage

## Environment Variables

- `CLAUDE_PROJECT_DIR` - Automatically set by Claude Code
- `CLAUDE_AGENT_NAME` - Optional, defaults to basename of project directory

## Debugging

Enable verbose mode in Claude Code (`ctrl+o`) to see hook output:
```
[Worker Stop Hook] Agent: TypeScript-forge-1-track, Status: completed
[Manager Check Hook] Found forge updates for TypeScript-em-forge
```

View the state file:
```bash
cat .orchestrator/state.json | jq
```

View pending notifications:
```bash
ls -la .orchestrator/notifications/
cat .orchestrator/notifications/*.json | jq
```

## Migration from File-Based System

The hook-based system can coexist with the old notification system:

1. Keep the old `.notify/notify.sh` for manual notifications
2. Hooks handle automatic completion reporting
3. Gradually migrate to hooks-only approach

To fully migrate:
1. Remove `npm run watcher` from startup
2. Remove `.notify/` directory
3. Deploy `.claude/hooks/` to all worktrees

## Example Workflow

### Worker completes a task:
```bash
# Worker in their worktree
$ git status
$ git commit -m "Implement feature X"
# Worker exits session
```

**Automatic:** Stop hook fires, writes to state:
```json
{
  "agents": {
    "TypeScript-forge-1": {
      "status": "completed",
      "message": "Worker session completed"
    }
  }
}
```

### Manager checks in:
```bash
# EM types any message
EM> check on workers
```

**Automatic:** UserPromptSubmit hook injects context:
```
ORCHESTRATOR UPDATES:
- [2025-01-11T10:30:00Z] TypeScript-forge-1: completed - Worker session completed

EM> (sees updates and can review/merge)
```

## Testing

Test the hooks manually:

```bash
# Test worker stop hook
export CLAUDE_PROJECT_DIR="/Users/mohsenazimi/code/TypeScript/wasm"
export CLAUDE_AGENT_NAME="test-worker"
echo '{"session_id":"test","transcript_path":"/tmp/test.jsonl"}' | \
  .claude/hooks/worker-stop.sh

# Check state
cat .orchestrator/state.json | jq

# Test manager check hook
export CLAUDE_PROJECT_DIR="/Users/mohsenazimi/code/TypeScript/wasm"
export CLAUDE_AGENT_NAME="test-manager"
echo '{"prompt":"check status"}' | \
  .claude/hooks/manager-check.sh
```

## Future Enhancements

- Add more notification types (blocked, need_task, etc.)
- Implement merge request tracking
- Add performance metrics
- Create dashboard for viewing state

## Related Files

- `orchestrator/NOTIFICATION_SYSTEM.md` - Old file-based system (deprecated)
- `orchestrator/src/notify/NotificationWatcher.ts` - Old watcher (no longer needed)
