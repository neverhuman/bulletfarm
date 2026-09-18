import { createRequire } from "node:module";
import { mkdir, open, realpath, stat } from "node:fs/promises";
import { dirname, isAbsolute, join, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { startPortalSampling } from "./demo-gif-portal-sampler.mjs";

export function commandObservation(value, expectedId = null) {
  const statuses = ["PENDING", "APPLIED", "VERIFIED", "FAILED", "UNKNOWN"];
  if (value === null || typeof value !== "object" || Array.isArray(value)
      || typeof value.id !== "string" || value.id.length < 1 || value.id.length > 256
      || (expectedId !== null && value.id !== expectedId)
      || value.kind !== "run_demo" || !statuses.includes(value.status)
      || typeof value.payload_digest !== "string" || !/^[a-f0-9]{64}$/.test(value.payload_digest)) {
    throw new Error("invalid command observation");
  }
  return { id: value.id, kind: value.kind, payload_digest: value.payload_digest,
    ledger_status: value.status, product_completion: "UNVERIFIED" };
}

export function validateOrigin(origin) {
  const url = new URL(origin);
  if (url.protocol !== "http:" || url.hostname !== "127.0.0.1" || !url.port
      || url.username || url.password || url.pathname !== "/" || url.search || url.hash) {
    throw new Error("explicit loopback origin required");
  }
  return url.origin;
}

async function privateRoot(out) {
  if (!isAbsolute(out) || resolve(out) !== out || await realpath(dirname(out)) !== dirname(out)) {
    throw new Error("canonical private output parent required");
  }
  const parent = await stat(dirname(out));
  if (parent.uid !== process.getuid() || (parent.mode & 0o077) !== 0) {
    throw new Error("private output owner/mode refused");
  }
  await mkdir(out, { mode: 0o700 }); // Existing attempts, including symlinks, refuse.
  await mkdir(join(out, "frames"), { mode: 0o700 });
}

async function exclusiveFile(path, bytes) {
  const handle = await open(path, "wx", 0o600);
  try {
    await handle.writeFile(bytes);
    await handle.sync();
  } finally {
    await handle.close();
  }
}

async function exclusiveJson(path, value) {
  await exclusiveFile(path, `${JSON.stringify(value, null, 2)}\n`);
}

export async function capturePortal(env = process.env) {
  const origin = validateOrigin(env.PORTAL_ORIGIN);
  const out = env.PORTAL_CAPTURE_DIR;
  const token = env.BULLET_BOOTSTRAP_TOKEN;
  if (!out || !token || !env.PORTAL_PACKAGE_JSON) throw new Error("capture inputs missing");
  await privateRoot(out);
  const observation = { schema_version: "bullet.portal-capture.v1", started_at: new Date().toISOString(),
    origin, classification: "PRIVATE_PORTAL_OBSERVATION", bullet_live_admission: false,
    listener_identity: "UNVERIFIED",
    product_completion: "UNVERIFIED", public_export_review: "REQUIRED", steps: [], frames: [],
    capture_status: "INCOMPLETE" };
  await exclusiveJson(join(out, "started.json"), observation);
  let browser;
  let context;
  let sampler;
  let failure;
  const started = performance.now();
  try {
    const { chromium } = createRequire(env.PORTAL_PACKAGE_JSON)("playwright");
    browser = await chromium.launch({ headless: true, timeout: 10000 });
    context = await browser.newContext({ viewport: { width: 1920, height: 1080 }, deviceScaleFactor: 1 });
    const page = await context.newPage();
    page.setDefaultTimeout(8000);
    page.setDefaultNavigationTimeout(10000);
    function responseFor(path, method) {
      return page.waitForResponse((response) => response.url() === `${origin}${path}`
        && response.request().method() === method);
    }
    await page.goto(`${origin}/#/control-tower`, { waitUntil: "domcontentloaded" });
    sampler = await startPortalSampling({ page, out, frames: observation.frames, started });
    observation.sampling = sampler.summary;
    observation.sampling.start_boundary = "INITIAL_DOMCONTENTLOADED";
    observation.sampling.landmark_reading_ms = 400;
    async function shot(name) {
      await sampler.checkpoint(name);
      await new Promise((resolve) => setTimeout(resolve, 400));
    }
    await page.getByRole("heading", { name: "Control Tower" }).waitFor();
    await page.getByLabel("One-time bootstrap token").waitFor();
    observation.steps.push({ step: "bootstrap_form", observed: true });
    await shot("01-bootstrap-form");
    const field = page.getByLabel("One-time bootstrap token");
    if (await field.getAttribute("type") !== "password") throw new Error("unmasked token input");
    await field.fill(token);
    await shot("02-token-entered-masked");
    const [auth] = await Promise.all([
      responseFor("/api/v1/auth/bootstrap", "POST"),
      page.getByRole("button", { name: "Authenticate local session" }).click(),
    ]);
    observation.steps.push({ step: "bootstrap_exchange_response", http_status: auth.status() });
    const authBody = await auth.json();
    if (auth.status() !== 200 || authBody === null || typeof authBody !== "object"
        || typeof authBody.csrf_token !== "string" || !authBody.csrf_token) {
      throw new Error("bootstrap exchange not observed successful");
    }
    // Do not persist the response, cookies, or CSRF/bootstrap values.
    observation.steps.push({ step: "bootstrap_exchange", http_status: auth.status() });
    await page.getByTestId("auth-state").waitFor();
    await shot("03-session-material");
    const [admission] = await Promise.all([
      responseFor("/api/v1/commands", "POST"),
      page.getByRole("button", { name: "Submit durable demo command" }).click(),
    ]);
    observation.steps.push({ step: "command_admission_response", http_status: admission.status() });
    const command = commandObservation(await admission.json());
    if (admission.status() !== 202 || command.ledger_status !== "PENDING") {
      throw new Error("new command admission not observed");
    }
    observation.command = command;
    observation.steps.push({ step: "command_admission", http_status: admission.status(), command_id: command.id });
    await page.getByTestId("command-id").waitFor();
    if (await page.getByTestId("command-id").textContent() !== command.id) throw new Error("displayed command mismatch");
    observation.displayed_phase = await page.getByTestId("phase").textContent();
    await shot("04-command-observation");
    for (const [nav, marker, frame] of [
      ["shift-brief", "shift-brief", "05-shift-brief"],
      ["fleet", "surface-fleet", "06-fleet"],
      ["mission-graph", "surface-mission-graph", "07-mission-graph"],
      ["control-tower", "status-header", "08-control-tower-return"],
    ]) {
      await page.getByTestId(`nav-${nav}`).click();
      await page.getByTestId(marker).waitFor();
      observation.steps.push({ step: nav, surface_present: true });
      await shot(frame);
    }
    observation.capture_status = "CAPTURED";
  } catch (error) {
    observation.capture_status = "FAILED";
    observation.failure_class = error instanceof Error ? error.name : "UnknownError";
    failure = error;
  } finally {
    try {
      if (sampler) await sampler.stop();
    } catch (error) {
      observation.capture_status = "FAILED";
      observation.sampling_cleanup = "FAILED";
      failure ??= error;
    }
    try {
      if (context) await context.close();
    } catch (error) {
      observation.capture_status = "FAILED";
      observation.context_cleanup = "FAILED";
      failure ??= error;
    }
    try {
      if (browser) await browser.close();
    } catch (error) {
      observation.capture_status = "FAILED";
      observation.browser_cleanup = "FAILED";
      failure ??= error;
    }
    observation.elapsed_ms = performance.now() - started;
    await exclusiveJson(join(out, "observation.json"), observation);
  }
  if (failure) throw new Error("Portal capture failed; private observations retained");
  return observation;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  await capturePortal();
}
