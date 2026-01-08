/**
 * Promisified exec utilities for running shell commands
 */

import { exec, spawn, type SpawnOptions } from 'node:child_process';
import { promisify } from 'node:util';
import type { CommandResult } from '../types.js';

const execAsync = promisify(exec);

export interface ExecOptions {
  cwd?: string;
  timeout?: number;
  env?: Record<string, string>;
}

/**
 * Execute a command and return stdout/stderr/exitCode
 */
export async function runCommand(
  command: string,
  options: ExecOptions = {}
): Promise<CommandResult> {
  try {
    const { stdout, stderr } = await execAsync(command, {
      cwd: options.cwd,
      timeout: options.timeout,
      env: { ...process.env, ...options.env },
      maxBuffer: 10 * 1024 * 1024, // 10MB
    });
    return { stdout: stdout.trim(), stderr: stderr.trim(), exitCode: 0 };
  } catch (error: unknown) {
    if (isExecError(error)) {
      return {
        stdout: error.stdout?.trim() ?? '',
        stderr: error.stderr?.trim() ?? '',
        exitCode: error.code ?? 1,
      };
    }
    throw error;
  }
}

/**
 * Execute a command and return only stdout, throwing on non-zero exit
 */
export async function runCommandStrict(
  command: string,
  options: ExecOptions = {}
): Promise<string> {
  const result = await runCommand(command, options);
  if (result.exitCode !== 0) {
    throw new Error(
      `Command failed with exit code ${result.exitCode}: ${result.stderr || result.stdout}`
    );
  }
  return result.stdout;
}

/**
 * Check if a command exists in PATH
 */
export async function commandExists(cmd: string): Promise<boolean> {
  const result = await runCommand(`command -v ${cmd}`);
  return result.exitCode === 0 && result.stdout.length > 0;
}

/**
 * Spawn a background process that persists after parent exits
 */
export function spawnBackground(
  command: string,
  args: string[],
  options: SpawnOptions = {}
): void {
  const child = spawn(command, args, {
    ...options,
    detached: true,
    stdio: 'ignore',
  });
  child.unref();
}

/**
 * Escape a string for safe use in shell commands
 */
export function shellEscape(str: string): string {
  // Use single quotes and escape any single quotes within
  return `'${str.replace(/'/g, "'\\''")}'`;
}

/**
 * Create a heredoc-style command for multi-line input
 */
export function heredoc(content: string, delimiter = 'EOF'): string {
  return `cat <<'${delimiter}'\n${content}\n${delimiter}`;
}

interface ExecError extends Error {
  stdout?: string;
  stderr?: string;
  code?: number;
}

function isExecError(error: unknown): error is ExecError {
  return error instanceof Error && 'code' in error;
}
