#!/usr/bin/env node
/**
 * Find TS2348 (Cannot invoke expression that lacks call signature) errors
 *
 * Usage: node find-ts2348.mjs
 */

import { readFileSync } from 'fs';
import { join } from 'path';

const conformanceOutput = process.argv[2] || './conformance_output.txt';

interface TestResult {
	file: string;
	extra: string[];
	missing: string[];
}

function parseConformanceOutput(content: string): TestResult[] {
	const results: TestResult[] = [];
	const lines = content.split('\n');
	let currentFile: string | null = null;
	let currentResult: TestResult | null = null;

	for (const line of lines) {
		const fileMatch = line.match(/^Testing: (.+)$/);
		if (fileMatch) {
			if (currentResult) {
				results.push(currentResult);
			}
			currentFile = fileMatch[1];
			currentResult = { file: currentFile, extra: [], missing: [] };
			continue;
		}

		if (!currentResult) continue;

		const extraMatch = line.match(/\s*\[Extra Error\] TS2348: (.+)$/);
		if (extraMatch) {
			currentResult.extra.push(extraMatch[1]);
			continue;
		}

		const missingMatch = line.match(/\s*\[Missing Error\] TS2348: (.+)$/);
		if (missingMatch) {
			currentResult.missing.push(missingMatch[1]);
		}
	}

	if (currentResult) {
		results.push(currentResult);
	}

	return results;
}

function main() {
	try {
		const content = readFileSync(conformanceOutput, 'utf-8');
		const results = parseConformanceOutput(content);

		const extraErrors = results.flatMap(r => r.extra);
		const missingErrors = results.flatMap(r => r.missing);

		console.log('='.repeat(60));
		console.log('TS2348 Analysis Report');
		console.log('='.repeat(60));
		console.log(`\nTotal files with TS2348 issues: ${results.filter(r => r.extra.length > 0 || r.missing.length > 0).length}`);
		console.log(`\nExtra TS2348 errors (false positives): ${extraErrors.length}`);
		console.log(`Missing TS2348 errors (false negatives): ${missingErrors.length}`);

		if (extraErrors.length > 0) {
			console.log('\n--- Extra TS2348 Errors (First 20) ---');
			extraErrors.slice(0, 20).forEach((error, i) => {
				console.log(`${i + 1}. ${error}`);
			});
			if (extraErrors.length > 20) {
				console.log(`... and ${extraErrors.length - 20} more`);
			}
		}

		if (missingErrors.length > 0) {
			console.log('\n--- Missing TS2348 Errors (First 20) ---');
			missingErrors.slice(0, 20).forEach((error, i) => {
				console.log(`${i + 1}. ${error}`);
			});
			if (missingErrors.length > 20) {
				console.log(`... and ${missingErrors.length - 20} more`);
			}
		}

		// Categorize by common patterns
		console.log('\n--- Common Patterns ---');
		const extraPatterns = {
			'Function calls': extraErrors.filter(e => e.includes('Cannot invoke')),
			'Method calls': extraErrors.filter(e => e.includes('this.') || e.includes('.')),
			'Constructors': extraErrors.filter(e => e.includes('new')),
		};

		for (const [pattern, count] of Object.entries(extraPatterns)) {
			if (count > 0) {
				console.log(`${pattern}: ${count.length}`);
			}
		}

		console.log('\n--- Files with Most Issues ---');
		const fileCounts = results
			.map(r => ({
				file: r.file,
				total: r.extra.length + r.missing.length
			}))
			.filter(r => r.total > 0)
			.sort((a, b) => b.total - a.total)
			.slice(0, 10);

		fileCounts.forEach(({ file, total }) => {
			console.log(`${total} issues - ${file}`);
		});

		console.log('\n' + '='.repeat(60));

	} catch (error) {
		if (error.code === 'ENOENT') {
			console.error(`Error: Could not find ${conformanceOutput}`);
			console.error('Please run conformance tests first:');
			console.error('  bash run-conformance.sh --max=100');
		} else {
			console.error('Error:', error.message);
		}
		process.exit(1);
	}
}

main();
