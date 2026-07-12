/**
 * Reconciles this plugin with a new Mago release using OpenAI, in two stages:
 *
 *   1. a Codex agentic session edits the source and runs `cargo test` until
 *      green (fixing breakage and/or wiring up new `FormatSettings`), then
 *   2. a SEPARATE reviewer model independently reviews the resulting diff.
 *      If it finds blocking issues they are fed back to Codex for another
 *      pass, bounded by `REVIEW_MAX_ROUNDS`. If it still isn't approved, this
 *      throws so the workflow fails and nothing is published.
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
  /** Whether `cargo test` already passed with the version bump applied. */
  testsPassed: boolean;
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
  const { isPatchBump, fromVersion, toVersion, testsPassed } = options;

  const situation = testsPassed
    ? `Mago was upgraded from ${fromVersion} to ${toVersion} (a ${
      isPatchBump ? "patch" : "minor"
    } bump). The project already compiles and \`cargo test\` passes, but a new Mago version may have ADDED, RENAMED, or REMOVED formatting settings that should be surfaced by this plugin.`
    : `Mago was upgraded from ${fromVersion} to ${toVersion} and the project no longer builds or \`cargo test\` fails. This is almost always because Mago's public API (usually its \`FormatSettings\` struct or related enums) changed.`;

  return [
    `You are updating the "dprint-plugin-mago" Rust crate, a dprint plugin that wraps the \`mago-formatter\` crate to format PHP.`,
    ``,
    situation,
    ``,
    `Your goal: make \`cargo test\` pass AND keep this plugin's configuration surface in sync with mago-formatter's \`FormatSettings\`. Do NOT commit or push; only edit files in the working tree.`,
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
    `4. When done, run \`cargo test\` and ensure it passes. Iterate until it does.`,
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

async function reviewChanges(options: AiFixOptions): Promise<ReviewResult> {
  const diff = await getWorkingTreeDiff();
  // default to a different, general-purpose model than the Codex fixer so the
  // review is a genuinely independent second opinion.
  const model = Deno.env.get("REVIEW_MODEL") ?? "gpt-5";

  $.logStep(`Reviewing changes with ${model}...`);
  const res = await fetch("https://api.openai.com/v1/chat/completions", {
    method: "POST",
    headers: {
      "Authorization": `Bearer ${Deno.env.get("OPENAI_API_KEY")}`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      model,
      messages: [
        {
          role: "system",
          content:
            "You are a meticulous Rust code reviewer for dprint-plugin-mago, a dprint plugin wrapping the mago-formatter crate to format PHP. You review a diff produced by another AI that reconciled the plugin with a new Mago release. Approve only if the changes are correct and complete; flag anything wrong as a blocking issue.",
        },
        { role: "user", content: buildReviewPrompt(options, diff) },
      ],
      response_format: {
        type: "json_schema",
        json_schema: {
          name: "review",
          strict: true,
          schema: {
            type: "object",
            properties: {
              approved: { type: "boolean" },
              summary: { type: "string" },
              issues: {
                type: "array",
                items: {
                  type: "object",
                  properties: {
                    severity: { type: "string", enum: ["blocking", "nit"] },
                    description: { type: "string" },
                  },
                  required: ["severity", "description"],
                  additionalProperties: false,
                },
              },
            },
            required: ["approved", "summary", "issues"],
            additionalProperties: false,
          },
        },
      },
    }),
  });

  if (!res.ok) {
    throw new Error(`OpenAI review request failed: ${res.status} ${await res.text()}`);
  }
  const data = await res.json();
  const content = data.choices?.[0]?.message?.content;
  if (typeof content !== "string") {
    throw new Error("OpenAI review returned no content.");
  }
  return JSON.parse(content) as ReviewResult;
}

function buildReviewPrompt(options: AiFixOptions, diff: string): string {
  return [
    `Mago was upgraded from ${options.fromVersion} to ${options.toVersion}. Review the diff below, which reconciles dprint-plugin-mago with that release.`,
    ``,
    describeWiring(),
    ``,
    `You only have the diff below (not the mago-formatter source), so review for internal consistency and obvious mistakes:`,
    `- The \`build_format_settings\` mapping is self-consistent: every \`settings.<field>\` it assigns has a matching \`Configuration\` field and dprint config key, and enum arms line up with the plugin's enums.`,
    `- Any newly exposed config option is wired through ALL layers: configuration.rs, resolve_config.rs, format_text.rs, deployment/schema.json, and README.md. A partial addition (e.g. a field added to the struct but missing from the schema or README) is a blocking issue.`,
    `- Naming conventions are consistent (Rust snake_case fields, camelCase dprint keys, matching schema/README).`,
    `- No obvious correctness bugs, and code style matches the surrounding code.`,
    `- If something looks like it needs checking against the actual mago-formatter API (e.g. a renamed field), flag it as a blocking issue so the fixer re-verifies it against the upstream source.`,
    ``,
    `Set approved=false if there is any blocking issue. Nits alone should not block. Keep issue descriptions specific and actionable.`,
    ``,
    `--- DIFF ---`,
    diff,
  ].join("\n");
}

function logReview(review: ReviewResult): void {
  $.log(`Review: ${review.approved ? "approved" : "changes requested"} — ${review.summary}`);
  for (const issue of review.issues) {
    $.logLight(`  [${issue.severity}] ${issue.description}`);
  }
}

async function getWorkingTreeDiff(): Promise<string> {
  // `-N` records intent-to-add so newly created files also appear in the diff.
  await $`git add -N .`.quiet();
  const diff = await $`git diff`.text();
  const maxLen = 200_000;
  return diff.length > maxLen ? diff.slice(0, maxLen) + "\n... (diff truncated)" : diff;
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
