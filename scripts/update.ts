/**
 * This script checks for any Mago updates and then automatically
 * publishes a new version of the plugin if so.
 */
import { $, CargoToml, semver } from "automation";
import { aiFixMagoUpdate } from "./ai_fix.ts";

const rootDirPath = $.path(import.meta.dirname!).parentOrThrow();
const cargoToml = new CargoToml(rootDirPath.join("Cargo.toml"));
const currentVersions = getMagoCargoTomlVersions(cargoToml.text());

$.logStep("Getting latest versions from crates.io...");
const latestVersions = await getLatestMagoVersions();

const hasFormatterUpdate = currentVersions.formatter !== latestVersions.formatter;
const hasPhpVersionUpdate = currentVersions.phpVersion !== latestVersions.phpVersion;

if (!hasFormatterUpdate) {
  $.log("No new formatter updates found. Exiting.");
  Deno.exit(0);
}

$.log("Found new versions:");
if (hasFormatterUpdate) {
  $.log(`  mago-formatter: ${currentVersions.formatter} -> ${latestVersions.formatter}`);
}
if (hasPhpVersionUpdate) {
  $.log(`  mago-php-version: ${currentVersions.phpVersion} -> ${latestVersions.phpVersion}`);
}

$.logStep("Updating rust-toolchain.toml...");
await updateRustToolchain(latestVersions.formatter);

$.logStep("Updating Cargo.toml...");
const isPatchBump = hasFormatterUpdate
  ? semver.parse(currentVersions.formatter).major === semver.parse(latestVersions.formatter).major
    && semver.parse(currentVersions.formatter).minor === semver.parse(latestVersions.formatter).minor
  : true;

// Update mago-formatter version
if (hasFormatterUpdate) {
  cargoToml.replaceAll(
    `mago-formatter = "${currentVersions.formatter}"`,
    `mago-formatter = "${latestVersions.formatter}"`,
  );
}
// Update mago-php-version
if (hasPhpVersionUpdate) {
  cargoToml.replaceAll(
    `mago-php-version = "${currentVersions.phpVersion}"`,
    `mago-php-version = "${latestVersions.phpVersion}"`,
  );
}

// Verify the update. A clean patch bump publishes exactly as before. A minor
// bump always gets an AI review (Mago may have added settings without breaking
// the build), and a patch bump that fails the checks gets an AI fix attempt.
$.logStep("Running checks (test + clippy)...");
const checks = await runChecks();

if (!isPatchBump || !checks.passed) {
  if (checks.passed) {
    $.logStep("Minor Mago update — running AI review for new/changed settings...");
  } else {
    $.logStep("Patch update failed the checks — running AI fix...");
  }
  await aiFixMagoUpdate({
    isPatchBump,
    fromVersion: currentVersions.formatter,
    toVersion: latestVersions.formatter,
    checksPassed: checks.passed,
    // hand the failing output to the AI so it can go straight to fixing
    // instead of re-running the checks just to rediscover the errors.
    checkOutput: checks.output,
  });

  // the AI must leave the project in a passing state, otherwise fail the
  // workflow (nothing gets published and the maintainer is notified).
  $.logStep("Re-running checks after AI changes...");
  await assertChecks();
}

if (Deno.args.includes("--skip-publish")) {
  Deno.exit(0);
}

$.logStep(`Committing Mago version bump commit...`);
await $`git add .`;
const message = `${isPatchBump ? "fix" : "feat"}: update to Mago ${latestVersions.formatter}`;
await $`git commit -m ${message}`;

$.logStep("Bumping version in Cargo.toml...");
// reload from disk before bumping: the AI may have edited Cargo.toml (e.g.
// swapping a dependency), and the in-memory copy loaded at startup is stale.
// Writing the stale copy here would clobber those edits.
const releaseCargoToml = new CargoToml(rootDirPath.join("Cargo.toml"));
releaseCargoToml.bumpCargoTomlVersion(isPatchBump ? "patch" : "minor");

// release
const newVersion = releaseCargoToml.version();
$.logStep(`Committing and publishing ${newVersion}...`);
await $`git add .`;
await $`git commit -m ${newVersion}`;
await $`git push origin main`;
await $`git tag ${newVersion}`;
await $`git push origin ${newVersion}`;

