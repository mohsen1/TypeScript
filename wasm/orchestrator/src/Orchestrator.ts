/**
 * Orchestrator - Main class that coordinates the multi-agent system
 *
 * Sets up the tmux session with Director, EMs, and Workers.
 * Handles start, kill, resume, and fresh modes.
 */

import { TmuxClient } from './tmux/index.js';
import { AgentManager } from './agent/index.js';
import { IdleMonitor, type MonitoredPane } from './monitor/index.js';
import { StateManager } from './state/index.js';
import { WorktreeManager } from './worktree/index.js';
import type { OrchestratorConfig, OperationMode, PaneId, SquadName, SquadConfig } from './types.js';

export interface OrchestratorOptions {
  config: OrchestratorConfig;
  mode?: OperationMode;
}

export class Orchestrator {
  private readonly config: OrchestratorConfig;
  private readonly tmux: TmuxClient;
  private readonly agent: AgentManager;
  private readonly monitor: IdleMonitor;
  private readonly state: StateManager;
  private readonly worktree: WorktreeManager;
  private readonly mode: OperationMode;

  constructor(options: OrchestratorOptions) {
    this.config = options.config;
    this.mode = options.mode ?? 'start';

    this.tmux = new TmuxClient();
    this.agent = new AgentManager(this.tmux, this.config);
    this.monitor = new IdleMonitor(this.tmux, this.agent, this.config);
    this.state = new StateManager(this.tmux, this.config);
    this.worktree = new WorktreeManager(this.config);
  }

  /**
   * Get configured squads
   */
  private get squads(): readonly SquadConfig[] {
    return this.config.squads;
  }

  /**
   * Get squad names
   */
  private getSquadNames(): string[] {
    return this.config.squads.map((s) => s.name);
  }

  /**
   * Get worker numbers for a squad
   */
  private getWorkerNums(squad: SquadName): number[] {
    const squadConfig = this.config.squads.find((s) => s.name === squad);
    if (!squadConfig) {
      return [];
    }
    return Array.from({ length: squadConfig.workerCount }, (_, i) => i + 1);
  }

  /**
   * Run the orchestrator in the configured mode
   */
  async run(): Promise<void> {
    switch (this.mode) {
      case 'kill':
        await this.kill();
        break;
      case 'resume':
        await this.resume();
        break;
      case 'fresh':
        await this.fresh();
        break;
      case 'graceful-exit':
        await this.gracefulExit();
        break;
      case 'start':
      default:
        await this.start();
        break;
    }
  }

  /**
   * Start a new session
   */
  async start(): Promise<void> {
    // Check prerequisites
    await this.validatePrerequisites();

    // Check if session already exists
    if (await this.tmux.sessionExists(this.config.session)) {
      console.log(
        `Session ${this.config.session} already exists. Use --kill to restart.`
      );
      return;
    }

    console.log(`Starting Zang Organization (${this.config.agentType})...`);

    // Setup phase
    await this.setup(false);

    // Create tmux session
    await this.createSession();

    // Start agents
    await this.startAllAgents();

    // Start monitor if enabled
    if (this.config.autoMonitor) {
      this.startMonitor();
    }

    // Print summary
    this.printSummary();

    // Auto-attach if enabled
    if (this.config.autoAttach && !this.tmux.isInsideTmux()) {
      await this.tmux.attach(this.config.session);
    }
  }

  /**
   * Kill the session (with state save)
   */
  async kill(): Promise<void> {
    if (!(await this.tmux.sessionExists(this.config.session))) {
      console.log(`No session found: ${this.config.session}`);
      return;
    }

    console.log('Saving session state...');
    await this.state.saveState();

    console.log(`Killing session: ${this.config.session}`);
    await this.tmux.killSession(this.config.session);

    console.log(`Session killed. State saved to: ${this.state.stateFile}`);
    console.log("Use '--resume' to continue where you left off");
  }

