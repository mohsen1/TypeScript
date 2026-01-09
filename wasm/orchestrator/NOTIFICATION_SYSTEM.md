# Notification-Based Orchestration System

This document describes the file-based notification system that replaces the time-based idle monitoring.

## Overview

Instead of polling tmux panes to detect idle agents, the new system uses explicit notifications:

1. **Workers/EMs write to notification files** when they need attention
2. **A watcher script** monitors file changes and sends tmux keys to notify managers
3. **Notifications are instant** - no more waiting for idle timeouts

## Components

### 1. Notification Script (`.notify/notify.sh`)

Simple bash script that agents call to send notifications:

```bash
# When work is ready for review
.notify/notify.sh ready "Completed feature X"

# When you need a new task
.notify/notify.sh task "Ready for assignment"

# When you're blocked
.notify/notify.sh blocked "Build fails with error Y"

# When branch is ready for merge
.notify/notify.sh merge "Pushed to worker/forge-1"

# Status update
.notify/notify.sh status "Working on tests"
```

### 2. Notification Watcher (`npm run watcher`)

Background process that watches for notifications and forwards them to tmux:

```bash
# Start with defaults
npm run watcher

# Or with options
npx tsx src/watcher-cli.ts --session zang-org --verbose --poll-interval 500
```

### 3. TypeScript API

Programmatic notification sending:

```typescript
import { writeNotification } from './notify/index.js';

writeNotification(
  '/path/to/.notify',
  'worker/forge-1',     // sender
  'ready_for_review',   // type
  'Completed task'      // message
);
```

## Message Flow

```
Worker completes task
    ↓
Worker runs: .notify/notify.sh ready "Done"
    ↓
Watcher detects new notification
    ↓
Watcher sends tmux keys to EM pane
    ↓
EM receives notification and takes action
    ↓
EM runs: .notify/notify.sh merge "Merged workers"
    ↓
Watcher sends tmux keys to Director pane
```

## Notification Types

| Type | When to Use |
|------|-------------|
| `ready` | Work is ready for review |
| `task` | Need a new task assignment |
| `blocked` | Blocked and need help |
| `merge` | Branch is ready to merge |
| `status` | General status update |

## File Format

Notifications are stored as JSON lines in `.notify/<sender>.notify`:

```json
{"timestamp":"2026-01-09T15:00:00Z","sender":"worker/forge-1","type":"ready_for_review","message":"Done"}
{"timestamp":"2026-01-09T15:10:00Z","sender":"worker/forge-1","type":"need_task","message":"Ready"}
```

## Testing

```bash
# Dry-run test (creates tmux session but no agents)
npm run test:notify

# Manual test
cd /Users/mohsenazimi/code/TypeScript/wasm/orchestrator
./test-notify/manual-test.sh

# Run with cheap model for testing
./test-notify/run-test.sh  # Uses gpt-4o-mini
```

## Integration with Orchestrator

The watcher should be started alongside the orchestrator:

```bash
# In one terminal
npm run watcher -- --session zang-org

# In another terminal
zang-org --start
```

Or integrate into the orchestrator startup sequence.

## Configuration

Squad-to-pane mapping is in `src/notify/types.ts`:

```typescript
const DEFAULT_SQUAD_PANES: Record<string, number> = {
  forge: 1,  // EM-Forge is in director window, pane 1
  anvil: 2,  // EM-Anvil is in director window, pane 2
};
```

## Benefits

1. **Instant response** - No more 2-10 minute idle timeouts
2. **Works with Codex** - No hooks needed, just shell scripts
3. **Explicit communication** - Agents clearly signal their state
4. **Stacked requests** - Multiple notifications queue up for managers
5. **Easy debugging** - Just `cat .notify/*.notify` to see history