// the checks that must pass before publishing. clippy is included because CI
// (and Codex) run it with warnings denied, so a clippy warning is as breaking
// as a test failure, and the wasm release build is included because that is
// what actually ships. `inheritPiped` + `captureCombined` streams the output
// to the CI log live while also capturing it so a failure can be handed to the
// AI.
async function runChecks(): Promise<{ passed: boolean; output: string }> {
  const results = [];
  for (const command of CHECK_COMMANDS) {
    results.push(await capture(command()));
  }
  const failures = results.filter((r) => r.code !== 0);
  return {
    passed: failures.length === 0,
    output: failures.map((r) => r.combined).join("\n\n"),
  };
}

function capture(command: ReturnType<typeof $>) {
  return command
    .stdout("inheritPiped")
    .stderr("inheritPiped")
    .captureCombined()
    .noThrow();
}

// same checks as `runChecks`, but throws on the first failure so the workflow
// aborts before anything is committed, tagged, or published.
async function assertChecks(): Promise<void> {
  for (const command of CHECK_COMMANDS) {
    await command();
  }
}

const CHECK_COMMANDS = [
  () => $`cargo test`,
  () => $`cargo clippy --all-targets --all-features -- -D warnings`,
  () => $`cargo build --target wasm32-unknown-unknown --features wasm --release`,
];

interface MagoVersions {
  formatter: string;
  phpVersion: string;
}

function getMagoCargoTomlVersions(text: string): MagoVersions {
  const formatterMatch = text.match(/mago-formatter = "([^"]+)"/);
  const formatter = formatterMatch?.[1];
  if (formatter == null) {
    throw new Error("Could not find mago-formatter version in Cargo.toml.");
  }

  const phpVersionMatch = text.match(/mago-php-version = "([^"]+)"/);
  const phpVersion = phpVersionMatch?.[1];
  if (phpVersion == null) {
    throw new Error("Could not find mago-php-version version in Cargo.toml.");
  }

  $.logLight("Found mago-formatter version in Cargo.toml:", formatter);
  $.logLight("Found mago-php-version version in Cargo.toml:", phpVersion);

  return { formatter, phpVersion };
}

async function getLatestMagoVersions(): Promise<MagoVersions> {
  const [formatter, phpVersion] = await Promise.all([
    getLatestCrateVersion("mago-formatter"),
    getLatestCrateVersion("mago-php-version"),
  ]);
  return { formatter, phpVersion };
}

async function updateRustToolchain(magoVersion: string) {
  const content = await $.request(
    `https://raw.githubusercontent.com/carthage-software/mago/${magoVersion}/Cargo.toml`,
  ).text();
  const match = content.match(/rust-version\s*=\s*"([^"]+)"/);
  if (match == null) {
    throw new Error("Could not find rust-version in mago's Cargo.toml.");
  }
  const magoRustVersion = match[1];
  const toolchainPath = rootDirPath.join("rust-toolchain.toml");
  const localContent = toolchainPath.readTextSync();
  const localMatch = localContent.match(/channel\s*=\s*"([^"]+)"/);
  if (localMatch == null) {
    throw new Error("Could not find channel in local rust-toolchain.toml.");
  }
  if (localMatch[1] !== magoRustVersion) {
    $.log(`Updating Rust toolchain: ${localMatch[1]} -> ${magoRustVersion}`);
    toolchainPath.writeTextSync(localContent.replace(localMatch[0], `channel = "${magoRustVersion}"`));
  } else {
    $.log(`Rust toolchain already at ${magoRustVersion}.`);
  }
}

async function getLatestCrateVersion(crateName: string): Promise<string> {
  const data = await $.request(`https://crates.io/api/v1/crates/${crateName}`)
    .json<{ crate?: { newest_version?: string } }>();
  const latestVersion = data.crate?.newest_version;
  if (latestVersion == null) {
    throw new Error(`Could not find latest version of ${crateName} on crates.io.`);
  }
  $.logLight(`Latest ${crateName} version on crates.io:`, latestVersion);
  return latestVersion;
}
