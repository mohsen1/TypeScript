#!/usr/bin/env node
/**
 * ask-gemini.mjs
 *
 * Uses yek to serialize codebase content and sends it to Gemini 3 Pro
 * with 1 million token context for answering complex codebase questions.
 *
 * Usage:
 *   ./scripts/ask-gemini.mjs "How does the type checker work?"
 *   ./scripts/ask-gemini.mjs --tokens=500k "Explain the binder logic"
 *   ./scripts/ask-gemini.mjs --dirs="src/compiler/" "What is the emit algorithm?"
 *   ./scripts/ask-gemini.mjs --review wasm/src/emitter.rs  # Code review mode
 *
 * Environment:
 *   GOOGLE_API_KEY - Required. Fetched from .env.local or environment.
 */

import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { createInterface } from "node:readline";
import { fileURLToPath } from "node:url";
import { Command } from "commander";
import chalk from "chalk";
import dotenv from "dotenv";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const REPO_ROOT = path.resolve(__dirname, "..");

// Load environment variables
dotenv.config({ path: path.join(REPO_ROOT, ".env.local") });

// Gemini 3 Pro Preview configuration
const DEFAULT_MODEL = "gemini-3-pro-preview";
const API_URL = "https://aiplatform.googleapis.com/v1/publishers/google/models";

// Default configuration - only wasm directory (Rust migration focus)
// Using 800k to stay safely under Gemini's 1M token limit (yek uses OpenAI tokenizer)
const DEFAULT_TOKENS = "800k";
const DEFAULT_DIRS = ["wasm/"];

// Review mode configuration - maximize context for thorough reviews
const REVIEW_TOKENS = "800k";
const REVIEW_DIRS = ["wasm/"];

// Code review system prompt for Rust migration
const CODE_REVIEW_PROMPT = `You are **RustReviewer**, an uncompromising senior systems engineer with 15+ years of experience in compiler development and Rust. You are reviewing code for the TypeScript-to-Rust migration project. Your reviews are **brutally honest**, **technically precise**, and **actionable**.

### Your Expertise
- Deep knowledge of the TypeScript compiler internals (scanner, parser, binder, checker, emitter)
- Expert-level Rust: ownership, lifetimes, zero-cost abstractions, unsafe code auditing
- Performance-critical systems programming
- Memory safety and undefined behavior detection
- Idiomatic Rust patterns vs. "translated from another language" anti-patterns

### Review Standards

**You reject code that:**
1. **Transliterates instead of translates** — Don't just port line-by-line from TypeScript. Rust has different idioms. Use \`Option\`/\`Result\` properly, not \`.is_none()\` checks everywhere. Use pattern matching. Use iterators.
2. **Ignores ownership** — Arena allocations are fine, but don't create a web of indices that's just pointers with extra steps and no safety guarantees.
3. **Uses \`unsafe\` without justification** — Every \`unsafe\` block needs a \`// SAFETY:\` comment explaining the invariant. No exceptions.
4. **Has \`TODO\` without issue links** — Every TODO must reference a tracking issue or be removed.
5. **Lacks error handling** — No silent failures. No \`.unwrap()\` in library code. Propagate errors properly.
6. **Has dead code** — No commented-out code. No unused functions. No \`#[allow(dead_code)]\` without explanation.
7. **Misses test coverage** — If you add a code path, you add a test. Roundtrip tests are the bare minimum.

### Review Format

For each issue found, provide:

\`\`\`
❌ [SEVERITY] file:line — Brief description

Problem: What's wrong and why it matters
Evidence: The specific code snippet
Fix: Concrete solution, not vague advice
\`\`\`

Severity levels:
- **BLOCKER** — Correctness bug, UB, memory safety issue. Cannot merge.
- **CRITICAL** — Performance regression, API misuse, missing error handling. Should not merge.
- **MAJOR** — Non-idiomatic code, maintainability issue, missing tests. Fix before merge.
- **MINOR** — Style, naming, documentation. Note for follow-up.

### What You Look For

1. **Correctness against TypeScript reference**
   - Does the Rust implementation match the TypeScript behavior exactly?
   - Are edge cases handled (empty inputs, unicode, malformed input)?

2. **Performance**
   - Unnecessary allocations? Use \`&str\` not \`String\` where possible.
   - Unnecessary clones? Consider borrowing or \`Cow<'_, str>\`.
   - Hot path allocations? Consider arena allocation or \`SmallVec\`.

3. **Memory Safety**
   - Are arena indices validated before use?
   - Can indices become stale/dangling?
   - Is there any \`transmute\` that could produce invalid values?

4. **API Design**
   - Is the API impossible to misuse?
   - Are invariants enforced at compile time, not runtime?
   - Is the API consistent with the rest of the codebase?

5. **Rust Idioms**
   - Use \`if let\` / \`match\` instead of \`.is_some()\` + \`.unwrap()\`
   - Use \`?\` operator for error propagation
   - Use iterators and combinators, not manual index loops
   - Prefer \`impl Trait\` over \`Box<dyn Trait>\` when possible

### Your Personality

- **Direct**: "This is wrong" not "Perhaps we could consider..."
- **Specific**: Point to exact lines and provide exact fixes
- **Educational**: Explain *why* something is wrong, cite Rustonomicon if needed
- **Unimpressed by volume**: 1000 lines of code means 1000 opportunities for bugs
- **Zero tolerance for regression**: If it worked in TypeScript, it must work in Rust

### Instructions

When given code to review:
1. Read the entire file/diff carefully
2. Cross-reference with TypeScript implementation in \`src/compiler/\` if behavior is unclear
3. List ALL issues, not just the first few
4. Prioritize blockers and critical issues at the top
5. End with a summary: "X blockers, Y critical, Z major issues. [APPROVED/CHANGES REQUESTED]"

**Remember: You are the last line of defense before this code ships. Be thorough. Be harsh. Be helpful.**`;

