#!/usr/bin/env node
/**
 * Error Distribution Analyzer
 * Analyzes and tracks error type distribution over time
 * Shows top 10 extra/missing errors and tracks changes
 */

import { createRequire } from 'module';
import { fileURLToPath } from 'url';
import { dirname, join, resolve } from 'path';
import { readFileSync, writeFileSync, existsSync, mkdirSync } from 'fs';

const require = createRequire(import.meta.url);
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const CONFIG = {
  metricsDir: resolve(__dirname, '../metrics-data'),
  errorDistFile: resolve(__dirname, '../metrics-data/error-distribution.json'),
};

const colors = {
  reset: '\x1b[0m',
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  cyan: '\x1b[36m',
  dim: '\x1b[2m',
  bold: '\x1b[1m',
};

function log(msg, color = '') {
  console.log(`${color}${msg}${colors.reset}`);
}

/**
 * Ensure metrics directory exists
 */
function ensureMetricsDir() {
  if (!existsSync(CONFIG.metricsDir)) {
    mkdirSync(CONFIG.metricsDir, { recursive: true });
  }
}

/**
 * Load error distribution data
 */
function loadErrorDistribution() {
  if (existsSync(CONFIG.errorDistFile)) {
    try {
      const data = readFileSync(CONFIG.errorDistFile, 'utf-8');
      return JSON.parse(data);
    } catch (e) {
      return { snapshots: [] };
    }
  }
  return { snapshots: [] };
}

/**
 * Save error distribution snapshot
 */
function saveErrorDistribution(data) {
  ensureMetricsDir();
  writeFileSync(CONFIG.errorDistFile, JSON.stringify(data, null, 2));
}

/**
 * Add new error distribution snapshot
 */
function addSnapshot(missingCodeCounts, extraCodeCounts, metadata = {}) {
  const data = loadErrorDistribution();

  // Get top 10 of each
  const sortedMissing = Object.entries(missingCodeCounts)
    .sort((a, b) => b[1] - a[1])
    .slice(0, 10)
    .map(([code, count]) => ({ code, count }));

  const sortedExtra = Object.entries(extraCodeCounts)
    .sort((a, b) => b[1] - a[1])
    .slice(0, 10)
    .map(([code, count]) => ({ code, count }));

  const snapshot = {
    timestamp: new Date().toISOString(),
    date: new Date().toLocaleDateString(),
    ...metadata,
    topMissingErrors: sortedMissing,
    topExtraErrors: sortedExtra,
    totalMissingCount: Object.values(missingCodeCounts).reduce((a, b) => a + b, 0),
    totalExtraCount: Object.values(extraCodeCounts).reduce((a, b) => a + b, 0),
  };

  data.snapshots.push(snapshot);

  // Keep only last 50 snapshots
  if (data.snapshots.length > 50) {
    data.snapshots = data.snapshots.slice(-50);
  }

  saveErrorDistribution(data);
  return snapshot;
}

/**
 * Compare error distributions between two snapshots
 */
function compareDistributions(current, previous) {
  if (!previous) return null;

  const currentMissing = new Map(current.topMissingErrors.map(e => [e.code, e.count]));
  const previousMissing = new Map(previous.topMissingErrors.map(e => [e.code, e.count]));

  const currentExtra = new Map(current.topExtraErrors.map(e => [e.code, e.count]));
  const previousExtra = new Map(previous.topExtraErrors.map(e => [e.code, e.count]));

  // Calculate changes
  const missingChanges = [];
  const extraChanges = [];

  const allMissingCodes = new Set([...currentMissing.keys(), ...previousMissing.keys()]);
  for (const code of allMissingCodes) {
    const currentCount = currentMissing.get(code) || 0;
    const previousCount = previousMissing.get(code) || 0;
    const change = currentCount - previousCount;
    if (change !== 0) {
      missingChanges.push({ code, change, currentCount, previousCount });
    }
  }

  const allExtraCodes = new Set([...currentExtra.keys(), ...previousExtra.keys()]);
  for (const code of allExtraCodes) {
    const currentCount = currentExtra.get(code) || 0;
    const previousCount = previousExtra.get(code) || 0;
    const change = currentCount - previousCount;
    if (change !== 0) {
      extraChanges.push({ code, change, currentCount, previousCount });
    }
  }

  return {
    missingChanges: missingChanges.sort((a, b) => Math.abs(b.change) - Math.abs(a.change)),
    extraChanges: extraChanges.sort((a, b) => Math.abs(b.change) - Math.abs(a.change)),
  };
}

