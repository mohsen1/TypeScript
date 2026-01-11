# Root Cause Analysis: Why the System Stopped

## Timeline
- **06:00-08:00**: System running well, 62 commits achieved
- **08:00-11:30**: System went idle, all workers stopped, no new commits

## Root Cause

### The Fundamental Issue: Codex Prompt Timing

**Problem**: `tmux send-keys` requires a delay between sending text and pressing Enter for Codex agents.

**What was happening**:
```bash
# OLD CODE (broken):
tmux send-keys -t pane "prompt text"
tmux send-keys -t pane C-m  # Enter pressed immediately
```

Codex needs time to:
1. Receive the text in the UI
2. Process it as a user message
3. Prepare to respond

**Immediate Enter** = Codex doesn't see it as a complete message, prompt gets ignored.

### Where This Occurred

1. **`.zang_org_monitor.sh`** (lines 171-173):
   - Sent prompts without delay
   - Workers/EMs received text but never processed it

2. **`.notify/send-prompt.sh`**:
   - Had 1-second delay, but insufficient for reliable delivery
   - Needed 2+ seconds

### Why It Manifested After 3 Hours

1. Workers completed tasks → became merge-ready
2. Monitor detected "recent activity" → didn't send prompts (working as designed)
3. Workers sat idle at prompts waiting for new assignments
4. EMs didn't pro actively check (also waiting at prompts)
5. No mechanism to restart the cycle

## The Fix

### 1. Updated `send-prompt.sh`
```bash
tmux send-keys -t "$PANE" "$MESSAGE"
sleep 2  # CRITICAL: Increased from 1 to 2 seconds
tmux send-keys -t "$PANE" Enter
```

### 2. Updated `.zang_org_monitor.sh`
```bash
tmux send-keys -t "$SESSION:$window.$pane" "$poke"
sleep 2  # CRITICAL: Added 2-second delay
tmux send-keys -t "$SESSION:$window.$pane" C-m
```

### 3. Rule for Future

**ALWAYS use this pattern**:
1. Send text
2. Sleep 2+ seconds
3. Send Enter

**NEVER**:
- Send text and Enter in same command
- Use delays < 2 seconds
- Assume tmux will batch correctly

## Testing the Fix

```bash
# Good:
tmux send-keys -t pane "message"
sleep 2
tmux send-keys -t pane Enter

# Bad:
tmux send-keys -t pane "message" Enter  # Too fast!
```

## Prevention

1. All new scripts that send prompts must use `.notify/send-prompt.sh`
2. Never use `tmux send-keys` directly for prompts
3. Monitor script should be only place with direct send-keys (with delay)

## Status After Fix

- Monitor restarted with 2-second delays
- send-prompt.sh updated to 2 seconds
- Workers kicked off manually (9/10 active)
- System should now self-sustain
