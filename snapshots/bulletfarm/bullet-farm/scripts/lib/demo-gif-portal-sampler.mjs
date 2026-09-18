import { createHash } from "node:crypto";
import { open, readFile } from "node:fs/promises";
import { join } from "node:path";

const CEILINGS = Object.freeze({ hz: 4, maxFrames: 64, maxSeconds: 15, maxBytes: 32 * 1024 * 1024 });
const SINGLE_PNG_BYTES = 8 * 1024 * 1024;

export function samplingLimits(overrides = {}) {
  if (overrides === null || typeof overrides !== "object" || Array.isArray(overrides)
      || Object.keys(overrides).some((key) => !Object.hasOwn(CEILINGS, key))) throw new Error("invalid sampling limits");
  const limits = { ...CEILINGS, ...overrides };
  for (const [key, value] of Object.entries(limits)) {
    if (!Number.isFinite(value) || value <= 0 || value > CEILINGS[key]
        || (["maxFrames", "maxBytes"].includes(key) && !Number.isInteger(value))) {
      throw new Error("invalid sampling limit");
    }
  }
  return limits;
}

export async function startPortalSampling({ page, out, frames, started, limits: overrides = {} }) {
  const limits = samplingLimits(overrides);
  if (!Number.isFinite(started) || !Array.isArray(frames) || frames.length !== 0) {
    throw new Error("fresh sampling state required");
  }
  const source = await readFile(new URL(import.meta.url));
  const began = performance.now();
  const deadline = began + limits.maxSeconds * 1000;
  const summary = { source: "TIMED_BROWSER_PNG_SAMPLES", status: "RUNNING", limits,
    sampler_source_sha256: createHash("sha256").update(source).digest("hex"),
    executable_closure: "UNVERIFIED", animations: "DISABLED", interpolation: "NONE",
    filesystem_io_deadline: "NOT_GUARANTEED", frame_count: 0, bytes: 0, checkpoints: [] };
  let stopping = false;
  let failure;
  let pending;
  let wake;
  function fail(code) {
    const error = new Error(code);
    error.name = "PortalSamplingError";
    return error;
  }
  const done = (async () => {
    while (!stopping) {
      const before = performance.now();
      if (before >= deadline) throw fail("CAPTURE_DEADLINE");
      if (frames.length >= limits.maxFrames) throw fail("FRAME_LIMIT");
      const image = await page.screenshot({ type: "png", animations: "disabled",
        mask: [page.locator("#bootstrap-token")], timeout: Math.min(8000, deadline - before) });
      const captured = performance.now();
      if (image.length > SINGLE_PNG_BYTES || summary.bytes + image.length > limits.maxBytes) {
        throw fail("PNG_BYTE_LIMIT");
      }
      if (image.length < 33 || !image.subarray(0, 8).equals(Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]))
          || image.toString("ascii", 12, 16) !== "IHDR" || image.readUInt32BE(16) !== 1920
          || image.readUInt32BE(20) !== 1080 || image[24] !== 8 || ![2, 6].includes(image[25])) {
        throw fail("PNG_GEOMETRY");
      }
      const file = `frames/sample-${String(frames.length + 1).padStart(3, "0")}.png`;
      const handle = await open(join(out, file), "wx", 0o600);
      try {
        await handle.writeFile(image);
        await handle.sync();
      } finally {
        await handle.close();
      }
      const frame = { file, elapsed_ms: captured - started, capture_started_ms: before - started,
        write_completed_ms: performance.now() - started,
        sha256: createHash("sha256").update(image).digest("hex"), bytes: image.length,
        width: 1920, height: 1080, source: "BROWSER_PNG_SCREENSHOT" };
      frames.push(frame);
      summary.bytes += image.length;
      if (performance.now() >= deadline) throw fail("CAPTURE_DEADLINE");
      if (pending && frame.capture_started_ms >= pending.after) {
        summary.checkpoints.push({ name: pending.name, file, requested_ms: pending.after });
        pending.resolve(frame);
        pending = undefined;
      }
      if (!stopping) {
        // A slow sample resets the schedule; there is no catch-up queue.
        const delay = Math.min(1000 / limits.hz, deadline - performance.now());
        await new Promise((resolve) => {
          const timer = setTimeout(() => { wake = undefined; resolve(); }, Math.max(0, delay));
          wake = () => { clearTimeout(timer); wake = undefined; resolve(); };
        });
      }
    }
  })().catch((error) => {
    failure = error;
    stopping = true;
    summary.status = "FAILED";
    summary.failure_class = error instanceof Error ? error.name : "UnknownError";
    summary.failure_code = error?.name === "PortalSamplingError" ? error.message : "CAPTURE_OR_WRITE_FAILED";
    pending?.reject(error);
    pending = undefined;
  }).finally(() => {
    summary.frame_count = frames.length;
    summary.duration_ms = performance.now() - began;
    summary.capture_gaps_ms = frames.slice(1).map((frame, index) => frame.capture_started_ms - frames[index].capture_started_ms);
    summary.observed_hz = frames.length > 1
      ? (frames.length - 1) * 1000 / (frames.at(-1).capture_started_ms - frames[0].capture_started_ms) : 0;
    if (!failure) summary.status = "STOPPED";
  }); // Rejection is handled from construction; stop() reports the latched failure.
  return { summary,
    checkpoint(name) {
      if (failure) return Promise.reject(failure);
      if (stopping || pending) return Promise.reject(fail("CHECKPOINT_STATE"));
      return new Promise((resolve, reject) => {
        pending = { name, after: performance.now() - started, resolve, reject };
      });
    },
    async stop() {
      stopping = true;
      wake?.();
      if (pending) { pending.reject(fail("CHECKPOINT_STOPPED")); pending = undefined; }
      await done; // Settle the real screenshot and durable write before browser teardown.
      if (failure) throw failure;
    },
  };
}