/**
 * Display error distribution report
 */
function displayErrorDistributionReport(data, snapshots = 5) {
  const recent = data.snapshots.slice(-snapshots);
  if (recent.length === 0) {
    log('No error distribution data available.', colors.yellow);
    return;
  }

  log('\n' + '═'.repeat(70), colors.bold);
  log('  ERROR DISTRIBUTION ANALYZER', colors.bold);
  log('═'.repeat(70), colors.bold);

  const latest = recent[recent.length - 1];
  const previous = recent.length > 1 ? recent[recent.length - 2] : null;

  // Latest snapshot
  log('\n  Latest Snapshot:', colors.cyan);
  log(`    Date:              ${latest.date}`, colors.dim);
  log(`    Total Missing:     ${latest.totalMissingCount}`, colors.red);
  log(`    Total Extra:       ${latest.totalExtraCount}`, colors.yellow);

  // Top 10 missing errors
  if (latest.topMissingErrors.length > 0) {
    log('\n  Top 10 Missing Error Codes:', colors.red + colors.bold);
    log(`    ${'Code'.padEnd(10)} ${'Count'.padEnd(10)} ${'Description'}`, colors.dim);
    log(`    ${'─'.repeat(10)} ${'─'.repeat(10)} ${'─'.repeat(50)}`, colors.dim);

    const ts = require('typescript');
    for (const { code, count } of latest.topMissingErrors) {
      const diag = ts.getDiagnosticMessage?.(parseInt(code, 10));
      const desc = diag ? diag.message : 'Unknown error';
      const truncatedDesc = desc.length > 47 ? desc.slice(0, 47) + '...' : desc;
      log(`    TS${String(code).padEnd(8)} ${String(count).padEnd(10)} ${truncatedDesc}`);
    }
  } else {
    log('\n  ✓ No missing errors!', colors.green);
  }

  // Top 10 extra errors
  if (latest.topExtraErrors.length > 0) {
    log('\n  Top 10 Extra Error Codes:', colors.yellow + colors.bold);
    log(`    ${'Code'.padEnd(10)} ${'Count'.padEnd(10)} ${'Description'}`, colors.dim);
    log(`    ${'─'.repeat(10)} ${'─'.repeat(10)} ${'─'.repeat(50)}`, colors.dim);

    const ts = require('typescript');
    for (const { code, count } of latest.topExtraErrors) {
      const diag = ts.getDiagnosticMessage?.(parseInt(code, 10));
      const desc = diag ? diag.message : 'Unknown error';
      const truncatedDesc = desc.length > 47 ? desc.slice(0, 47) + '...' : desc;
      log(`    TS${String(code).padEnd(8)} ${String(count).padEnd(10)} ${truncatedDesc}`);
    }
  } else {
    log('\n  ✓ No extra errors!', colors.green);
  }

  // Comparison with previous
  if (previous) {
    const comparison = compareDistributions(latest, previous);

    if (comparison && (comparison.missingChanges.length > 0 || comparison.extraChanges.length > 0)) {
      log('\n  Changes Since Previous Run:', colors.cyan);

      if (comparison.missingChanges.length > 0) {
        log('\n    Missing Error Changes:', colors.dim);
        for (const { code, change, currentCount } of comparison.missingChanges.slice(0, 10)) {
          const arrow = change > 0 ? '↑' : '↓';
          const color = change < 0 ? colors.green : colors.red;
          log(`      TS${code}: ${change > 0 ? '+' : ''}${change} (${arrow}) ${color}→ ${currentCount} total${colors.reset}`);
        }
      }

      if (comparison.extraChanges.length > 0) {
        log('\n    Extra Error Changes:', colors.dim);
        for (const { code, change, currentCount } of comparison.extraChanges.slice(0, 10)) {
          const arrow = change > 0 ? '↑' : '↓';
          const color = change < 0 ? colors.green : colors.yellow;
          log(`      TS${code}: ${change > 0 ? '+' : ''}${change} (${arrow}) ${color}→ ${currentCount} total${colors.reset}`);
        }
      }
    } else {
      log('\n  ✓ No changes in error distribution since last run', colors.green);
    }
  }

  // Historical trend table
  if (recent.length > 1) {
    log('\n  Historical Trend:', colors.cyan);
    log(`    ${'Date'.padEnd(12)} ${'Total Missing'.padEnd(15)} ${'Total Extra'.padEnd(15)} ${'Top Missing'.padEnd(20)} ${'Top Extra'.padEnd(20)}`, colors.dim);
    log(`    ${'─'.repeat(12)} ${'─'.repeat(15)} ${'─'.repeat(15)} ${'─'.repeat(20)} ${'─'.repeat(20)}`, colors.dim);

    for (const snap of recent) {
      const shortDate = snap.date.split('/').slice(0, 2).join('/');
      const topMissing = snap.topMissingErrors.slice(0, 3).map(e => `TS${e.code}`).join(', ') || 'none';
      const topExtra = snap.topExtraErrors.slice(0, 3).map(e => `TS${e.code}`).join(', ') || 'none';

      log(`    ${shortDate.padEnd(12)} ${String(snap.totalMissingCount).padEnd(15)} ${String(snap.totalExtraCount).padEnd(15)} ${topMissing.padEnd(20)} ${topExtra.padEnd(20)}`);
    }
  }

  log('\n' + '═'.repeat(70) + '\n', colors.bold);
}

