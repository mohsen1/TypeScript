/**
 * WorktreeManager - Manages git worktrees for workers and EMs
 *
 * Creates and manages separate worktrees for each agent, allowing
 * them to work on different branches without interference.
 */

import { existsSync, mkdirSync, copyFileSync } from 'node:fs';
import { join } from 'node:path';
import { runCommand, runCommandStrict } from '../utils/index.js';
import type { OrchestratorConfig, SquadName, WorktreeInfo } from '../types.js';

const SQUADS: SquadName[] = ['forge', 'anvil'];
const WORKER_NUMS = [1, 2, 3, 4, 5];

export class WorktreeManager {
  private readonly config: OrchestratorConfig;

  constructor(config: OrchestratorConfig) {
    this.config = config;
  }

  /**
   * Fetch from origin with prune
   */
  async fetch(): Promise<void> {
    if (!this.config.autoFetch) {
      return;
    }
    await runCommand('git fetch --prune origin', { cwd: this.config.rootDir });
  }

  /**
   * Ensure the rust branch exists
   */
  async ensureRustBranch(): Promise<void> {
    const { rootDir } = this.config;

    // Check if local rust branch exists
    const localResult = await runCommand(
      'git show-ref --verify --quiet refs/heads/rust',
      { cwd: rootDir }
    );

    if (localResult.exitCode === 0) {
      return; // Already exists
    }

    // Check if remote rust branch exists
    const remoteResult = await runCommand(
      'git show-ref --verify --quiet refs/remotes/origin/rust',
      { cwd: rootDir }
    );

    if (remoteResult.exitCode === 0) {
      // Create tracking branch from origin
      await runCommand('git branch --track rust origin/rust', { cwd: rootDir });
    } else {
      // Create local rust branch
      await runCommand('git branch rust', { cwd: rootDir });
    }
  }

  /**
   * Get worktree path for a worker
   */
  getWorkerWorktreePath(squad: SquadName, workerNum: number): string {
    return join(this.config.worktreeBase, `TypeScript-${squad}-${workerNum}-track`);
  }

  /**
   * Get worktree path for an EM
   */
  getEmWorktreePath(squad: SquadName): string {
    return join(this.config.worktreeBase, `TypeScript-em-${squad}`);
  }

  /**
   * Get worker branch name
   */
  getWorkerBranch(squad: SquadName, workerNum: number): string {
    return `worker/${squad}-${workerNum}`;
  }

  /**
   * Get EM branch name
   */
  getEmBranch(squad: SquadName): string {
    return `em/${squad}`;
  }

  /**
   * Get squad branch name
   */
  getSquadBranch(squad: SquadName): string {
    return `squad/${squad}`;
  }

  /**
   * Check if a path is a git worktree
   */
  async isWorktree(path: string): Promise<boolean> {
    const result = await runCommand(
      `git worktree list --porcelain | awk '/^worktree /{print $2}'`,
      { cwd: this.config.rootDir }
    );

    if (result.exitCode !== 0) {
      return false;
    }

    const worktrees = result.stdout.split('\n').filter(Boolean);
    return worktrees.includes(path);
  }

  /**
   * Ensure a worktree exists for a worker
   */
  async ensureWorkerWorktree(
    squad: SquadName,
    workerNum: number,
    fresh = false
  ): Promise<WorktreeInfo> {
    const dir = this.getWorkerWorktreePath(squad, workerNum);
    const branch = this.getWorkerBranch(squad, workerNum);

    return this.ensureWorktree(dir, branch, fresh);
  }

  /**
   * Ensure a worktree exists for an EM
   */
  async ensureEmWorktree(
    squad: SquadName,
    fresh = false
  ): Promise<WorktreeInfo> {
    const dir = this.getEmWorktreePath(squad);
    const branch = this.getEmBranch(squad);

    return this.ensureWorktree(dir, branch, fresh);
  }

  /**
   * Ensure a worktree exists at the given path
   */
  private async ensureWorktree(
    dir: string,
    branch: string,
    fresh: boolean
  ): Promise<WorktreeInfo> {
    const { rootDir } = this.config;

    if (existsSync(dir)) {
      const isWt = await this.isWorktree(dir);
      if (!isWt) {
        console.warn(`${dir} exists but is not a git worktree`);
        return { path: dir, branch, exists: false };
      }

      if (fresh) {
        // Reset worktree to origin/rust
        await runCommand('git fetch origin', { cwd: dir });
        await runCommand('git reset --hard origin/rust', { cwd: dir });
        await runCommand('git clean -fd', { cwd: dir });
        await runCommand(`git checkout -B "${branch}" origin/rust`, {
          cwd: dir,
        });
      }

      return { path: dir, branch, exists: true };
    }

    // Create new worktree
    const result = await runCommand(
      `git worktree add --force "${dir}" rust`,
      { cwd: rootDir }
    );

    if (result.exitCode !== 0) {
      console.warn(`Could not create worktree: ${result.stderr}`);
      return { path: dir, branch, exists: false };
    }

    // Create and checkout branch
    await runCommand(`git checkout -B "${branch}" origin/rust`, { cwd: dir });

    return { path: dir, branch, exists: true };
  }

  /**
   * Ensure all worker and EM worktrees exist
   */
  async ensureAllWorktrees(fresh = false): Promise<Map<string, WorktreeInfo>> {
    const worktrees = new Map<string, WorktreeInfo>();

    // Create EM worktrees
    for (const squad of SQUADS) {
      const info = await this.ensureEmWorktree(squad, fresh);
      worktrees.set(`em-${squad}`, info);
    }

    // Create worker worktrees
    for (const squad of SQUADS) {
      for (const num of WORKER_NUMS) {
        const info = await this.ensureWorkerWorktree(squad, num, fresh);
        worktrees.set(`${squad}-${num}`, info);
      }
    }

    return worktrees;
  }

