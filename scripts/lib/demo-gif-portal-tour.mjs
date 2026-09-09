import { createRequire } from "node:module";
import { mkdir, writeFile } from "node:fs/promises";
import { join } from "node:path";

const require = createRequire(process.env.PORTAL_PACKAGE_JSON);
const { chromium } = require("playwright");

const origin = process.env.PORTAL_ORIGIN;
const out = process.env.PORTAL_CAPTURE_DIR;
const token = process.env.BULLET_BOOTSTRAP_TOKEN;
if (!origin || !out || !token) {
  throw new Error("PORTAL_ORIGIN, PORTAL_CAPTURE_DIR, and BULLET_BOOTSTRAP_TOKEN are required");
}

await mkdir(out, { recursive: true });
await mkdir(join(out, "frames"), { recursive: true });

const browser = await chromium.launch({ headless: true });
const context = await browser.newContext({
  viewport: { width: 1920, height: 1080 },
  deviceScaleFactor: 1,
  recordVideo: {
    dir: join(out, "video-raw"),
    size: { width: 1920, height: 1080 },
  },
});
const page = await context.newPage();
page.setDefaultTimeout(30000);

async function shot(name) {
  await page.screenshot({
    path: join(out, "frames", `${name}.png`),
    type: "png",
    animations: "disabled",
  });
}

async function hold(ms) {
  await page.waitForTimeout(ms);
}

await page.goto(`${origin}/#/control-tower`, { waitUntil: "networkidle" });
await page.waitForSelector('[data-testid="status-header"]');
await page.getByRole("heading", { name: "Control Tower" }).waitFor();
await page.getByLabel("One-time bootstrap token").waitFor();
await hold(900);
await shot("01-bootstrap-form");

const field = page.getByLabel("One-time bootstrap token");
await field.click();
await field.pressSequentially(token, { delay: 28 });
await hold(500);
await shot("02-token-entered");

await page.getByRole("button", { name: "Authenticate local session" }).click();
await page.getByTestId("auth-state").waitFor();
await hold(800);
await shot("03-authenticated");

await page.getByRole("button", { name: "Submit durable demo command" }).click();
await page.getByTestId("phase").waitFor();
await hold(1200);
await shot("04-demo-command");

await page.locator('[data-testid="nav-shift-brief"]').click();
await page.waitForSelector('[data-testid="shift-brief"]');
await hold(800);
await shot("05-shift-brief");

await page.locator('[data-testid="nav-fleet"]').click();
await page.waitForSelector('[data-testid="surface-fleet"]');
await hold(800);
await shot("06-fleet");

await page.locator('[data-testid="nav-mission-graph"]').click();
await page.waitForSelector('[data-testid="surface-mission-graph"]');
await hold(800);
await shot("07-mission-graph");

await page.locator('[data-testid="nav-control-tower"]').click();
await page.waitForSelector('[data-testid="status-header"]');
await page.getByTestId("auth-state").waitFor();
await hold(1000);
await shot("08-control-tower-return");

const video = page.video();
await context.close();
await browser.close();
if (video) {
  const videoPath = await video.path();
  await writeFile(join(out, "video-path.txt"), `${videoPath}\n`);
}