  /**
   * Resume from saved state
   */
  async resume(): Promise<void> {
    if (!this.state.stateExists()) {
      console.log(`No saved state found at ${this.state.stateFile}`);
      console.log('Starting fresh session instead...');
      await this.start();
      return;
    }

    console.log('Resuming from saved state...');
    console.log(await this.state.formatStateInfo());

    // Check if session already exists
    if (await this.tmux.sessionExists(this.config.session)) {
      console.log(
        `Session ${this.config.session} already exists. Use --kill first.`
      );
      return;
    }

    // Setup phase (don't reset branches)
    await this.setup(false);

    // Create session and start agents with resume prompts
    await this.createSession();
    await this.startAllAgents(true);

    // Start monitor if enabled
    if (this.config.autoMonitor) {
      this.startMonitor();
    }

    // Print summary
    this.printSummary();
    console.log('\n*** RESUMED from saved state ***');

    // Auto-attach if enabled
    if (this.config.autoAttach && !this.tmux.isInsideTmux()) {
      await this.tmux.attach(this.config.session);
    }
  }

  /**
   * Fresh start (reset all branches)
   */
  async fresh(): Promise<void> {
    console.log('Fresh mode: Resetting all branches to origin/rust...');

    // Kill existing session if present
    if (await this.tmux.sessionExists(this.config.session)) {
      await this.tmux.killSession(this.config.session);
    }

    // Delete saved state
    await this.state.deleteState();

    // Setup with fresh flag
    await this.setup(true);

    // Normal start
    await this.createSession();
    await this.startAllAgents();

    // Start monitor if enabled
    if (this.config.autoMonitor) {
      this.startMonitor();
    }

    // Print summary
    this.printSummary();
    console.log('\n*** FRESH START - All branches reset to origin/rust ***');

    // Auto-attach if enabled
    if (this.config.autoAttach && !this.tmux.isInsideTmux()) {
      await this.tmux.attach(this.config.session);
    }
  }

  /**
   * Graceful exit - wait for current work to finish without assigning new tasks
   */
  async gracefulExit(): Promise<void> {
    if (!(await this.tmux.sessionExists(this.config.session))) {
      console.log(`No session found: ${this.config.session}`);
      return;
    }

    console.log('Graceful exit: Signaling agents to finish current work...');

    // Send graceful exit message to all workers
    const { session } = this.config;
    const gracefulMessage =
      'GRACEFUL EXIT: Finish your current task, commit and push your work, then exit. Do not start new tasks.';

    // Send to all workers
    for (const squad of this.squads) {
      for (const workerNum of this.getWorkerNums(squad.name)) {
        const paneNum = workerNum - 1;
        const pane: PaneId = { session, window: squad.name, pane: paneNum };
        console.log(`  Signaling ${squad.name}-${workerNum}...`);
        await this.tmux.sendKeys(pane, gracefulMessage);
        await this.sleep(500);
        await this.tmux.sendKeys(pane, 'Enter');
      }
    }

    // Send to EMs
    for (let i = 0; i < this.squads.length; i++) {
      const squad = this.squads[i]!;
      const pane: PaneId = { session, window: 'director', pane: i + 1 };
      console.log(`  Signaling EM-${squad.name}...`);
      await this.tmux.sendKeys(
        pane,
        'GRACEFUL EXIT: Workers are finishing. Review their work, merge what is ready, then exit. Do not assign new tasks.'
      );
      await this.sleep(500);
      await this.tmux.sendKeys(pane, 'Enter');
    }

    // Send to Director
    const directorPane: PaneId = { session, window: 'director', pane: 0 };
    console.log('  Signaling Director...');
    await this.tmux.sendKeys(
      directorPane,
      'GRACEFUL EXIT: EMs and workers are finishing current work. After all merges are done, summarize the session and exit.'
    );
    await this.sleep(500);
    await this.tmux.sendKeys(directorPane, 'Enter');

    console.log('\nGraceful exit signals sent to all agents.');
    console.log('Agents will finish current work and exit on their own.');
    console.log(
      'Monitor the session to see when all agents have exited, then use --kill to clean up.'
    );
  }

  /**
   * Validate prerequisites
   */
  private async validatePrerequisites(): Promise<void> {
    // Check tmux
    if (!(await this.tmux.isAvailable())) {
      throw new Error('tmux not found in PATH');
    }

    // Check agent CLI
    if (!(await this.agent.isAgentAvailable())) {
      throw new Error(`${this.agent.getAgentCommand()} not found in PATH`);
    }

    // Check git
    const { commandExists } = await import('./utils/index.js');
    if (!(await commandExists('git'))) {
      throw new Error('git not found in PATH');
    }
  }

