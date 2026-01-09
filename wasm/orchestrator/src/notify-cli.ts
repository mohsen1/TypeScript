#!/usr/bin/env node
/**
 * CLI for sending notifications from agents
 *
 * Usage:
 *   notify ready_for_review [message]    - Signal work is ready for review
 *   notify need_task [message]           - Request a new task
 *   notify blocked [message]             - Signal you're blocked
 *   notify merge_ready [message]         - Signal branch is ready to merge
 *   notify status [message]              - Send a status update
 *
 * Environment variables:
 *   SQUAD_NAME  - Your squad (forge, anvil)
 *   WORKER_NUM  - Your worker number (1-5)
 *   NOTIFY_DIR  - Directory for notification files (default: ~/code/TypeScript/.notify)
 *
 * The sender is automatically determined from SQUAD_NAME and WORKER_NUM.
 * If neither is set, uses "unknown" as sender.
 */

import { writeNotification, NotificationRequest } from './notify/index.js';
import { join } from 'node:path';
import { homedir } from 'node:os';

function getSender(): string {
  const squad = process.env['SQUAD_NAME'];
  const workerNum = process.env['WORKER_NUM'];

  if (squad && workerNum) {
    return `worker/${squad}-${workerNum}`;
  }
  if (squad) {
    return `em/${squad}`;
  }
  return 'unknown';
}

function getNotifyDir(): string {
  return process.env['NOTIFY_DIR'] ?? join(homedir(), 'code', 'TypeScript', '.notify');
}

function main(): void {
  const args = process.argv.slice(2);

  if (args.length === 0) {
    console.error('Usage: notify <type> [message]');
    console.error('Types: ready_for_review, need_task, blocked, merge_ready, status');
    process.exit(1);
  }

  const typeArg = args[0]!;
  const message = args.slice(1).join(' ') || undefined;

  // Map short names to full types
  const typeMap: Record<string, NotificationRequest['type']> = {
    ready: 'ready_for_review',
    ready_for_review: 'ready_for_review',
    review: 'ready_for_review',
    task: 'need_task',
    need_task: 'need_task',
    blocked: 'blocked',
    block: 'blocked',
    merge: 'merge_ready',
    merge_ready: 'merge_ready',
    status: 'status_update',
    update: 'status_update',
    status_update: 'status_update',
  };

  const type = typeMap[typeArg.toLowerCase()];
  if (!type) {
    console.error(`Unknown notification type: ${typeArg}`);
    console.error('Valid types: ready_for_review, need_task, blocked, merge_ready, status');
    process.exit(1);
  }

  const sender = getSender();
  const notifyDir = getNotifyDir();

  writeNotification(notifyDir, sender, type, message);

  console.log(`Notification sent: ${type} from ${sender}`);
}

main();
