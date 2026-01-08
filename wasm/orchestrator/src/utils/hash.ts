/**
 * Simple hash utility for content comparison
 */

import { createHash } from 'node:crypto';

/**
 * Calculate a fast hash for content comparison
 * Uses MD5 for speed - not for cryptographic purposes
 */
export function contentHash(content: string): string {
  return createHash('md5').update(content).digest('hex');
}

/**
 * Calculate CRC32-like checksum (similar to bash cksum)
 */
export function checksum(content: string): string {
  let crc = 0xffffffff;
  for (let i = 0; i < content.length; i++) {
    crc ^= content.charCodeAt(i);
    for (let j = 0; j < 8; j++) {
      crc = (crc >>> 1) ^ (crc & 1 ? 0xedb88320 : 0);
    }
  }
  return ((crc ^ 0xffffffff) >>> 0).toString();
}
