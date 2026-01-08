/**
 * Tests for utility functions
 */

import { describe, it, expect } from 'vitest';
import { contentHash, checksum } from '../src/utils/hash.js';
import { shellEscape, heredoc } from '../src/utils/exec.js';

describe('contentHash', () => {
  it('returns consistent hash for same content', () => {
    const content = 'test content';
    expect(contentHash(content)).toBe(contentHash(content));
  });

  it('returns different hash for different content', () => {
    expect(contentHash('content a')).not.toBe(contentHash('content b'));
  });

  it('handles empty string', () => {
    expect(contentHash('')).toBe(contentHash(''));
    expect(contentHash('')).toHaveLength(32); // MD5 hex length
  });

  it('handles unicode content', () => {
    const hash = contentHash('こんにちは世界');
    expect(hash).toHaveLength(32);
  });
});

describe('checksum', () => {
  it('returns consistent checksum for same content', () => {
    const content = 'test content';
    expect(checksum(content)).toBe(checksum(content));
  });

  it('returns different checksum for different content', () => {
    expect(checksum('content a')).not.toBe(checksum('content b'));
  });

  it('returns numeric string', () => {
    const result = checksum('test');
    expect(/^\d+$/.test(result)).toBe(true);
  });
});

describe('shellEscape', () => {
  it('wraps simple strings in single quotes', () => {
    expect(shellEscape('hello')).toBe("'hello'");
  });

  it('handles empty string', () => {
    expect(shellEscape('')).toBe("''");
  });

  it('escapes single quotes within strings', () => {
    expect(shellEscape("it's")).toBe("'it'\\''s'");
  });

  it('handles multiple single quotes', () => {
    const input = "don't say 'never'";
    const escaped = shellEscape(input);
    expect(escaped).toContain("'\\''");
  });

  it('preserves special characters inside single quotes', () => {
    const input = 'echo $HOME && rm -rf /';
    const escaped = shellEscape(input);
    expect(escaped).toBe("'echo $HOME && rm -rf /'");
  });

  it('handles newlines', () => {
    const input = 'line1\nline2';
    const escaped = shellEscape(input);
    expect(escaped).toContain('\n');
  });
});

describe('heredoc', () => {
  it('creates proper heredoc format', () => {
    const content = 'line1\nline2';
    const result = heredoc(content);
    expect(result).toBe("cat <<'EOF'\nline1\nline2\nEOF");
  });

  it('uses custom delimiter', () => {
    const result = heredoc('content', 'CUSTOM');
    expect(result).toContain("<<'CUSTOM'");
    expect(result).toContain('\nCUSTOM');
  });

  it('handles empty content', () => {
    const result = heredoc('');
    expect(result).toBe("cat <<'EOF'\n\nEOF");
  });
});
