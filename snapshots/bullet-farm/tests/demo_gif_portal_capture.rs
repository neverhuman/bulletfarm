#![cfg(target_os = "linux")]

use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
};

const PLAYWRIGHT_PROBE: &str = r"
import {createRequire} from 'node:module';
import {accessSync, constants} from 'node:fs';
const {chromium} = createRequire(process.env.PORTAL_PACKAGE_JSON)('playwright');
accessSync(chromium.executablePath(), constants.X_OK);
";

/// Every browser case needs Node to resolve `playwright`, with an executable
/// Chromium, from the portal's `node_modules`. A checkout without that tree
/// (hosted single-repo CI) refuses by name instead of dying on a raw module
/// error; when the subject is present every assertion below still applies.
fn playwright_available(package: &Path) -> bool {
    let probe = Command::new("timeout")
        .args([
            "--kill-after=2s",
            "20s",
            "node",
            "--input-type=module",
            "-e",
            PLAYWRIGHT_PROBE,
        ])
        .env("PORTAL_PACKAGE_JSON", package)
        .output();
    let reason = match probe {
        Ok(output) if output.status.success() => return true,
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            stderr
                .lines()
                .find(|line| line.contains("Error:"))
                .or_else(|| stderr.lines().find(|line| !line.trim().is_empty()))
                .unwrap_or("probe exited non-zero")
                .to_owned()
        }
        Err(error) => format!("could not start node: {error}"),
    };
    eprintln!(
        "MEDIA_CAPTURE_SUBJECTS_UNAVAILABLE: playwright — node cannot resolve 'playwright' with an executable Chromium from {}; hosted single-repo checkouts do not provide the portal's node_modules ({reason})",
        package.display()
    );
    false
}