/**
 * Generate HTML report for error distribution
 */
function generateHtmlReport(data) {
  const snapshots = data.snapshots.slice(-20);
  if (snapshots.length === 0) return '';

  const latest = snapshots[snapshots.length - 1];

  const missingRows = latest.topMissingErrors.map(({ code, count }) => {
    const ts = require('typescript');
    const diag = ts.getDiagnosticMessage?.(parseInt(code, 10));
    const desc = diag ? diag.message : 'Unknown error';
    return `<tr><td>TS${code}</td><td>${count}</td><td>${desc}</td></tr>`;
  }).join('');

  const extraRows = latest.topExtraErrors.map(({ code, count }) => {
    const ts = require('typescript');
    const diag = ts.getDiagnosticMessage?.(parseInt(code, 10));
    const desc = diag ? diag.message : 'Unknown error';
    return `<tr><td>TS${code}</td><td>${count}</td><td>${desc}</td></tr>`;
  }).join('');

  const html = `<!DOCTYPE html>
<html>
<head>
  <title>Error Distribution Report</title>
  <style>
    body { font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; margin: 20px; background: #f5f5f5; }
    .container { max-width: 1200px; margin: 0 auto; background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
    h1 { color: #333; }
    h2 { color: #666; margin-top: 30px; }
    table { width: 100%; border-collapse: collapse; margin-top: 15px; }
    th, td { padding: 10px; text-align: left; border-bottom: 1px solid #ddd; }
    th { background: #4CAF50; color: white; }
    tr:hover { background: #f5f5f5; }
    .summary { display: flex; gap: 20px; margin-bottom: 20px; }
    .summary-card { flex: 1; padding: 15px; background: #f9f9f9; border-radius: 5px; }
    .summary-card h3 { margin: 0 0 5px 0; color: #666; font-size: 14px; }
    .summary-card .value { font-size: 24px; font-weight: bold; }
    .missing-header { background: #f44336; }
    .extra-header { background: #ff9800; }
  </style>
</head>
<body>
  <div class="container">
    <h1>Error Distribution Report</h1>
    <p>Generated: ${new Date().toLocaleString()}</p>

    <div class="summary">
      <div class="summary-card">
        <h3>Total Missing Errors</h3>
        <div class="value" style="color: #f44336">${latest.totalMissingCount}</div>
      </div>
      <div class="summary-card">
        <h3>Total Extra Errors</h3>
        <div class="value" style="color: #ff9800">${latest.totalExtraCount}</div>
      </div>
    </div>

    <h2>Top 10 Missing Error Codes</h2>
    <table>
      <thead>
        <tr class="missing-header">
          <th>Code</th>
          <th>Count</th>
          <th>Description</th>
        </tr>
      </thead>
      <tbody>
        ${missingRows || '<tr><td colspan="3">No missing errors!</td></tr>'}
      </tbody>
    </table>

    <h2>Top 10 Extra Error Codes</h2>
    <table>
      <thead>
        <tr class="extra-header">
          <th>Code</th>
          <th>Count</th>
          <th>Description</th>
        </tr>
      </thead>
      <tbody>
        ${extraRows || '<tr><td colspan="3">No extra errors!</td></tr>'}
      </tbody>
    </table>
  </div>
</body>
</html>`;

  ensureMetricsDir();
  const htmlPath = join(CONFIG.metricsDir, 'error-distribution.html');
  writeFileSync(htmlPath, html);
  return htmlPath;
}

