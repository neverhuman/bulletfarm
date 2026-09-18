// Walk Shift Brief + Control Tower + closed Head chip on a real farmd session.
// Cookie is read from the operator store in-process and never printed.
import { createRequire } from "node:module";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { createHash } from "node:crypto";

const portalRoot = process.env.BULLET_RECORD_PORTAL_ROOT;
if (!portalRoot) {
  console.error("PORTAL_ROOT_REQUIRED");
  process.exit(2);
}
const { chromium } = createRequire(`${portalRoot}/package.json`)("playwright");

const origin = process.env.BULLET_RECORD_PORTAL_ORIGIN ?? "http://127.0.0.1:7420";
const stateFile = process.env.BULLET_RECORD_SESSION_FILE;
const out = process.env.BULLET_RECORD_FRAME_DIR;
const width = 1920;
const height = 1080;
if (!stateFile || !out) {
  console.error("PORTAL_NARRATION_INPUTS_REQUIRED");
  process.exit(2);
}

const raw = JSON.parse(await readFile(stateFile, "utf8"));
const cookie = typeof raw.cookie === "string" ? raw.cookie : "";
const value = cookie.startsWith("bullet_session=") ? cookie.slice("bullet_session=".length) : cookie;
if (!value || value.startsWith("ses_") === false) {
  console.error("PORTAL_SESSION_UNAVAILABLE");
  process.exit(3);
}

await mkdir(out, { recursive: true, mode: 0o700 });
const browser = await chromium.launch({
  args: [
    "--disable-lcd-text",
    "--disable-font-subpixel-positioning",
    "--font-render-hinting=none",
    "--force-color-profile=srgb",
    "--disable-gpu",
  ],
});
const context = await browser.newContext({
  viewport: { width, height },
  deviceScaleFactor: 1,
  colorScheme: "dark",
});
await context.addCookies([
  { name: "bullet_session", value, url: origin },
]);
const page = await context.newPage();
const frames = [];

async function shot(name, holdCs) {
  const file = `${name}.png`;
  await page.screenshot({ path: `${out}/${file}`, fullPage: false });
  frames.push({ file, hold_cs: holdCs });
}

await page.goto(`${origin}/#/shift-brief`, { waitUntil: "domcontentloaded" });
await page.waitForTimeout(1200);
await shot("shift-brief", 220);
await page.goto(`${origin}/#/control-tower`, { waitUntil: "domcontentloaded" });
await page.waitForTimeout(1200);
await shot("control-tower", 220);
const head = page.locator("[data-testid=head-chip], [data-testid=talk-chip], button:has-text('Head')").first();
if (await head.count()) {
  await head.click({ timeout: 2000 }).catch(() => undefined);
  await page.waitForTimeout(800);
}
await shot("head-chip", 180);
await browser.close();

const observation = {
  schema: "bullet.portal-observation.v1",
  origin: "loopback-farmd",
  surfaces: ["shift-brief", "control-tower", "head-chip-closed-or-get"],
  send_omitted: true,
  frames: frames.map((row) => row.file),
};
await writeFile(`${out}/observation.json`, `${JSON.stringify(observation, null, 2)}\n`);
await writeFile(`${out}/frames.json`, `${JSON.stringify(frames, null, 2)}\n`);
process.stdout.write(`frames=${frames.length} digest=${createHash("sha256").update(JSON.stringify(observation)).digest("hex").slice(0, 16)}\n`);