  /**
   * Setup worktrees and spec directories
   */
  private async setup(fresh: boolean): Promise<void> {
    // Auto-update agent if enabled
    if (this.config.agentAutoUpdate) {
      console.log(`Updating ${this.agent.getAgentCommand()}...`);
      await this.agent.updateAgent();
    }

    // Fetch from origin
    await this.worktree.fetch();

    // Ensure rust branch exists
    await this.worktree.ensureRustBranch();

    // Fresh mode: reset all branches
    if (fresh) {
      await this.worktree.resetAllBranches();
    }

    // Ensure all worktrees exist
    console.log('Setting up worktrees...');
    await this.worktree.ensureAllWorktrees(fresh);

    // Setup role-specific AGENTS.md files
    console.log('Setting up role-specific agent instructions...');
    await this.worktree.setupRoleAgents();

    // Setup squad spec directories
    await this.worktree.setupSquadSpecs();
  }

  /**
   * Create the tmux session with windows and panes
   */
  private async createSession(): Promise<void> {
    const { session, rootDir } = this.config;

    // Create session with director window
    await this.tmux.createSession(session, 'director', rootDir);

    // Split director window for EMs
    // Pane 0: Director (already exists)
    // Subsequent panes: one per EM
    for (let i = 0; i < this.squads.length; i++) {
      const squad = this.squads[i]!;
      const emDir = this.worktree.getEmWorktreePath(squad.name);

      if (i === 0) {
        // First EM: split right from director
        await this.tmux.splitPane(
          { session, window: 'director', pane: 0 },
          { horizontal: true, cwd: emDir }
        );
      } else {
        // Subsequent EMs: split below previous EM
        await this.tmux.splitPane(
          { session, window: 'director', pane: i },
          { horizontal: false, cwd: emDir }
        );
      }
    }

    // Set pane titles
    await this.tmux.setPaneTitle({ session, window: 'director', pane: 0 }, 'director');
    for (let i = 0; i < this.squads.length; i++) {
      const squad = this.squads[i]!;
      await this.tmux.setPaneTitle(
        { session, window: 'director', pane: i + 1 },
        `em-${squad.name}`
      );
    }

    // Create squad windows (workers only)
    for (const squad of this.squads) {
      await this.createSquadWindow(squad.name);
    }

    // Select director window
    await this.tmux.selectWindow(session, 'director');
  }

  /**
   * Create a squad window with worker panes
   */
  private async createSquadWindow(squad: SquadName): Promise<void> {
    const { session } = this.config;
    const workerNums = this.getWorkerNums(squad);

    if (workerNums.length === 0) {
      return;
    }

    // Create window for first worker
    const firstWorkerDir = this.worktree.getWorkerWorktreePath(squad, 1);
    await this.tmux.createWindow(session, squad, firstWorkerDir);

    // Create panes for additional workers
    for (let i = 2; i <= workerNums.length; i++) {
      const workerDir = this.worktree.getWorkerWorktreePath(squad, i);
      // Split from last pane
      await this.tmux.splitPane(
        { session, window: squad, pane: i - 2 },
        { cwd: workerDir }
      );
      await this.tmux.selectLayout(session, squad, 'tiled');
    }

    // Final layout balance
    await this.tmux.selectLayout(session, squad, 'tiled');

    // Set pane titles
    for (let i = 0; i < workerNums.length; i++) {
      await this.tmux.setPaneTitle(
        { session, window: squad, pane: i },
        `${squad}-${i + 1}`
      );
    }
  }

