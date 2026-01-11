# Notification System for Multi-Agent Coordination

## Problem: Codex Agents Don't Respond to External Prompts

Codex agents are designed to respond to user input, not programmatic prompts. The `send-prompt.sh` script attempts to simulate user input via `tmux send-keys`, but **this rarely works reliably**.

## Solution: Polling-Based Status Checks

Instead of trying to "notify" agents, we **check their state directly** using tmux pane captures and notification files.

## How It Works

### 1. Agents Write Notifications (Outbound)

Workers and EMs write JSON notifications to `.notify/*.notify` files:

```bash
# Worker reports ready
WORKER_NUM=1 SQUAD_NAME=forge .notify/notify.sh ready "TS2454 implementation complete"

# EM reports merge
SQUAD_NAME=forge .notify/notify.sh merge "Merged all 5 workers into squad/forge"
```

This writes to `.notify/worker-forge-1.notify` or `.notify/em-forge.notify`:

```json
{"timestamp":"2026-01-11T05:30:00Z","sender":"worker/forge-1","type":"ready_for_review","message":"TS2454 implementation complete"}
```

### 2. Director Polls State (Inbound)

The Director doesn't wait for notifications - it **actively checks** state:

```bash
# Check notification files
cat .notify/em-forge.notify
cat .notify/worker-forge-1.notify

# Check live pane output (most reliable)
tmux capture-pane -p -t zang-org:director.1 -S -100  # EM-Forge
tmux capture-pane -p -t zang-org:forge.1 -S -80      # Worker 1
```

**Pane output is the source of truth**, not notification files.

### 3. send-prompt.sh (Unreliable, But Try Anyway)

The `send-prompt.sh` script attempts to send prompts to Codex panes:

```bash
.notify/send-prompt.sh zang-org:forge.1 "New task: implement TS2454"
```

**How it works:**
1. Send the text via `tmux send-keys`
2. Wait 1 second (Codex needs time to process)
3. Send Enter key

**Why it often fails:**
- Codex agents may be in middle of task (busy)
- Codex may not recognize tmux input as "user input"
- Timing issues (1 second may not be enough)

**Best practice:** Use `send-prompt.sh` but **don't rely on it**. Always follow up by checking the pane output to verify the prompt was received and acted upon.

## Notification Types

| Type | Meaning | Sender |
|------|---------|--------|
| `ready_for_review` | Work complete, branch pushed, ready for merge | Worker |
| `need_task` | Worker is idle and needs new assignment | Worker |
| `blocked` | Worker is blocked by error or external dependency | Worker, EM |
| `merge_ready` | Squad branch merged and pushed | EM |
| `status_update` | General status update | Any |

## File Structure

```
.notify/
├── notify.sh              # Create notifications (used by agents)
├── send-prompt.sh         # Send prompts to panes (unreliable with Codex)
├── heartbeat.sh           # Monitor agent liveness (optional)
├── em-forge.notify        # EM-Forge notifications
├── em-anvil.notify        # EM-Anvil notifications
├── worker-forge-1.notify  # Worker Forge-1 notifications
├── worker-forge-2.notify  # Worker Forge-2 notifications
├── worker-forge-3.notify  # Worker Forge-3 notifications
├── worker-forge-4.notify  # Worker Forge-4 notifications
├── worker-forge-5.notify  # Worker Forge-5 notifications
├── worker-anvil-1.notify  # Worker Anvil-1 notifications
├── worker-anvil-2.notify  # Worker Anvil-2 notifications
├── worker-anvil-3.notify  # Worker Anvil-3 notifications
├── worker-anvil-4.notify  # Worker Anvil-4 notifications
└── worker-anvil-5.notify  # Worker Anvil-5 notifications
```

## Usage Examples

### Worker Reports Ready

```bash
# Set environment
export WORKER_NUM=1
export SQUAD_NAME=forge

# Send notification
.notify/notify.sh ready "Implemented TS2454 definite assignment, tests passing"

# Result: Appends to .notify/worker-forge-1.notify
```

### EM Reports Merge

```bash
# Set environment
export SQUAD_NAME=forge

# Send notification
.notify/notify.sh merge "Merged worker/forge-1,2,3,4,5 into squad/forge, pushed to origin"

# Result: Appends to .notify/em-forge.notify
```

### Director Checks EM Status

```bash
# Read notification file
tail -5 .notify/em-forge.notify

# Read pane directly (more reliable)
tmux capture-pane -p -t zang-org:director.1 -S -100
```

### Director Sends Task to Worker (Via EM)

```bash
# Director tells EM to assign task
.notify/send-prompt.sh zang-org:director.1 "W1 is merge-ready. Assign next task from backlog (TS2564 property initialization). Update worker-1_plan.md and sync worker now."

# Then verify EM received it
tmux capture-pane -p -t zang-org:director.1 -S -20
```

## Debugging Failed Prompts

If `send-prompt.sh` doesn't work:

1. **Check if pane exists:**
   ```bash
   tmux list-panes -t zang-org -F "#{pane_index}: #{pane_title}"
   ```

2. **Check if pane is responsive:**
   ```bash
   tmux capture-pane -p -t zang-org:forge.1 -S -20
   # Look for recent activity
   ```

3. **Manually verify prompt was received:**
   ```bash
   # Send prompt
   .notify/send-prompt.sh zang-org:forge.1 "Test prompt"

   # Check if it appears in pane
   sleep 2
   tmux capture-pane -p -t zang-org:forge.1 -S -10 | grep "Test prompt"
   ```

4. **Codex is busy:** If the agent is in the middle of a long task, it won't respond to new prompts. Wait for it to finish.

5. **Codex didn't see it as user input:** This is the fundamental limitation. Codex agents expect input from the terminal user, not programmatic tmux sends.

## Best Practices

1. **Always check panes directly** - `tmux capture-pane` is the source of truth
2. **Use send-prompt.sh optimistically** - Try it, but don't assume it worked
3. **Verify prompts were received** - Check pane output after sending
4. **Poll regularly** - Director should check EM panes every loop (not wait for notifications)
5. **Keep notification files for history** - Useful for debugging and audit trail
6. **Don't spam prompts** - If agent is busy, wait for them to finish

## Alternative: File-Based Task Queue

If `send-prompt.sh` continues to be unreliable, consider a file-based approach:

```bash
# Director writes task file
echo "TS2564 property initialization" > .notify/worker-forge-1.task

# Worker polls for tasks
if [ -f .notify/worker-forge-1.task ]; then
  TASK=$(cat .notify/worker-forge-1.task)
  rm .notify/worker-forge-1.task
  # Process task...
fi
```

But this requires workers to actively poll, which Codex agents don't do automatically.

## Conclusion

**The notification system is best-effort, not guaranteed delivery.**

The most reliable coordination method is:
1. Director polls EM panes regularly
2. Director reads notification history
3. Director infers worker state from pane output
4. Director updates plan files directly
5. Director tries `send-prompt.sh` but verifies receipt

This matches how real engineering management works: managers proactively check on teams, not wait for teams to report.