// Colors for terminal output
const colors = {
  reset: "\x1b[0m",
  bright: "\x1b[1m",
  dim: "\x1b[2m",
  red: "\x1b[31m",
  green: "\x1b[32m",
  yellow: "\x1b[33m",
  blue: "\x1b[34m",
  cyan: "\x1b[36m",
};

function log(message, color = colors.reset) {
  console.error(`${color}${message}${colors.reset}`);
}

async function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

async function fetchWithRetry(url, options, maxRetries = 5) {
  let lastError;

  for (let attempt = 0; attempt <= maxRetries; attempt++) {
    try {
      const response = await fetch(url, options);

      // If rate limited, retry with exponential backoff
      if (response.status === 429) {
        if (attempt === maxRetries) {
          throw new Error(`Rate limited after ${maxRetries} retries`);
        }

        const retryAfter = response.headers.get('retry-after');
        const waitTime = retryAfter ? parseInt(retryAfter) * 1000 : Math.pow(2, attempt) * 1000;

        log(`\n${colors.yellow}Rate limited. Waiting ${Math.round(waitTime/1000)}s before retry ${attempt + 1}/${maxRetries}...${colors.reset}`, colors.yellow);
        await sleep(waitTime);
        continue;
      }

      // If other error, try to get error message from response
      if (!response.ok) {
        const errorText = await response.text();
        let errorMessage = `Gemini API error (${response.status})`;

        try {
          const errorJson = JSON.parse(errorText);
          if (errorJson.error?.message) {
            errorMessage += `: ${errorJson.error.message}`;
          }
        } catch {
          // If not JSON, use text response
          if (errorText) {
            errorMessage += `: ${errorText}`;
          }
        }

        throw new Error(errorMessage);
      }

      return response;
    } catch (error) {
      lastError = error;

      // Only retry on network errors or rate limits
      if (error.message.includes('Rate limited') || error.message.includes('fetch failed')) {
        if (attempt < maxRetries) {
          const waitTime = Math.pow(2, attempt) * 1000;
          log(`\n${colors.yellow}Request failed. Retrying in ${Math.round(waitTime/1000)}s... (attempt ${attempt + 1}/${maxRetries})${colors.reset}`, colors.yellow);
          await sleep(waitTime);
          continue;
        }
      }

      throw error;
    }
  }

  throw lastError;
}