const BROWSER: &str = r#"
import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {mkdir,readFile,writeFile,chmod,readdir} from 'node:fs/promises';
import {join} from 'node:path';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
const root=process.env.CAPTURE_TEST_ROOT, hub=process.env.CAPTURE_TEST_HUB;
const {startPortalSampling,samplingLimits}=await import(pathToFileURL(join(hub,'scripts/lib/demo-gif-portal-sampler.mjs')));
const {capturePortal}=await import(pathToFileURL(join(hub,'scripts/lib/demo-gif-portal-tour.mjs')));
const {chromium}=createRequire(process.env.PORTAL_PACKAGE_JSON)('playwright');
const server=await chromium.launchServer({headless:true,executablePath:chromium.executablePath(),timeout:10000});
const browser=await chromium.connect(server.wsEndpoint());
const browserVersion=browser.version();
const watchdog=setTimeout(()=>{server.kill().finally(()=>process.exit(124));},30000);
const context=await browser.newContext({viewport:{width:1920,height:1080},deviceScaleFactor:1});
await context.route('**/*',route=>route.abort());
const page=await context.newPage();
page.setDefaultTimeout(3000);
const sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms));
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
async function sample(name,limits={}) {
  await page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));
  const out=join(root,name);await mkdir(out,{mode:0o700});await mkdir(join(out,'frames'),{mode:0o700});
  const frames=[],started=performance.now();
  const sampler=await startPortalSampling({page,out,frames,started,limits});
  return {out,frames,sampler};
}
async function retained(capture) {
  await writeFile(join(capture.out,'sampling.json'),JSON.stringify(capture.sampler.summary));
  for(const frame of capture.frames) {
    const bytes=await readFile(join(capture.out,frame.file));assert.equal(hash(bytes),frame.sha256);
    assert.equal(bytes.readUInt32BE(16),1920);assert.equal(bytes.readUInt32BE(20),1080);
    assert(frame.capture_started_ms<=frame.elapsed_ms && frame.elapsed_ms<=frame.write_completed_ms);
  }
}
try {
"#;

fn browser_case(body: &str) {
    let root = tempfile::Builder::new()
        .permissions(fs::Permissions::from_mode(0o700))
        .tempdir()
        .unwrap();
    let hub = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let package = std::env::var_os("PORTAL_PACKAGE_JSON")
        .map(PathBuf::from)
        .unwrap_or_else(|| hub.parent().unwrap().join("bullet-portal/package.json"));
    if !playwright_available(&package) {
        return;
    }
    let source = format!(
        "{BROWSER}\n{body}\n}} finally {{ clearTimeout(watchdog); await browser.close(); await server.close(); assert.notEqual(server.process().exitCode,null); console.log(JSON.stringify({{fixture:'REAL_CHROMIUM_SYNTHETIC',browser_version:browserVersion,browser_pid:server.process().pid,browser_exit:server.process().exitCode}})); }}"
    );
    let output = Command::new("timeout")
        .args([
            "--kill-after=2s",
            "40s",
            "node",
            "--input-type=module",
            "-e",
        ])
        .arg(&source)
        .env("CAPTURE_TEST_ROOT", root.path())
        .env("CAPTURE_TEST_HUB", hub)
        .env("PORTAL_PACKAGE_JSON", package)
        .output()
        .unwrap();
    if let Some(archive) = std::env::var_os("BULLET_BROWSER_TEST_EVIDENCE") {
        let archive = PathBuf::from(archive);
        fs::create_dir_all(&archive).unwrap();
        let destination = archive.join(root.path().file_name().unwrap());
        fs::write(root.path().join("fixture.mjs"), &source).unwrap();
        fs::write(root.path().join("stdout.log"), &output.stdout).unwrap();
        fs::write(root.path().join("stderr.log"), &output.stderr).unwrap();
        fs::rename(root.into_path(), destination).unwrap();
    }
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn browser_samples_real_changes_and_preserves_static_control() {
    browser_case(
        r##"
await page.setContent('<body style="margin:0;background:#abcdef">static</body>');
const stable=await sample('stable');
await stable.sampler.checkpoint('first');await stable.sampler.checkpoint('second');await stable.sampler.stop();
assert.equal(new Set(stable.frames.map(f=>f.sha256)).size,1);await retained(stable);
await page.setContent('<body style="margin:0"><div id="counter" style="font-size:80px"></div><script>let n=0;setInterval(()=>{counter.textContent=String(++n);document.body.style.backgroundColor=n%2?"#abcdef":"#fedcba"},40)</script>');
const moving=await sample('moving');
await moving.sampler.checkpoint('first');await moving.sampler.checkpoint('second');await moving.sampler.stop();
assert(new Set(moving.frames.map(f=>f.sha256)).size>1);assert(moving.frames.length>=2);
assert(moving.sampler.summary.capture_gaps_ms.every(gap=>gap>=250));
assert.equal(moving.sampler.summary.source,'TIMED_BROWSER_PNG_SAMPLES');
assert.equal(moving.sampler.summary.interpolation,'NONE');await retained(moving);
"##,
    );
}

#[test]
fn browser_masks_dummy_values_and_keeps_unmasked_negative() {
    browser_case(
        r##"
await page.setContent('<input id="bootstrap-token" style="font:40px monospace;width:600px" value="dummy-one">');
const unmaskedOne=await page.screenshot({type:'png'});
const first=await sample('first');await first.sampler.checkpoint('dummy-one');await first.sampler.stop();
await page.locator('#bootstrap-token').fill('different-dummy-value');await page.locator('#bootstrap-token').blur();
const unmaskedTwo=await page.screenshot({type:'png'});
const second=await sample('second');await second.sampler.checkpoint('dummy-two');await second.sampler.stop();
assert.notEqual(hash(unmaskedOne),hash(unmaskedTwo));
assert.equal(first.frames.at(-1).sha256,second.frames.at(-1).sha256);
await retained(first);await retained(second);
"##,
    );
}

#[test]
fn delayed_real_screenshots_are_serial_and_checkpointed_after_landmarks() {
    browser_case(
        r##"
await page.setContent('<body>serial capture</body>');
const actual=page.screenshot.bind(page);let active=0,maxActive=0,completed=0;
page.screenshot=async options=>{active++;maxActive=Math.max(active,maxActive);try{await sleep(320);return await actual(options)}finally{active--;completed++}};
const capture=await sample('serial');
await capture.sampler.checkpoint('first');await capture.sampler.checkpoint('second');await capture.sampler.stop();
assert.equal(active,0);assert.equal(maxActive,1);assert.equal(completed,capture.frames.length);
assert(capture.sampler.summary.capture_gaps_ms.every(gap=>gap>=570));
const early=await sample('stop-in-flight');await sleep(40);assert.equal(active,1);
const beforeStop=completed;await early.sampler.stop();
assert.equal(active,0);assert.equal(completed,beforeStop+1);assert.equal(early.frames.length,1);
await retained(early);
for(const point of capture.sampler.summary.checkpoints) {
  const frame=capture.frames.find(f=>f.file===point.file);assert(frame.capture_started_ms>=point.requested_ms);
}
await retained(capture);
"##,
    );
}

#[test]
fn browser_caps_refuse_without_discarding_prior_pngs() {
    browser_case(
        r##"
for(const limits of [{hz:NaN},{maxFrames:65},{maxBytes:Infinity},{maxSeconds:16},{maxFrames:1.5},{hz:0},{unknown:1},{toString:1}]) assert.throws(()=>samplingLimits(limits));
await page.setContent('<body>bounded capture</body>');
const frameBound=await sample('frames',{maxFrames:2});
await frameBound.sampler.checkpoint('first');
await assert.rejects(frameBound.sampler.checkpoint('beyond-limit'));
await assert.rejects(frameBound.sampler.stop());
assert.equal(frameBound.frames.length,2);assert.equal(frameBound.sampler.summary.failure_code,'FRAME_LIMIT');await retained(frameBound);
const byteBound=await sample('bytes',{maxBytes:1});
await assert.rejects(byteBound.sampler.checkpoint('too-large'));await assert.rejects(byteBound.sampler.stop());
assert.equal(byteBound.frames.length,0);assert.equal(byteBound.sampler.summary.failure_code,'PNG_BYTE_LIMIT');
assert.equal((await readdir(join(byteBound.out,'frames'))).length,0);await retained(byteBound);
const accepted=await sample('accepted');await accepted.sampler.checkpoint('short');await accepted.sampler.stop();
assert.equal(accepted.sampler.summary.status,'STOPPED');await retained(accepted);
"##,
    );
}

#[test]
fn browser_deadline_and_real_write_failure_settle_before_close() {
    browser_case(
        r##"
await page.setContent('<body>retained failure</body>');
const actual=page.screenshot.bind(page);let active=0;
page.screenshot=async options=>{active++;try{await page.evaluate(()=>{const end=performance.now()+350;while(performance.now()<end){/* synthetic browser busywork */}});return await actual(options)}finally{active--}};
const deadline=await sample('deadline',{maxSeconds:0.2});
await assert.rejects(deadline.sampler.checkpoint('late'));await assert.rejects(deadline.sampler.stop());
assert.equal(active,0);assert.equal(deadline.sampler.summary.status,'FAILED');await retained(deadline);
page.screenshot=actual;
const denied=await sample('denied');await denied.sampler.checkpoint('retained-first');
const prior=denied.frames.length;await chmod(join(denied.out,'frames'),0o500);
try {
  await assert.rejects(denied.sampler.checkpoint('write-denied'));await assert.rejects(denied.sampler.stop());
  assert.equal(denied.frames.length,prior);assert.equal(denied.sampler.summary.status,'FAILED');
} finally {await chmod(join(denied.out,'frames'),0o700)}
await retained(denied);
const names=await readdir(join(denied.out,'frames'));await context.close();await sleep(350);
assert.deepEqual(await readdir(join(denied.out,'frames')),names);
"##,
    );
}

#[test]
fn real_browser_tour_uses_samples_and_drains_failed_capture() {
    browser_case(
        r##"
const html=`<body><h1>Control Tower</h1><label>One-time bootstrap token<input type="password" id="bootstrap-token"></label>
<button id="auth">Authenticate local session</button><div id="state"></div>
<button id="submit">Submit durable demo command</button><div id="command"></div>
<button data-testid="nav-shift-brief">brief</button><div data-testid="shift-brief">brief</div>
<button data-testid="nav-fleet">fleet</button><div data-testid="surface-fleet">fleet</div>
<button data-testid="nav-mission-graph">graph</button><div data-testid="surface-mission-graph">graph</div>
<button data-testid="nav-control-tower">tower</button><div data-testid="status-header">status</div>
<script>auth.onclick=async()=>{await fetch('/api/v1/auth/bootstrap',{method:'POST'});state.innerHTML='<div data-testid="auth-state">synthetic</div>'};
submit.onclick=async()=>{const r=await fetch('/api/v1/commands',{method:'POST'});const c=await r.json();command.innerHTML='<div data-testid="command-id">'+c.id+'</div><div data-testid="phase">PENDING</div>'};</script></body>`;
const fixture=join(root,'fixture');await mkdir(join(fixture,'node_modules/playwright'),{recursive:true,mode:0o700});
await writeFile(join(fixture,'package.json'),'{}');
await writeFile(join(fixture,'node_modules/playwright/index.js'),'exports.chromium={launch:async()=>globalThis.realFixtureBrowser()};');
const actualContext=browser.newContext.bind(browser),actualClose=browser.close.bind(browser);
let active=0,closed=0,failed=false,network=0;
globalThis.realFixtureBrowser=async()=>({
  newContext:async options=>{
    const c=await actualContext(options),newPage=c.newPage.bind(c),close=c.close.bind(c);
    await c.route('**/*',async route=>{
      network++;const url=route.request().url();
      if(url==='http://127.0.0.1:7421/') return route.fulfill({contentType:'text/html',body:html});
      if(url==='http://127.0.0.1:7421/api/v1/auth/bootstrap') return route.fulfill({status:200,json:{csrf_token:'dummy-not-a-credential'}});
      if(url==='http://127.0.0.1:7421/api/v1/commands') return route.fulfill({status:202,json:{id:'cmd_synthetic',kind:'run_demo',payload_digest:'a'.repeat(64),status:'PENDING'}});
      return route.abort();
    });
    c.newPage=async()=>{const p=await newPage(),shot=p.screenshot.bind(p);let count=0;
      p.screenshot=async options=>{active++;try{const png=await shot(options);if(failed && ++count===3) await chmod(join(root,'failed','frames'),0o500);return png}catch(error){await writeFile(join(root,'screenshot-error.json'),JSON.stringify({name:error.name,message:error.message}));throw error}finally{active--}};
      return p;};
    c.close=async()=>{assert.equal(active,0);closed++;await close()};return c;
  },close:async()=>{assert.equal(active,0);closed++}
});
const env={PORTAL_ORIGIN:'http://127.0.0.1:7421',PORTAL_PACKAGE_JSON:join(fixture,'package.json'),BULLET_BOOTSTRAP_TOKEN:'synthetic-only'};
const good=await capturePortal({...env,PORTAL_CAPTURE_DIR:join(root,'good')});
assert.equal(good.capture_status,'CAPTURED');assert.equal(good.sampling.status,'STOPPED');
assert.equal(good.sampling.checkpoints.length,8);assert(good.frames.length>8);assert.equal(closed,2);
assert.equal(good.product_completion,'UNVERIFIED');assert.equal(good.bullet_live_admission,false);
failed=true;
try {await assert.rejects(capturePortal({...env,PORTAL_CAPTURE_DIR:join(root,'failed')}));}
finally {await chmod(join(root,'failed','frames'),0o700)}
const bad=JSON.parse(await readFile(join(root,'failed','observation.json')));
assert.equal(bad.capture_status,'FAILED');assert.equal(bad.sampling.status,'FAILED');assert.equal(closed,4);
const names=await readdir(join(root,'failed','frames'));await sleep(350);
assert.deepEqual(await readdir(join(root,'failed','frames')),names);assert(network>0);
assert(!JSON.stringify([good,bad]).includes('synthetic-only'));
await actualClose();
"##,
    );
}
