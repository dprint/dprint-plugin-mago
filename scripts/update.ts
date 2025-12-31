/**
 * This script checks for any Mago updates and then automatically
 * publishes a new version of the plugin if so.
 */
import { $, CargoToml, semver } from "automation";

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

// run the tests
$.logStep("Running tests...");
await $`cargo test`;

if (Deno.args.includes("--skip-publish")) {
  Deno.exit(0);
}

$.logStep(`Committing Mago version bump commit...`);
await $`git add .`;
const message = `${isPatchBump ? "fix" : "feat"}: update to Mago ${latestVersions.formatter}`;
await $`git commit -m ${message}`;

$.logStep("Bumping version in Cargo.toml...");
cargoToml.bumpCargoTomlVersion(isPatchBump ? "patch" : "minor");

// release
const newVersion = cargoToml.version();
$.logStep(`Committing and publishing ${newVersion}...`);
await $`git add .`;
await $`git commit -m ${newVersion}`;
await $`git push origin main`;
await $`git tag ${newVersion}`;
await $`git push origin ${newVersion}`;

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

async function getLatestCrateVersion(crateName: string): Promise<string> {
  const response = await fetch(`https://crates.io/api/v1/crates/${crateName}`);
  if (!response.ok) {
    throw new Error(`Failed to fetch ${crateName} crate info: ${response.statusText}`);
  }
  const data = await response.json();
  const latestVersion = data.crate?.newest_version;
  if (latestVersion == null) {
    throw new Error(`Could not find latest version of ${crateName} on crates.io.`);
  }
  $.logLight(`Latest ${crateName} version on crates.io:`, latestVersion);
  return latestVersion;
}