function loadEnvFile(filePath) {
  if (!fs.existsSync(filePath)) return {};

  const content = fs.readFileSync(filePath, "utf8");
  const env = {};

  for (const line of content.split("\n")) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith("#")) continue;

    const eqIndex = trimmed.indexOf("=");
    if (eqIndex === -1) continue;

    const key = trimmed.slice(0, eqIndex);
    let value = trimmed.slice(eqIndex + 1);

    // Remove surrounding quotes
    if (
      (value.startsWith('"') && value.endsWith('"')) ||
      (value.startsWith("'") && value.endsWith("'"))
    ) {
      value = value.slice(1, -1);
    }

    env[key] = value;
  }

  return env;
}

function getApiKey() {
  // Check environment first (Vertex AI Express API key)
  if (process.env.GCP_VERTEX_EXPRESS_API_KEY) {
    return process.env.GCP_VERTEX_EXPRESS_API_KEY;
  }
  if (process.env.GOOGLE_API_KEY) {
    return process.env.GOOGLE_API_KEY;
  }
  if (process.env.GEMINI_API_KEY) {
    return process.env.GEMINI_API_KEY;
  }

  // Load from .env.local
  const envLocal = loadEnvFile(path.join(REPO_ROOT, ".env.local"));
  if (envLocal.GCP_VERTEX_EXPRESS_API_KEY) {
    return envLocal.GCP_VERTEX_EXPRESS_API_KEY;
  }
  if (envLocal.GOOGLE_API_KEY) {
    return envLocal.GOOGLE_API_KEY;
  }
  if (envLocal.GEMINI_API_KEY) {
    return envLocal.GEMINI_API_KEY;
  }

  // Try .env as fallback
  const envFile = loadEnvFile(path.join(REPO_ROOT, ".env"));
  if (envFile.GCP_VERTEX_EXPRESS_API_KEY) {
    return envFile.GCP_VERTEX_EXPRESS_API_KEY;
  }
  if (envFile.GOOGLE_API_KEY) {
    return envFile.GOOGLE_API_KEY;
  }
  if (envFile.GEMINI_API_KEY) {
    return envFile.GEMINI_API_KEY;
  }

  return null;
}

function parseArgs(args) {
  const config = {
    tokens: DEFAULT_TOKENS,
    dirs: DEFAULT_DIRS,
    question: null,
    interactive: false,
    stream: true,
    review: false,
    reviewFiles: [],
  };

  for (const arg of args) {
    if (arg.startsWith("--tokens=")) {
      config.tokens = arg.slice("--tokens=".length);
    } else if (arg.startsWith("--dirs=")) {
      config.dirs = arg
        .slice("--dirs=".length)
        .split(",")
        .map((d) => d.trim());
    } else if (arg === "--no-stream") {
      config.stream = false;
    } else if (arg === "-i" || arg === "--interactive") {
      config.interactive = true;
    } else if (arg === "--review" || arg === "-r") {
      config.review = true;
      config.tokens = REVIEW_TOKENS;
      config.dirs = REVIEW_DIRS;
    } else if (arg === "--help" || arg === "-h") {
      printUsage();
      process.exit(0);
    } else if (!arg.startsWith("-")) {
      if (config.review) {
        // In review mode, non-flag args are files to review
        config.reviewFiles.push(arg);
      } else {
        config.question = arg;
      }
    }
  }

  return config;
}

function printUsage() {
  console.log(`
${colors.bright}ask-gemini.js${colors.reset} - Ask Gemini 3 Pro questions about TypeScript compiler codebase

${colors.cyan}USAGE:${colors.reset}
  ./scripts/ask-gemini.js [options] "Your question here"
  ./scripts/ask-gemini.js -i                              # Interactive mode
  ./scripts/ask-gemini.js --review wasm/src/emitter.rs    # Code review mode

${colors.cyan}OPTIONS:${colors.reset}
  --tokens=<size>     Token limit for yek (default: 1000k, review: 1000k)
  --dirs=<dirs>       Comma-separated directories to include
  --no-stream         Disable streaming output
  -i, --interactive   Interactive mode - ask multiple questions
  -r, --review        Code review mode for Rust migration (uses maximum context)
  -h, --help          Show this help message

${colors.cyan}EXAMPLES:${colors.reset}
  ./scripts/ask-gemini.js "How does the type checker work?"
  ./scripts/ask-gemini.js --tokens=500k "Explain the binder logic"
  ./scripts/ask-gemini.js --dirs="src/compiler/" "What is the emit algorithm?"
  ./scripts/ask-gemini.js --review wasm/src/scanner.rs
  ./scripts/ask-gemini.js --review wasm/src/emitter.rs wasm/src/types.rs
  ./scripts/ask-gemini.js -i

${colors.cyan}CODE REVIEW MODE:${colors.reset}
  The --review flag activates Rust migration code review mode:
  - Uses a specialized system prompt for reviewing Rust ports of TypeScript code
  - Maximizes context (1000k tokens) to include both Rust and TypeScript sources
  - Provides severity-rated feedback (BLOCKER, CRITICAL, MAJOR, MINOR)
  - Cross-references against TypeScript implementation for correctness

${colors.cyan}ENVIRONMENT:${colors.reset}
  GCP_VERTEX_EXPRESS_API_KEY  API key from Vertex AI Express Mode
  GOOGLE_API_KEY              Fallback API key (loaded from .env.local or environment)
`);
}

