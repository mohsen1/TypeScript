/**
 * NotificationWatcher - Monitors notification files and triggers manager notifications
 *
 * This replaces the time-based IdleMonitor with an event-driven system.
 * Workers/EMs write to notification files when they need attention,
 * and this watcher notifies the appropriate manager via tmux.
 */

import { watch, existsSync, mkdirSync, readFileSync, statSync, writeFileSync, readdirSync, appendFileSync } from 'node:fs';
import { join, basename } from 'node:path';
import { TmuxClient } from '../tmux/index.js';
import type { NotificationRequest, NotificationTarget, WatcherConfig } from './types.js';
import { getManagerTarget } from './types.js';

export class NotificationWatcher {
  private readonly config: WatcherConfig;
  private readonly tmux: TmuxClient;
  private readonly processedRequests: Set<string> = new Set();
  private watcher: ReturnType<typeof watch> | null = null;
  private pollInterval: ReturnType<typeof setInterval> | null = null;
  private lastMtimes: Map<string, number> = new Map();

  constructor(config: WatcherConfig, tmux?: TmuxClient) {
    this.config = config;
    this.tmux = tmux ?? new TmuxClient();
  }

  /**
   * Start watching for notifications
   */
  start(): void {
    // Ensure notify directory exists
    if (!existsSync(this.config.notifyDir)) {
      mkdirSync(this.config.notifyDir, { recursive: true });
    }

    // Use polling since fs.watch can be unreliable across platforms
    this.pollInterval = setInterval(() => {
      this.checkForNotifications().catch((err) => {
        console.error('[NotificationWatcher] Error checking notifications:', err);
      });
    }, this.config.pollInterval);

    console.log(`[NotificationWatcher] Started watching ${this.config.notifyDir}`);
  }

  /**
   * Stop watching
   */
  stop(): void {
    if (this.watcher) {
      this.watcher.close();
      this.watcher = null;
    }
    if (this.pollInterval) {
      clearInterval(this.pollInterval);
      this.pollInterval = null;
    }
    console.log('[NotificationWatcher] Stopped');
  }

  /**
   * Check all notification files for new requests
   */
  private async checkForNotifications(): Promise<void> {
    const files = this.getNotificationFiles();

    for (const file of files) {
      await this.processNotificationFile(file);
    }
  }

  /**
   * Get list of notification files in the notify directory
   */
  private getNotificationFiles(): string[] {
    try {
      const entries = readdirSync(this.config.notifyDir);
      return entries
        .filter((f: string) => f.endsWith('.notify'))
        .map((f: string) => join(this.config.notifyDir, f));
    } catch {
      return [];
    }
  }

  /**
   * Process a single notification file
   */
  private async processNotificationFile(filePath: string): Promise<void> {
    try {
      const stat = statSync(filePath);
      const lastMtime = this.lastMtimes.get(filePath) ?? 0;

      // Only process if file was modified
      if (stat.mtimeMs <= lastMtime) {
        return;
      }

      this.lastMtimes.set(filePath, stat.mtimeMs);

      const content = readFileSync(filePath, 'utf-8');
      const requests = this.parseNotificationFile(content);

      for (const request of requests) {
        const requestKey = `${request.sender}:${request.timestamp}:${request.type}`;

        if (this.processedRequests.has(requestKey)) {
          continue;
        }

        this.processedRequests.add(requestKey);

        // Determine target manager
        const target = getManagerTarget(request.sender, this.config.session);

        // Trigger callback
        await this.config.onNotification(request, target);
      }
    } catch (err) {
      // File may not exist or be malformed, ignore
    }
  }

  /**
   * Parse notification file content
   * Format: one JSON object per line
   */
  private parseNotificationFile(content: string): NotificationRequest[] {
    const requests: NotificationRequest[] = [];
    const lines = content.trim().split('\n').filter(Boolean);

    for (const line of lines) {
      try {
        const request = JSON.parse(line) as NotificationRequest;
        if (request.timestamp && request.sender && request.type) {
          requests.push(request);
        }
      } catch {
        // Invalid JSON, skip
      }
    }

    return requests;
  }

  /**
   * Clear processed requests (useful for testing or reset)
   */
  clearProcessed(): void {
    this.processedRequests.clear();
    this.lastMtimes.clear();
  }
}

/**
 * Write a notification request to a file
 * This is what agents call when they need attention
 */
export function writeNotification(
  notifyDir: string,
  sender: string,
  type: NotificationRequest['type'],
  message?: string
): void {
  // Ensure directory exists
  if (!existsSync(notifyDir)) {
    mkdirSync(notifyDir, { recursive: true });
  }

  const request: NotificationRequest = {
    timestamp: new Date().toISOString(),
    sender,
    type,
    message,
  };

  // Each sender has their own file
  const safeFileName = sender.replace(/\//g, '-');
  const filePath = join(notifyDir, `${safeFileName}.notify`);

  // Append to file (one JSON per line)
  const line = JSON.stringify(request) + '\n';

  appendFileSync(filePath, line);
}

/**
 * Create default notification handler that sends tmux messages
 */
export function createTmuxNotificationHandler(
  tmux: TmuxClient
): (request: NotificationRequest, target: NotificationTarget) => Promise<void> {
  return async (request, target) => {
    const message = formatNotificationMessage(request);

    console.log(`[Notify] ${request.sender} -> ${target.role}${target.squad ? ` (${target.squad})` : ''}: ${request.type}`);

    const paneId = {
      session: target.pane.split(':')[0]!,
      window: target.pane.split(':')[1]!.split('.')[0]!,
      pane: parseInt(target.pane.split('.')[1]!, 10),
    };

    // IMPORTANT: For Codex, must send keys, wait 1s, then send literal "Enter"
    // Send message to target pane
    await tmux.sendKeys(paneId, message);

    // 1 second delay before sending Enter (required for Codex)
    await new Promise((r) => setTimeout(r, 1000));

    // Send literal "Enter" key (pressEnter=true sends C-m which Codex recognizes)
    await tmux.sendKeys(paneId, '', true);
  };
}

/**
 * Format a notification into a message for the manager
 */
function formatNotificationMessage(request: NotificationRequest): string {
  const time = new Date(request.timestamp).toLocaleTimeString();

  switch (request.type) {
    case 'ready_for_review':
      return `[${time}] NOTIFICATION from ${request.sender}: Ready for review/merge. ${request.message ?? 'Please check worker branch.'}`;
    case 'need_task':
      return `[${time}] NOTIFICATION from ${request.sender}: Needs a task. ${request.message ?? 'Worker is idle and ready for assignment.'}`;
    case 'blocked':
      return `[${time}] NOTIFICATION from ${request.sender}: BLOCKED - ${request.message ?? 'Needs assistance.'}`;
    case 'merge_ready':
      return `[${time}] NOTIFICATION from ${request.sender}: Merge ready. ${request.message ?? 'Branch ready to be merged.'}`;
    case 'status_update':
      return `[${time}] STATUS from ${request.sender}: ${request.message ?? 'Status update.'}`;
    default:
      return `[${time}] NOTIFICATION from ${request.sender}: ${request.message ?? 'Needs attention.'}`;
  }
}

export default NotificationWatcher;