  /**
   * Start all agents in their panes
   */
  private async startAllAgents(resume = false): Promise<void> {
    const { session, rootDir, timing } = this.config;
    const monitoredPanes: MonitoredPane[] = [];

    // Start Director
    console.log('Starting Director...');
    const directorPane: PaneId = { session, window: 'director', pane: 0 };
    const directorResumePrompt = resume
      ? await this.state.getResumePrompt('director_0')
      : null;

    await this.agent.startAgent({
      pane: directorPane,
      role: 'director',
      cwd: rootDir,
      resumePrompt: directorResumePrompt ?? undefined,
    });
    monitoredPanes.push({ pane: directorPane, role: 'director' });

    // Start EMs (in director window)
    for (let i = 0; i < this.squads.length; i++) {
      const squad = this.squads[i]!;
      const paneNum = i + 1;
      const emPane: PaneId = { session, window: 'director', pane: paneNum };
      const emDir = this.worktree.getEmWorktreePath(squad.name);

      console.log(`Starting EM-${squad.name}...`);
      const emResumePrompt = resume
        ? await this.state.getResumePrompt(`director_${paneNum}`)
        : null;

      await this.agent.startAgent({
        pane: emPane,
        role: 'em',
        squad: squad.name,
        cwd: emDir,
        resumePrompt: emResumePrompt ?? undefined,
      });
      monitoredPanes.push({ pane: emPane, role: 'em' });
    }

    // Start Workers (in squad windows)
    for (const squad of this.squads) {
      console.log(`Starting ${squad.name} squad workers...`);

      for (const workerNum of this.getWorkerNums(squad.name)) {
        const paneNum = workerNum - 1;
        const workerPane: PaneId = { session, window: squad.name, pane: paneNum };
        const workerDir = this.worktree.getWorkerWorktreePath(squad.name, workerNum);

        const workerResumePrompt = resume
          ? await this.state.getResumePrompt(`${squad.name}_${paneNum}`)
          : null;

        await this.agent.startAgent({
          pane: workerPane,
          role: 'worker',
          squad: squad.name,
          workerNum,
          cwd: workerDir,
          resumePrompt: workerResumePrompt ?? undefined,
        });

        monitoredPanes.push({ pane: workerPane, role: 'worker' });

        // Stagger worker starts
        await this.sleep(timing.staggerPause * 1000);
      }
    }

    // Register panes with monitor
    this.monitor.registerPanes(monitoredPanes);
  }

  /**
   * Start the idle monitor
   */
  private startMonitor(): void {
    console.log('Starting idle monitor...');
    this.monitor.start(5000); // Check every 5 seconds
  }

  /**
   * Print session summary
   */
  private printSummary(): void {
    const { session, agentType, agentArgs } = this.config;

    console.log('\n==============================================');
    console.log('Zang Organization Started');
    console.log('==============================================');
    console.log(`Session: ${session}`);
    console.log('');
    console.log('Windows:');

    // Build director window description
    const emNames = this.squads.map((s) => `EM-${s.name.charAt(0).toUpperCase() + s.name.slice(1)}`);
    console.log(`  1. director  - Director (pane 0) + ${emNames.join(' + ')} (panes 1-${this.squads.length})`);

    // Print squad windows
    let windowNum = 2;
    for (const squad of this.squads) {
      const workerCount = squad.workerCount;
      console.log(
        `  ${windowNum}. ${squad.name.padEnd(10)} - ${workerCount} Workers (panes 0-${workerCount - 1})`
      );
      windowNum++;
    }
    console.log('');
    console.log('EM Worktrees:');
    for (const squad of this.squads) {
      const dir = this.worktree.getEmWorktreePath(squad.name);
      const branch = this.worktree.getEmBranch(squad.name);
      console.log(`  em-${squad.name}: ${dir} (branch: ${branch})`);
    }
    console.log('');
    console.log('Worker Worktrees:');
    for (const squad of this.squads) {
      for (const num of this.getWorkerNums(squad.name)) {
        const dir = this.worktree.getWorkerWorktreePath(squad.name, num);
        console.log(`  ${squad.name}-${num}: ${dir}`);
      }
    }
    console.log('');
    console.log(`Agent CLI: ${agentType} ${agentArgs}`);
    console.log('');
    console.log('Quick navigation:');
    console.log(`  tmux select-window -t ${session}:director`);
    for (const squad of this.squads) {
      console.log(`  tmux select-window -t ${session}:${squad.name}`);
    }
    console.log('');
    console.log(`Attach: tmux attach -t ${session}`);
    console.log('Kill:          zang-org --kill           (saves state for resume)');
    console.log('Resume:        zang-org --resume         (continue where you left off)');
    console.log('Fresh:         zang-org --fresh          (reset all branches to origin/rust)');
    console.log('Graceful Exit: zang-org --graceful-exit  (finish work and exit cleanly)');
    console.log('==============================================');
  }

  /**
   * Sleep helper
   */
  private sleep(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }
}

export default Orchestrator;
