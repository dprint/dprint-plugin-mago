/**
 * Reconciles this plugin with a new Mago release using OpenAI Codex, in two
 * stages that each run Codex with a different model:
 *
 *   1. a Codex agentic session edits the source and runs `cargo test` until
 *      green (fixing breakage and/or wiring up new `FormatSettings`), then
 *   2. a SEPARATE reviewer model reviews the result with Codex, so it can
 *      investigate (read the upstream mago-formatter source, grep the repo). It
 *      is instructed to review only and not modify anything. If it finds
 *      blocking issues they are fed back to the stage-1 Codex for another pass,
 *      bounded by `REVIEW_MAX_ROUNDS`. If it still isn't approved, this throws
 *      so the workflow fails and nothing is published.
 *
 * Two situations call this:
 *   - a patch bump that failed to build/test (fix the breakage), and
 *   - any minor bump (review for new/renamed/removed settings even when it
 *     still compiles).
 *
 * Codex must NOT commit or push -- `update.ts` captures the working tree
 * changes into the existing Mago bump commit afterwards.
 */
import { $ } from "automation";

export interface AiFixOptions {
  /** true for a patch bump, false for a minor/major bump. */
  isPatchBump: boolean;
  /** Mago version currently in Cargo.toml (before the bump). */
  fromVersion: string;
  /** Mago version being upgraded to. */
  toVersion: string;
  /** Whether the checks (test + clippy) already passed with the bump applied. */
  checksPassed: boolean;
  /** Combined output of the failing checks (empty when `checksPassed`). */
  checkOutput: string;
}

/** Max number of times the reviewer may send changes back to Codex. */
const REVIEW_MAX_ROUNDS = 2;

export async function aiFixMagoUpdate(options: AiFixOptions): Promise<void> {
  const apiKey = requireApiKey();
  await ensureCodexInstalled();
  await codexLogin(apiKey);

  // stage 1: let Codex reconcile the update.
  await runCodex(buildFixPrompt(options));

  // stage 2: independent second-model review, with a bounded refix loop.
  for (let round = 1;; round++) {
    const review = await reviewChanges(options);
    logReview(review);
    if (review.approved) {
      return;
    }
    if (round > REVIEW_MAX_ROUNDS) {
      const blocking = review.issues.filter((i) => i.severity === "blocking");
      throw new Error(
        `AI reviewer did not approve after ${REVIEW_MAX_ROUNDS} refix round(s). Blocking issues:\n` +
          blocking.map((i) => `  - ${i.description}`).join("\n"),
      );
    }
    $.logStep(`Reviewer requested changes — Codex refix round ${round}...`);
    await runCodex(buildRefixPrompt(review));
  }
}

// stage 1: Codex ---------------------------------------------------------------

async function runCodex(prompt: string): Promise<void> {
  const args = ["exec", "--dangerously-bypass-approvals-and-sandbox", "--skip-git-repo-check"];
  const model = Deno.env.get("CODEX_MODEL");
  if (model) {
    args.push("--model", model);
  }
  args.push(prompt);

  $.logStep("Running Codex...");
  await $`codex ${args}`;
}

function buildFixPrompt(options: AiFixOptions): string {
  const { isPatchBump, fromVersion, toVersion, checksPassed, checkOutput } = options;

  const situation = checksPassed
    ? `Mago was upgraded from ${fromVersion} to ${toVersion} (a ${
      isPatchBump ? "patch" : "minor"
    } bump). The project already compiles and the checks pass, but a new Mago version may have ADDED, RENAMED, or REMOVED formatting settings that should be surfaced by this plugin.`
    : `Mago was upgraded from ${fromVersion} to ${toVersion} and the checks fail (the project no longer builds, or \`cargo test\` or \`cargo clippy\` fails). This is almost always because Mago's public API (usually its \`FormatSettings\` struct or related enums) changed.`;

  const failureOutput = checkOutput.trim().length > 0
    ? [
      ``,
      `The checks currently fail with the output below. Use it as your starting point instead of re-running the checks just to rediscover the errors:`,
      "```",
      truncateHead(checkOutput.trim()),
      "```",
    ]
    : [];

  return [
    `You are updating the "dprint-plugin-mago" Rust crate, a dprint plugin that wraps the \`mago-formatter\` crate to format PHP.`,
    ``,
    situation,
    ...failureOutput,
    ``,
    `Your goal: make the checks pass AND keep this plugin's configuration surface in sync with mago-formatter's \`FormatSettings\`. Do NOT commit or push; only edit files in the working tree.`,
    ``,
    describeWiring(),
    ``,
    `To see exactly what changed in mago-formatter, inspect the actual dependency source that cargo already downloaded, e.g.:`,
    `  find ~/.cargo/registry/src -type d -name 'mago-formatter-*'`,
    `then read its \`src/settings.rs\` (the \`FormatSettings\` struct) and any changed enums. Compare that against \`build_format_settings\` in this repo.`,
    ``,
    `Rules:`,
    `1. If a setting was RENAMED or REMOVED in FormatSettings, update the mapping in \`src/format_text.rs\` (and remove/rename the corresponding plugin config in the other files if it no longer exists upstream).`,
    `2. If a setting was ADDED in FormatSettings, expose it as a new plugin config option across ALL of: \`configuration.rs\`, \`resolve_config.rs\`, \`format_text.rs\`, \`deployment/schema.json\`, and \`README.md\`. Match the existing naming conventions (Rust snake_case fields, camelCase dprint keys).`,
    `3. Preserve the existing code style. Keep non-test code above test modules. New comments start lowercase unless multiple sentences.`,
    `4. When done, BOTH of these must pass (CI denies clippy warnings, so a clippy warning is a hard failure) — iterate until both are clean:`,
    `     cargo test`,
    `     cargo clippy --all-targets --all-features -- -D warnings`,
    `5. Do not change the plugin's own version in Cargo.toml, do not run git commit, and do not push.`,
  ].join("\n");
}