function runYek(tokens, dirs) {
  const yekArgs = [`--tokens=${tokens}`, ...dirs];
  const cmd = `yek ${yekArgs.join(" ")}`;

  log(`\n${colors.dim}Running: ${cmd}${colors.reset}`, colors.dim);

  try {
    const output = execFileSync("yek", yekArgs, {
      cwd: REPO_ROOT,
      encoding: "utf8",
      maxBuffer: 100 * 1024 * 1024, // 100MB buffer
    });
    return output;
  } catch (error) {
    if (error.status === 127) {
      log("\nError: yek is not installed. Install it with: cargo install yek", colors.red);
      process.exit(1);
    }
    log(`\n${colors.red}Error: yek command failed with exit code ${error.status ?? "unknown"}${colors.reset}`, colors.red);
    if (error.stderr) {
      log(`${colors.yellow}stderr:${colors.reset}\n${error.stderr}`, colors.yellow);
    }
    if (error.stdout) {
      log(`${colors.yellow}stdout:${colors.reset}\n${error.stdout}`, colors.yellow);
    }
    process.exit(1);
  }
}

async function askGeminiStream(apiKey, codebaseContext, question, systemPrompt = null) {
  const url = `${API_URL}/${DEFAULT_MODEL}:streamGenerateContent?key=${apiKey}&alt=sse`;

  // Read architecture doc dynamically
  const architectureDoc = fs.readFileSync(path.join(REPO_ROOT, "wasm/specs/WASM_ARCHITECTURE.md"), "utf8");

  const defaultSystemPrompt = `You are an expert systems engineer working on **tsc-rust**: a high-performance Rust/WASM port of the TypeScript compiler designed to beat TypeScript-Go in speed.

## Project Architecture

${architectureDoc}

## When Answering

1. **Performance first** — Prefer arena allocation, avoid heap allocations in hot paths, use \`&str\` not \`String\`
2. **Match TypeScript exactly** — The Rust code must produce identical output to \`src/compiler/\`
3. **Use the solver model** — Types are sets. Subtyping is subset. Use TypeId for equality.
4. **Reference specific code** — Quote file paths and line numbers
5. **Be concise** — No fluff. Direct answers with code examples.`;

  const requestBody = {
    contents: [
      {
        role: "user",
        parts: [
          {
            text: `<codebase_context>
${codebaseContext}
</codebase_context>

<question>
${question}
</question>`,
          },
        ],
      },
    ],
    systemInstruction: {
      parts: [{ text: systemPrompt || defaultSystemPrompt }],
    },
    generationConfig: {
      temperature: 0.7,
      maxOutputTokens: 65536,
      thinkingConfig: {
        thinkingBudget: 32768,
      },
    },
  };

  const response = await fetchWithRetry(url, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(requestBody),
  });

  if (!response.ok) {
    const errorText = await response.text();
    throw new Error(`API error (${response.status}): ${errorText}`);
  }

  if (!response.body) {
    throw new Error("No response body from API");
  }

  const reader = response.body.getReader();
  const decoder = new TextDecoder();
  let buffer = "";
  let fullResponse = "";

  process.stdout.write(`\n${colors.green}${colors.bright}Answer:${colors.reset}\n\n`);

  while (true) {
    const { done, value } = await reader.read();
    if (done) break;

    buffer += decoder.decode(value, { stream: true });
    const lines = buffer.split("\n");
    buffer = lines.pop() || "";

    for (const line of lines) {
      const trimmed = line.trim();
      if (!trimmed || trimmed === "[DONE]") continue;

      const jsonStr = trimmed.startsWith("data: ") ? trimmed.slice(6) : trimmed;
      if (jsonStr === "[DONE]") continue;

      try {
        const data = JSON.parse(jsonStr);
        const text = data?.candidates?.[0]?.content?.parts?.[0]?.text;
        if (text) {
          process.stdout.write(text);
          fullResponse += text;
        }
      } catch {
        // Skip malformed JSON chunks
      }
    }
  }

  console.log("\n");
  return fullResponse;
}

