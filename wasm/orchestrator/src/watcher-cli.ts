#!/usr/bin/env node
/**
 * Notification Watcher CLI
 *
 * Runs as a background process to watch for notifications from agents
 * and forward them to the appropriate managers via tmux.
 *
 * Usage:
 *   watcher [options]
 *
 * Options:
 *   --notify-dir <path>   Directory for notification files (default: ~/code/TypeScript/.notify)
 *   --session <name>      Tmux session name (default: zang-org)
 *   --poll-interval <ms>  Polling interval in ms (default: 1000)
 *   --verbose             Enable verbose logging
 */

import { NotificationWatcher, createTmuxNotificationHandler } from './notify/index.js';
import { TmuxClient } from './tmux/index.js';
import { join } from 'node:path';
import { homedir } from 'node:os';

interface WatcherOptions {
  notifyDir: string;
  session: string;
  pollInterval: number;
  verbose: boolean;
}

function parseArgs(): WatcherOptions {
  const args = process.argv.slice(2);
  const options: WatcherOptions = {
    notifyDir: join(homedir(), 'code', 'TypeScript', '.notify'),
    session: 'zang-org',
    pollInterval: 1000,
    verbose: false,
  };

  for (let i = 0; i < args.length; i++) {
    const arg = args[i]!;
    switch (arg) {
      case '--notify-dir':
        options.notifyDir = args[++i] ?? options.notifyDir;
        break;
      case '--session':
        options.session = args[++i] ?? options.session;
        break;
      case '--poll-interval':
        options.pollInterval = parseInt(args[++i] ?? '1000', 10);
        break;
      case '--verbose':
      case '-v':
        options.verbose = true;
        break;
      case '--help':
      case '-h':
        console.log(`
Notification Watcher - Monitors agent notifications and forwards to managers

Usage: watcher [options]

Options:
  --notify-dir <path>   Directory for notification files
                        Default: ~/code/TypeScript/.notify
  --session <name>      Tmux session name
                        Default: zang-org
  --poll-interval <ms>  Polling interval in milliseconds
                        Default: 1000
  --verbose, -v         Enable verbose logging
  --help, -h            Show this help message

Examples:
  watcher                                    # Use defaults
  watcher --session test-org --verbose       # Custom session with logging
  watcher --poll-interval 500                # Faster polling
`);
        process.exit(0);
    }
  }

  return options;
}

async function main(): Promise<void> {
  const options = parseArgs();

  console.log('===========================================');
  console.log('Notification Watcher Starting');
  console.log('===========================================');
  console.log(`Notify Directory: ${options.notifyDir}`);
  console.log(`Tmux Session:     ${options.session}`);
  console.log(`Poll Interval:    ${options.pollInterval}ms`);
  console.log('===========================================');

  const tmux = new TmuxClient();

  // Check if tmux session exists
  const sessionExists = await tmux.sessionExists(options.session);
  if (!sessionExists) {
    console.warn(`Warning: Tmux session '${options.session}' does not exist yet.`);
    console.warn('Watcher will run but notifications will fail until session is created.');
  }

  const handler = createTmuxNotificationHandler(tmux);

  const watcher = new NotificationWatcher(
    {
      notifyDir: options.notifyDir,
      session: options.session,
      pollInterval: options.pollInterval,
      onNotification: async (request, target) => {
        if (options.verbose) {
          console.log(`[${new Date().toISOString()}] Processing notification:`, request);
        }
        await handler(request, target);
      },
    },
    tmux
  );

  // Handle graceful shutdown
  process.on('SIGINT', () => {
    console.log('\nShutting down watcher...');
    watcher.stop();
    process.exit(0);
  });

  process.on('SIGTERM', () => {
    console.log('\nShutting down watcher...');
    watcher.stop();
    process.exit(0);
  });

  watcher.start();
  console.log('\nWatcher is running. Press Ctrl+C to stop.\n');
}

main().catch((err) => {
  console.error('Fatal error:', err);
  process.exit(1);
});