function buildRefixPrompt(review: ReviewResult): string {
  return [
    `An independent reviewer examined your changes to dprint-plugin-mago and found issues that must be fixed. Address every blocking issue below, keeping \`cargo test\` green. Do not commit or push.`,
    ``,
    `Reviewer summary: ${review.summary}`,
    ``,
    `Issues:`,
    ...review.issues.map((i) => `  - [${i.severity}] ${i.description}`),
    ``,
    describeWiring(),
  ].join("\n");
}

function describeWiring(): string {
  return [
    `How the plugin is wired (keep all of these consistent with each other):`,
    `- \`src/format_text.rs\` -> \`build_format_settings\` maps this plugin's \`Configuration\` onto mago-formatter's \`FormatSettings\` and its enums (\`BraceStyle\`, \`MethodChainBreakingStyle\`, \`NullTypeHint\`, \`EndOfLine\`).`,
    `- \`src/configuration/configuration.rs\` -> the plugin's own \`Configuration\` struct and enums.`,
    `- \`src/configuration/resolve_config.rs\` -> reads each dprint config key (camelCase) into \`Configuration\`.`,
    `- \`deployment/schema.json\` -> the JSON schema of config options shown to users.`,
    `- \`README.md\` -> documents each config option.`,
  ].join("\n");
}

// keep the tail of long check output -- the errors and the "could not compile"
// summary are at the end, which is the most useful part for the AI.
function truncateHead(text: string, max = 20_000): string {
  return text.length > max ? `... (truncated)\n${text.slice(text.length - max)}` : text;
}

// stage 2: independent reviewer ------------------------------------------------

interface ReviewResult {
  approved: boolean;
  summary: string;
  issues: ReviewIssue[];
}

interface ReviewIssue {
  severity: "blocking" | "nit";
  description: string;
}

// The reviewer runs Codex too so it can actually investigate (read the
// mago-formatter source cargo downloaded, grep the repo, verify field names
// against the real upstream API). Its verdict is captured as JSON via
// --output-last-message.
//
// NOTE: we deliberately do NOT use `--sandbox read-only` here. That makes Codex
// wrap commands in bubblewrap, which cannot set up its network namespace on the
// GitHub Actions runner ("bwrap: loopback: Failed RTM_NEWADDR: Operation not
// permitted"), so every command the reviewer runs fails before executing. We
// use the same no-sandbox mode as the fixer and enforce read-only via the
// prompt instead. (CI is itself a throwaway VM, and the final check gate in
// update.ts re-verifies whatever ends up in the working tree.)
async function reviewChanges(options: AiFixOptions): Promise<ReviewResult> {
  // default to a different model than the Codex fixer so the review is a
  // genuinely independent second opinion.
  const model = Deno.env.get("REVIEW_MODEL") ?? "gpt-5.6-sol";

  // record intent-to-add so any files Codex created also show in `git diff`.
  await $`git add -N .`.quiet();

  const outputFile = await Deno.makeTempFile({ prefix: "codex-review-", suffix: ".json" });
  try {
    $.logStep(`Reviewing changes with Codex (${model})...`);
    const args = [
      "exec",
      "--skip-git-repo-check",
      "--dangerously-bypass-approvals-and-sandbox",
      "--model",
      model,
      "--output-last-message",
      outputFile,
      buildReviewPrompt(options),
    ];
    await $`codex ${args}`;
    return parseReview(await Deno.readTextFile(outputFile));
  } finally {
    await Deno.remove(outputFile).catch(() => {});
  }
}