async function askGemini(apiKey, codebaseContext, question, systemPrompt = null) {
  const url = `${API_URL}/${DEFAULT_MODEL}:generateContent?key=${apiKey}`;

  // Read architecture doc dynamically
  const architectureDoc = fs.readFileSync(path.join(REPO_ROOT, "wasm/specs/WASM_ARCHITECTURE.md"), "utf8");

  const defaultSystemPrompt = `You are an expert systems engineer working on **tsc-rust**: a high-performance Rust/WASM port of the TypeScript compiler designed to beat TypeScript-Go in speed.

## Project Architecture

${architectureDoc}

## When Answering

1. **Performance first** — Prefer arena allocation, avoid heap allocations in hot paths, use \`&str\` not \`String\`
2. **Match TypeScript exactly** — The Rust code must produce identical output to \`src/compiler/\`
3. **Use the solver model** — Types are sets. Subtyping is subset. Use TypeId for equality.
4. **Reference specific code** — Quote file paths and line numbers
5. **Be concise** — No fluff. Direct answers with code examples.`;

  const requestBody = {
    contents: [
      {
        role: "user",
        parts: [
          {
            text: `<codebase_context>
${codebaseContext}
</codebase_context>

<question>
${question}
</question>`,
          },
        ],
      },
    ],
    systemInstruction: {
      parts: [{ text: systemPrompt || defaultSystemPrompt }],
    },
    generationConfig: {
      temperature: 0.7,
      maxOutputTokens: 65536,
      thinkingConfig: {
        thinkingBudget: 32768,
      },
    },
  };

  const response = await fetchWithRetry(url, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "x-goog-api-key": apiKey,
    },
    body: JSON.stringify(requestBody),
  });

  const data = await response.json();
  const text = data?.candidates?.[0]?.content?.parts?.[0]?.text;

  if (!text) {
    throw new Error("No response text from Gemini");
  }

  console.log(`\n${colors.green}${colors.bright}Answer:${colors.reset}\n`);
  console.log(text);
  console.log();

  return text;
}

function readFilesForReview(filePaths) {
  const fileContents = [];

  for (const filePath of filePaths) {
    const absolutePath = path.isAbsolute(filePath)
      ? filePath
      : path.join(REPO_ROOT, filePath);

    if (!fs.existsSync(absolutePath)) {
      log(`Warning: File not found: ${filePath}`, colors.yellow);
      continue;
    }

    const content = fs.readFileSync(absolutePath, "utf8");
    const relativePath = path.relative(REPO_ROOT, absolutePath);
    fileContents.push(`\n=== FILE: ${relativePath} ===\n${content}`);
  }

  return fileContents.join("\n");
}

