#!/usr/bin/env node
/**
 * Find TS2683 ('this' implicitly has type 'any') errors
 * Also tracks TS2571 (Object is of type 'unknown') which is incorrectly emitted instead
 *
 * Usage: node find-ts2683.mjs
 */

import { readFileSync } from 'fs';

const conformanceOutput = process.argv[2] || './conformance_output.txt';

interface TestResult {
	file: string;
	extra2683: string[];
	missing2683: string[];
	extra2571: string[];
	missing2571: string[];
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
			currentResult = {
				file: currentFile,
				extra2683: [],
				missing2683: [],
				extra2571: [],
				missing2571: []
			};
			continue;
		}

		if (!currentResult) continue;

		const extra2683Match = line.match(/\s*\[Extra Error\] TS2683: (.+)$/);
		if (extra2683Match) {
			currentResult.extra2683.push(extra2683Match[1]);
			continue;
		}

		const missing2683Match = line.match(/\s*\[Missing Error\] TS2683: (.+)$/);
		if (missing2683Match) {
			currentResult.missing2683.push(missing2683Match[1]);
			continue;
		}

		const extra2571Match = line.match(/\s*\[Extra Error\] TS2571: (.+)$/);
		if (extra2571Match) {
			currentResult.extra2571.push(extra2571Match[1]);
			continue;
		}

		const missing2571Match = line.match(/\s*\[Missing Error\] TS2571: (.+)$/);
		if (missing2571Match) {
			currentResult.missing2571.push(missing2571Match[1]);
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

		const extra2683 = results.flatMap(r => r.extra2683);
		const missing2683 = results.flatMap(r => r.missing2683);
		const extra2571 = results.flatMap(r => r.extra2571);
		const missing2571 = results.flatMap(r => r.missing2571);

		// Check for "this" in extra TS2571 (these should be TS2683)
		const thisInExtra2571 = extra2571.filter(e =>
			e.toLowerCase().includes('this') ||
			e.includes("'this'")
		);

		console.log('='.repeat(60));
		console.log('TS2683/TS2571 This Type Analysis Report');
		console.log('='.repeat(60));
		console.log(`\nTotal files with this type issues: ${results.filter(r =>
			r.extra2683.length > 0 || r.missing2683.length > 0 ||
			r.extra2571.length > 0 || r.missing2571.length > 0
		).length}`);

		console.log('\n--- TS2683 (this implicitly has type any) ---');
		console.log(`Extra TS2683 errors (false positives): ${extra2683.length}`);
		console.log(`Missing TS2683 errors (should emit but dont): ${missing2683.length}`);

		console.log('\n--- TS2571 (Object is of type unknown) ---');
		console.log(`Extra TS2571 errors: ${extra2571.length}`);
		console.log(`  - Of these, ${thisInExtra2571.length} appear to be "this" related (should be TS2683)`);
		console.log(`Missing TS2571 errors: ${missing2571.length}`);

		if (missing2683.length > 0) {
			console.log('\n--- Missing TS2683 Errors (First 20) ---');
			missing2683.slice(0, 20).forEach((error, i) => {
				console.log(`${i + 1}. ${error}`);
			});
			if (missing2683.length > 20) {
				console.log(`... and ${missing2683.length - 20} more`);
			}
		}

		if (thisInExtra2571.length > 0) {
			console.log('\n--- TS2571 Errors That Should Be TS2683 (First 20) ---');
			thisInExtra2571.slice(0, 20).forEach((error, i) => {
				console.log(`${i + 1}. ${error}`);
			});
			if (thisInExtra2571.length > 20) {
				console.log(`... and ${thisInExtra2571.length - 20} more`);
			}
		}

		// Categorize by context
		console.log('\n--- Context Analysis ---');
		const missingContexts = {
			'Regular functions': missing2683.filter(e =>
				e.includes('function') && !e.includes('class')
			),
			'Methods': missing2683.filter(e =>
				e.includes('class') || e.includes('method')
			),
			'Arrow functions': missing2683.filter(e =>
				e.includes('=>') || e.includes('arrow')
			),
		};

		for (const [context, errors] of Object.entries(missingContexts)) {
			if (errors.length > 0) {
				console.log(`${context}: ${errors.length} missing TS2683`);
			}
		}

		console.log('\n--- Files with Most Issues ---');
		const fileCounts = results
			.map(r => ({
				file: r.file,
				total: r.extra2683.length + r.missing2683.length +
				       r.extra2571.length + r.missing2571.length
			}))
			.filter(r => r.total > 0)
			.sort((a, b) => b.total - a.total)
			.slice(0, 10);

		fileCounts.forEach(({ file, total }) => {
			console.log(`${total} issues - ${file}`);
		});

		console.log('\n--- Analysis ---');
		if (missing2683.length > 0 && thisInExtra2571.length > 0) {
			console.log('ISSUE DETECTED: TS2683 is not being emitted for "this" in non-method functions.');
			console.log('Instead, "this" is being typed as "unknown" and emitting TS2571 on property access.');
			console.log(`\nRecommendation: Check current_this_type() in thin_checker.rs (around line 629)`);
		} else if (missing2683.length > 0) {
			console.log('TS2683 is missing in some contexts where it should be emitted.');
		} else if (thisInExtra2571.length > 0) {
			console.log('TS2571 is being emitted for "this" when TS2683 should be used instead.');
		} else {
			console.log('This type handling appears to be working correctly.');
		}

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