function buildReviewPrompt(options: AiFixOptions): string {
  return [
    `You are an independent reviewer for dprint-plugin-mago, a dprint plugin that wraps the mago-formatter crate to format PHP. Mago was just upgraded from ${options.fromVersion} to ${options.toVersion} and another AI edited this plugin to reconcile it. Review the UNCOMMITTED working-tree changes.`,
    `IMPORTANT: this is REVIEW ONLY. Read any files and run read-only commands (git diff, cat, grep, find, etc.) to investigate, but you MUST NOT edit, create, or delete any files, and MUST NOT run git commit or git push. Leave the working tree exactly as you found it.`,
    ``,
    `Investigate as needed:`,
    `- Run \`git --no-pager diff\` (and \`git status\`) to see exactly what changed, including any new files.`,
    `- Verify against the REAL mago-formatter ${options.toVersion} API by reading the source cargo downloaded:`,
    `    find ~/.cargo/registry/src -type d -name 'mago-formatter-*'`,
    `  then read its \`src/settings.rs\` (the \`FormatSettings\` struct) and any changed enums.`,
    `- Read the plugin files to confirm they are consistent with each other and with upstream.`,
    ``,
    describeWiring(),
    ``,
    `Verify specifically:`,
    `- Every field assigned in \`build_format_settings\` still exists in mago-formatter's \`FormatSettings\` with that exact name (catch removed/renamed fields).`,
    `- Any FormatSettings field newly added upstream is exposed through ALL layers: configuration.rs, resolve_config.rs, format_text.rs, deployment/schema.json, and README.md. A partial addition is a blocking issue.`,
    `- Naming conventions are consistent (Rust snake_case fields, camelCase dprint keys, matching schema/README).`,
    `- No obvious correctness bugs, and code style matches the surrounding code.`,
    ``,
    `When done, your FINAL message must be ONLY a JSON object (no markdown code fences, no extra prose) of exactly this shape:`,
    `{"approved": true|false, "summary": "one sentence", "issues": [{"severity": "blocking"|"nit", "description": "..."}]}`,
    `Set approved=false if there is any blocking issue; nits alone do not block.`,
  ].join("\n");
}

function parseReview(message: string): ReviewResult {
  const review = JSON.parse(extractJsonObject(message)) as ReviewResult;
  if (typeof review.approved !== "boolean" || !Array.isArray(review.issues)) {
    throw new Error(`Codex review returned malformed JSON:\n${message}`);
  }
  return review;
}

// Codex is instructed to emit only JSON, but tolerate an accidental code fence
// or surrounding prose by extracting the outermost { ... } object.
function extractJsonObject(text: string): string {
  const start = text.indexOf("{");
  const end = text.lastIndexOf("}");
  if (start === -1 || end === -1 || end < start) {
    throw new Error(`Could not find a JSON object in Codex review output:\n${text}`);
  }
  return text.slice(start, end + 1);
}

function logReview(review: ReviewResult): void {
  $.log(`Review: ${review.approved ? "approved" : "changes requested"} — ${review.summary}`);
  for (const issue of review.issues) {
    $.logLight(`  [${issue.severity}] ${issue.description}`);
  }
}

// setup ------------------------------------------------------------------------

function requireApiKey(): string {
  const apiKey = Deno.env.get("OPENAI_API_KEY");
  if (!apiKey) {
    throw new Error("OPENAI_API_KEY is not set. It is required to run the AI fixer and reviewer.");
  }
  return apiKey;
}

async function ensureCodexInstalled(): Promise<void> {
  const found = await $`codex --version`.noThrow().quiet();
  if (found.code === 0) {
    return;
  }
  $.logStep("Installing OpenAI Codex CLI...");
  await $`npm install -g @openai/codex`;
}

// `codex exec` does not read OPENAI_API_KEY on its own -- it needs credentials
// stored via `codex login` first, otherwise its requests go out with no auth
// header and the API returns 401. The key is piped over stdin so it never
// appears in the process arguments.
async function codexLogin(apiKey: string): Promise<void> {
  $.logStep("Authenticating Codex with the OpenAI API key...");
  await $`codex login --with-api-key`.stdinText(apiKey);
}