/**
 * Main CLI interface
 */
async function main() {
  const args = process.argv.slice(2);
  const command = args[0] || 'report';

  switch (command) {
    case 'add': {
      // Add a new snapshot from code counts
      // Usage: add <json-file-with-missing-and-extra-counts>
      const file = args[1];
      if (!file) {
        log('Error: Please specify a JSON file with error counts', colors.red);
        return;
      }

      const filePath = resolve(process.cwd(), file);
      if (!existsSync(filePath)) {
        log(`Error: File not found: ${file}`, colors.red);
        return;
      }

      const data = JSON.parse(readFileSync(filePath, 'utf-8'));
      const snapshot = addSnapshot(
        data.missingCodeCounts || {},
        data.extraCodeCounts || {},
        data.metadata || {}
      );
      log('Error distribution snapshot saved.', colors.green);
      log(`  Missing: ${snapshot.totalMissingCount}`, colors.red);
      log(`  Extra: ${snapshot.totalExtraCount}`, colors.yellow);
      break;
    }

    case 'report': {
      // Display error distribution report
      const snapshots = parseInt(args.find(a => a.startsWith('--snapshots='))?.split('=')[1] || '5', 10);
      const data = loadErrorDistribution();
      displayErrorDistributionReport(data, snapshots);
      break;
    }

    case 'html': {
      // Generate HTML report
      const data = loadErrorDistribution();
      const htmlPath = generateHtmlReport(data);
      log(`\nHTML report generated: ${htmlPath}`, colors.green);
      break;
    }

    case 'analyze': {
      // Run conformance tests and analyze error distribution
      log('Running conformance tests to analyze error distribution...', colors.cyan);
      const { runTests } = await import('./conformance-embedded.mjs');
      const results = await runTests({ maxTests: 500, verbose: false });

      const snapshot = addSnapshot(
        results.missingCodeCounts || {},
        results.extraCodeCounts || {},
        { totalTests: results.totalTests }
      );

      log('\nError Distribution Analysis Complete:', colors.green);
      log(`  Total Tests:    ${results.totalTests}`, colors.dim);
      log(`  Missing Errors: ${snapshot.totalMissingCount}`, colors.red);
      log(`  Extra Errors:   ${snapshot.totalExtraCount}`, colors.yellow);

      displayErrorDistributionReport(loadErrorDistribution(), 5);
      break;
    }

    default:
      log('Error: Unknown command', colors.red);
      log('\nAvailable commands:', colors.dim);
      log('  add <file>                    Add snapshot from JSON file');
      log('  report [--snapshots=N]        Show terminal report');
      log('  html                          Generate HTML report');
      log('  analyze                       Run tests and analyze distribution');
  }
}

main().catch(e => {
  console.error('Error:', e);
  process.exit(1);
});
