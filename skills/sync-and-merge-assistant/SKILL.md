---
name: sync-and-merge-assistant
description: Safely syncs with origin/rust using commit or stash, merge, and conflict triage.
---

# Sync and Merge Assistant

Use this when syncing with `origin/rust` without losing local work.

## Rules
- Never use destructive commands (`git reset --hard`, `git checkout --`).
- Keep changes by committing or stashing before merge.

## Workflow
1. Check status: `git status -sb`.
2. If there are changes to keep:
   - Prefer commit: `git add . && git commit -m "[wasm] <component>: <desc>"`
   - Or stash if work is incomplete: `git stash push -u -m "sync"`
3. Sync loop:
   - `git push origin rust` (if you committed)
   - `git fetch origin && git merge origin/rust`
   - Resolve conflicts; commit the merge if needed
   - `git push origin rust`
4. If merge fails due to local changes:
   - Commit or stash, then re-run the merge.

## Output
- Report merge results and any conflicts resolved.
- If stash was used, remember to `git stash pop` after merge and resolve any conflicts.