  /**
   * Reset all branches to origin/rust (fresh mode)
   */
  async resetAllBranches(): Promise<void> {
    const { rootDir } = this.config;

    // Reset squad branches
    for (const squad of SQUADS) {
      const branch = this.getSquadBranch(squad);
      await runCommand(`git branch -D "${branch}"`, { cwd: rootDir });
      await runCommand(`git branch "${branch}" origin/rust`, { cwd: rootDir });
    }

    // Reset worker branches
    for (const squad of SQUADS) {
      for (const num of WORKER_NUMS) {
        const branch = this.getWorkerBranch(squad, num);
        await runCommand(`git branch -D "${branch}"`, { cwd: rootDir });
        await runCommand(`git branch "${branch}" origin/rust`, { cwd: rootDir });
      }
    }
  }

  /**
   * Setup role-specific AGENTS.md files
   */
  async setupRoleAgents(): Promise<void> {
    const { rootDir } = this.config;

    // Director: .role/AGENTS.md = DIRECTOR_AGENT.md
    const directorRoleDir = join(rootDir, '.role');
    mkdirSync(directorRoleDir, { recursive: true });
    const directorSrc = join(rootDir, 'DIRECTOR_AGENT.md');
    const directorDst = join(directorRoleDir, 'AGENTS.md');
    if (existsSync(directorSrc)) {
      copyFileSync(directorSrc, directorDst);
    }

    // EM worktrees: .role/AGENTS.md = SQUAD_LEAD_AGENT.md
    for (const squad of SQUADS) {
      const emDir = this.getEmWorktreePath(squad);
      if (existsSync(emDir)) {
        const roleDir = join(emDir, '.role');
        mkdirSync(roleDir, { recursive: true });
        const src = join(rootDir, 'SQUAD_LEAD_AGENT.md');
        const dst = join(roleDir, 'AGENTS.md');
        if (existsSync(src)) {
          copyFileSync(src, dst);
        }
      }
    }

    // Worker worktrees: .role/AGENTS.md = AGENTS.md
    for (const squad of SQUADS) {
      for (const num of WORKER_NUMS) {
        const workerDir = this.getWorkerWorktreePath(squad, num);
        if (existsSync(workerDir)) {
          const roleDir = join(workerDir, '.role');
          mkdirSync(roleDir, { recursive: true });
          const src = join(rootDir, 'AGENTS.md');
          const dst = join(roleDir, 'AGENTS.md');
          if (existsSync(src)) {
            copyFileSync(src, dst);
          }
        }
      }
    }
  }

  /**
   * Create squad spec directories and goal files
   */
  async setupSquadSpecs(): Promise<void> {
    const { rootDir } = this.config;
    const squadDir = join(rootDir, 'wasm', 'specs', 'squads');

    for (const squad of SQUADS) {
      const dir = join(squadDir, squad);
      mkdirSync(dir, { recursive: true });

      // Create GOALS.md if it doesn't exist
      const goalsFile = join(dir, 'GOALS.md');
      if (!existsSync(goalsFile)) {
        await this.createGoalsFile(squad, goalsFile);
      }

      // Create worker plans if they don't exist
      for (const num of WORKER_NUMS) {
        const planFile = join(dir, `worker-${num}_plan.md`);
        if (!existsSync(planFile)) {
          await this.createWorkerPlan(squad, num, planFile);
        }
      }
    }
  }

  private async createGoalsFile(squad: SquadName, path: string): Promise<void> {
    const squadCap = squad.charAt(0).toUpperCase() + squad.slice(1);
    const date = new Date().toISOString().split('T')[0];

    const content = `# Squad ${squadCap} Goals

Updated: ${date}

## Current Milestone
[Director: Set the current milestone based on Project Direction]

## Objectives (Ranked)
1. **[Objective 1]**
   - Context: [Why this matters]
   - Success Criteria: [Measurable outcome]
   - Key Files: [Paths]
   - Estimated Complexity: [Low/Medium/High]

## Anti-Priorities
- [Director: List things NOT to work on]

## Cross-Squad Dependencies
- None currently

## Notes to EM
- Read wasm/specs/WASM_ARCHITECTURE.md
- Use Docker for tests: ./wasm/test.sh

## Squad Status
- Last EM Report: Not yet started
- Workers Active: 0/5
- Branches Pending Merge: None
- Current Focus: Awaiting Director assignment
- Blockers: None
`;

    const { writeFile } = await import('node:fs/promises');
    await writeFile(path, content);
  }

  private async createWorkerPlan(
    squad: SquadName,
    num: number,
    path: string
  ): Promise<void> {
    const squadCap = squad.charAt(0).toUpperCase() + squad.slice(1);

    const content = `# Worker ${num} Plan

## Mission
Execute tasks assigned by EM-${squadCap} for the ${squadCap} squad.

Status: Active
Priority: ${num}

## Current Assignment
- [EM: Assign initial task]

## Task Queue
- [ ] [EM: Add queued tasks]

## Completed
- [x] (Move finished items here with brief notes and tests run)

## Ready for Merge
No

## Notes
- Follow \`wasm/specs/WASM_ARCHITECTURE.md\`
- Use Docker for Rust tests: \`./wasm/test.sh\`
- Commit format: \`[wasm] <component>: <description>\`
- Sync before each task: \`git fetch origin && git merge origin/rust --no-edit\`
- Push to: \`origin/worker/${squad}-${num}\`
`;

    const { writeFile } = await import('node:fs/promises');
    await writeFile(path, content);
  }
}

export default WorktreeManager;