async function reviewMode(apiKey, config) {
  log(`\n${colors.cyan}${colors.bright}Code Review Mode (Rust Migration)${colors.reset}`, colors.cyan);

  // Read specific files to review
  let filesToReview = "";
  if (config.reviewFiles.length > 0) {
    log(`${colors.dim}Reading files to review...${colors.reset}`, colors.dim);
    filesToReview = readFilesForReview(config.reviewFiles);
    if (!filesToReview) {
      log("Error: No valid files to review.", colors.red);
      process.exit(1);
    }
    log(
      `${colors.green}✓ Loaded ${config.reviewFiles.length} file(s) for review${colors.reset}`,
      colors.green
    );
  }

  // Load codebase context for reference
  log(`${colors.dim}Loading codebase context for cross-reference...${colors.reset}`, colors.dim);
  const codebaseContext = runYek(config.tokens, config.dirs);
  const contextSize = codebaseContext.length;
  const estimatedTokens = Math.round(contextSize / 4);

  log(
    `${colors.green}✓ Loaded ~${estimatedTokens.toLocaleString()} tokens of reference context${colors.reset}`,
    colors.green
  );

  // Build the review request
  const reviewRequest = config.reviewFiles.length > 0
    ? `Please review the following Rust code for the TypeScript-to-Rust migration:

<files_to_review>
${filesToReview}
</files_to_review>

Cross-reference against the TypeScript implementation in the codebase context to verify correctness.`
    : `Please review all Rust code in the wasm/src/ directory for the TypeScript-to-Rust migration. Focus on:
1. Correctness against the TypeScript reference implementation
2. Memory safety and proper error handling
3. Idiomatic Rust patterns
4. Performance considerations`;

  log(`${colors.dim}Submitting review request...${colors.reset}`, colors.dim);

  try {
    if (config.stream) {
      await askGeminiStream(apiKey, codebaseContext, reviewRequest, CODE_REVIEW_PROMPT);
    } else {
      await askGemini(apiKey, codebaseContext, reviewRequest, CODE_REVIEW_PROMPT);
    }
  } catch (error) {
    log(`\nError: ${error.message}`, colors.red);
    process.exit(1);
  }
}

async function interactiveMode(apiKey, config) {
  log(`\n${colors.cyan}${colors.bright}Interactive Mode${colors.reset}`, colors.cyan);
  log(`${colors.dim}Loading codebase context...${colors.reset}`, colors.dim);

  const codebaseContext = runYek(config.tokens, config.dirs);
  const contextSize = codebaseContext.length;
  const estimatedTokens = Math.round(contextSize / 4);

  log(
    `${colors.green}✓ Loaded ~${estimatedTokens.toLocaleString()} tokens of context${colors.reset}`,
    colors.green
  );
  log(`${colors.dim}Type your questions (Ctrl+C to exit)${colors.reset}\n`, colors.dim);

  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
  });

  const askQuestion = () => {
    rl.question(`${colors.blue}❯ ${colors.reset}`, async (question) => {
      if (!question.trim()) {
        askQuestion();
        return;
      }

      try {
        if (config.stream) {
          await askGeminiStream(apiKey, codebaseContext, question);
        } else {
          await askGemini(apiKey, codebaseContext, question);
        }
      } catch (error) {
        log(`Error: ${error.message}`, colors.red);
      }

      askQuestion();
    });
  };

  rl.on("close", () => {
    console.log("\nGoodbye!");
    process.exit(0);
  });

  askQuestion();
}

async function main() {
  const args = process.argv.slice(2);
  const config = parseArgs(args);

  // Check for API key
  const apiKey = getApiKey();
  if (!apiKey) {
    log("\nError: GCP_VERTEX_EXPRESS_API_KEY not found.", colors.red);
    log("Set GCP_VERTEX_EXPRESS_API_KEY in .env.local or as environment variable", colors.yellow);
    process.exit(1);
  }

  // Review mode
  if (config.review) {
    await reviewMode(apiKey, config);
    return;
  }

  // Interactive mode
  if (config.interactive) {
    await interactiveMode(apiKey, config);
    return;
  }

  // Single question mode
  if (!config.question) {
    log("\nError: No question provided.", colors.red);
    printUsage();
    process.exit(1);
  }

  log(`${colors.cyan}${colors.bright}Asking Gemini 3 Pro...${colors.reset}`, colors.cyan);
  log(`${colors.dim}Loading codebase context...${colors.reset}`, colors.dim);

  const codebaseContext = runYek(config.tokens, config.dirs);
  const contextSize = codebaseContext.length;
  const estimatedTokens = Math.round(contextSize / 4);

  log(
    `${colors.green}✓ Loaded ~${estimatedTokens.toLocaleString()} tokens of context${colors.reset}`,
    colors.green
  );
  log(`${colors.dim}Question: ${config.question}${colors.reset}`, colors.dim);

  try {
    if (config.stream) {
      await askGeminiStream(apiKey, codebaseContext, config.question);
    } else {
      await askGemini(apiKey, codebaseContext, config.question);
    }
  } catch (error) {
    log(`\nError: ${error.message}`, colors.red);
    process.exit(1);
  }
}

main().catch((error) => {
  log(`\nUnexpected error: ${error.message}`, colors.red);
  process.exit(1);
});
