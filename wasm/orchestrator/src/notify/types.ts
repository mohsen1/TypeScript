/**
 * Types for the file-based notification system
 */

export interface NotificationRequest {
  /** ISO timestamp of the request */
  timestamp: string;
  /** Who sent the notification (e.g., "worker/forge-1", "em/forge") */
  sender: string;
  /** Type of notification */
  type: 'ready_for_review' | 'need_task' | 'blocked' | 'merge_ready' | 'status_update';
  /** Optional message/context */
  message?: string | undefined;
}

export interface NotificationFile {
  /** Path to the notification file */
  path: string;
  /** Last modification time (for change detection) */
  lastModified: number;
  /** Parsed requests from the file */
  requests: NotificationRequest[];
}

export interface WatcherConfig {
  /** Base directory for notification files */
  notifyDir: string;
  /** tmux session name */
  session: string;
  /** Polling interval in milliseconds */
  pollInterval: number;
  /** Callback when notifications are received */
  onNotification: (request: NotificationRequest, target: NotificationTarget) => Promise<void>;
}

export interface NotificationTarget {
  /** tmux pane target (e.g., "zang-org:director.0") */
  pane: string;
  /** Role of the target (director, em, worker) */
  role: 'director' | 'em' | 'worker';
  /** Squad name if applicable */
  squad?: string | undefined;
}

/**
 * Default squad to pane mapping
 * Can be overridden by passing a custom map
 */
const DEFAULT_SQUAD_PANES: Record<string, number> = {
  forge: 1,
  anvil: 2,
  test: 1, // For testing
};

export function getManagerTarget(
  sender: string,
  session: string,
  squadPanes: Record<string, number> = DEFAULT_SQUAD_PANES
): NotificationTarget {
  // Workers notify their EM
  // EMs notify the Director

  if (sender.startsWith('worker/')) {
    // worker/forge-1 -> EM-Forge (director window, pane 1 for forge, pane 2 for anvil)
    const match = sender.match(/worker\/(\w+)-\d+/);
    if (match) {
      const squad = match[1]!;
      // Default to pane 1 if squad not found in mapping
      const pane = squadPanes[squad] ?? 1;
      return {
        pane: `${session}:director.${pane}`,
        role: 'em',
        squad,
      };
    }
  }

  if (sender.startsWith('em/')) {
    // em/forge -> Director (director window, pane 0)
    const squad = sender.replace('em/', '');
    return {
      pane: `${session}:director.0`,
      role: 'director',
      squad,
    };
  }

  // Default to director
  return {
    pane: `${session}:director.0`,
    role: 'director',
  };
}
