import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { chmodSync, copyFileSync, existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { validateRenderedHost } from "../proof/rendered-host.ts";

assert.doesNotThrow(() => validateRenderedHost("xbabe2", "linux", {}));
for (const host of ["other", "XBABE2", "xbabe2.example"]) {
  assert.throws(() => validateRenderedHost(host, "linux", {}), /RENDERED_HOST_REQUIRED/);
}
for (const platform of ["darwin", "win32"]) {
  assert.throws(() => validateRenderedHost("xbabe2", platform, {}), /RENDERED_HOST_REQUIRED/);
}
for (const key of ["CI", "GITHUB_ACTIONS"]) for (const value of [undefined, "", "false", "true"]) {
  assert.throws(() => validateRenderedHost("xbabe2", "linux", { [key]: value }), /RENDERED_HOST_REQUIRED/);
}

// Product host helper only: no Tuiwright/Playwright invocation, even for refusals.
const helper = pathToFileURL(resolve("ops/proof/rendered-host.ts")).href;
const localEnvironment = { ...process.env };
delete localEnvironment.CI;
delete localEnvironment.GITHUB_ACTIONS;
for (const key of ["CI", "GITHUB_ACTIONS"]) for (const value of ["", "false", "true"]) {
  const result = spawnSync(process.execPath, ["--input-type=module", "-e",
    `import { requireRenderedHost } from ${JSON.stringify(helper)}; requireRenderedHost();`],
  { encoding: "utf8", env: { ...localEnvironment, [key]: value } });
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /RENDERED_HOST_REQUIRED/);
}

// Only the npm dependency is a fixture. Execute the actual contract, wrapper,
// lock/observation producer and failure path. These are not product test receipts.
for (const [stage, status] of [["bundle:typecheck", 23], ["bundle:test", 24]]) {
  const fixture = mkdtempSync(join(tmpdir(), "bullet-contract-failure-"));
  const run = (command, args, env = {}) => spawnSync(command, args, {
    cwd: fixture, encoding: "utf8", env: { ...process.env, ...env },
  });
  const good = (result) => assert.equal(result.status, 0, result.stderr);
  try {
    for (const file of ["scripts/ci-local.sh", "ops/ci/contract.sh", "ops/ci/lib.sh",
      "ops/ci/observation.mjs", "ops/ci/bundle-report.mjs"]) {
      mkdirSync(dirname(join(fixture, file)), { recursive: true });
      copyFileSync(file, join(fixture, file));
    }
    writeFileSync(join(fixture, ".gitignore"), ".ci-artifacts/\n");
    mkdirSync(join(fixture, "fixture-bin"));
    const npm = join(fixture, "fixture-bin/npm");
    writeFileSync(npm, `#!/bin/sh
if [ "$1" = --version ]; then echo 10.9.8; exit 0; fi
printf '%s\\n' "$*" >> .ci-artifacts/npm-invocations
if [ "$1" = run ] && [ "$2" = '${stage}' ]; then exit ${status}; fi
exit 0
`);
    chmodSync(npm, 0o700);
    good(run("git", ["init", "--quiet"]));
    good(run("git", ["add", "."]));
    good(run("git", ["-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
      "commit", "--quiet", "-m", "synthetic contract failure fixture"]));
    const result = run("bash", ["scripts/ci-local.sh", "contract"], {
      PATH: join(fixture, "fixture-bin") + ":" + process.env.PATH,
    });
    assert.equal(result.status, status, result.stderr);
    assert.doesNotMatch(result.stdout, /contract lane passed/);
    assert.equal(existsSync(join(fixture, ".git/bullet-ci.lock.d")), false);
    const observed = JSON.parse(readFileSync(join(fixture, ".ci-artifacts/observations/contract.json")));
    assert.deepEqual(observed.outcomes, [{ lane: "contract", status: "FAIL", exit_code: status }]);
    assert.equal(observed.artifact_hashes.length, 1);
    const commands = readFileSync(join(fixture, ".ci-artifacts/npm-invocations"), "utf8");
    assert.equal(commands, stage === "bundle:typecheck" ? "run bundle:typecheck\n" :
      "run bundle:typecheck\nrun bundle:test\n");
  } finally { rmSync(fixture, { recursive: true, force: true }); }
}
console.log("[ci] six host refusals and both actual contract failure paths passed");
