#!/usr/bin/env node
// Beater Playwright runner.
//
// Bridges the `beater-browser-playwright` Rust driver to a real browser. Reads
// newline-delimited JSON commands on stdin and writes newline-delimited JSON
// responses on stdout, implementing PROTOCOL_VERSION 1 (kept in lockstep with
// `crates/beater-browser-playwright/src/protocol.rs`).
//
// Commands (one JSON object per line):
//   { id, op: "launch", protocol_version, engine, channel?, headless }
//   { id, op: "goto",    url }
//   { id, op: "click",   selector }
//   { id, op: "type",    selector, text }
//   { id, op: "scroll",  x, y }
//   { id, op: "select",  selector, value }
//   { id, op: "wait",    millis }
//   { id, op: "extract", selector }
//   { id, op: "observe" }
//   { id, op: "screenshot" }
//   { id, op: "dom" }
//   { id, op: "close" }
//
// Responses (one JSON object per line):
//   { id, kind: "ready",       protocol_version, engine }   // startup banner, id 0
//   { id, kind: "observation", observation }
//   { id, kind: "outcome",     selector_existed, matched_element, selector?, error?, value?, observation }
//   { id, kind: "bytes",       base64 }
//   { id, kind: "text",        text }
//   { id, kind: "ack" }
//   { id, kind: "error",       message }
//
// Grounding contract: a missing selector is reported as an `outcome` with
// selector_existed=false (NOT an `error`); `error` is reserved for
// transport/backend failures.

'use strict';

const readline = require('readline');

const PROTOCOL_VERSION = 1;

let playwright = null;
try {
  // eslint-disable-next-line global-require
  playwright = require('playwright');
} catch (_err) {
  // Deferred: surfaced as an error response when `launch` is attempted, so the
  // Rust side gets a clean BrowserError instead of a process crash.
  playwright = null;
}

let browser = null;
let context = null;
let page = null;

function write(obj) {
  process.stdout.write(`${JSON.stringify(obj)}\n`);
}

// Engine name -> Playwright browser type.
function browserType(engine) {
  switch (engine) {
    case 'firefox':
      return playwright.firefox;
    case 'webkit':
      return playwright.webkit;
    case 'chromium':
    default:
      return playwright.chromium;
  }
}

async function snapshotObservation() {
  if (!page) {
    return { url: '', console: [] };
  }
  const url = page.url();
  let title = null;
  let domHtml = null;
  try {
    title = await page.title();
  } catch (_err) {
    title = null;
  }
  try {
    domHtml = await page.content();
  } catch (_err) {
    domHtml = null;
  }
  const observation = { url, console: [] };
  if (title !== null && title !== undefined) {
    observation.title = title;
  }
  if (domHtml !== null && domHtml !== undefined) {
    observation.dom_html = domHtml;
  }
  return observation;
}

// Resolve a selector to at most one element handle. Returns null when absent so
// callers can record selector_existed=false.
async function resolveOne(selector) {
  if (!page) {
    return null;
  }
  const handle = await page.$(selector);
  return handle; // ElementHandle or null
}

async function handleLaunch(cmd) {
  if (!playwright) {
    return {
      kind: 'error',
      message: "the 'playwright' package is not installed; run `npm install` in the runner directory",
    };
  }
  if (browser) {
    // Idempotent: already launched.
    return { kind: 'ack' };
  }
  if (typeof cmd.protocol_version === 'number' && cmd.protocol_version !== PROTOCOL_VERSION) {
    return {
      kind: 'error',
      message: `driver protocol version ${cmd.protocol_version} != runner ${PROTOCOL_VERSION}`,
    };
  }
  const type = browserType(cmd.engine);
  const launchOptions = { headless: cmd.headless !== false };
  if (cmd.channel) {
    launchOptions.channel = cmd.channel;
  }
  browser = await type.launch(launchOptions);
  context = await browser.newContext();
  page = await context.newPage();
  return { kind: 'ack' };
}

async function handleGoto(cmd) {
  try {
    await page.goto(cmd.url, { waitUntil: 'load' });
  } catch (err) {
    return { kind: 'error', message: `navigation failed: ${err && err.message ? err.message : err}` };
  }
  return { kind: 'observation', observation: await snapshotObservation() };
}

async function groundedOutcome(selector, action) {
  const handle = await resolveOne(selector);
  if (!handle) {
    return {
      kind: 'outcome',
      selector_existed: false,
      matched_element: false,
      selector,
      error: `selector not found: ${selector}`,
      observation: await snapshotObservation(),
    };
  }
  let value;
  try {
    value = await action(handle);
  } catch (err) {
    // The element existed but the action failed to engage it.
    await handle.dispose();
    return {
      kind: 'outcome',
      selector_existed: true,
      matched_element: false,
      selector,
      error: err && err.message ? err.message : String(err),
      observation: await snapshotObservation(),
    };
  }
  await handle.dispose();
  const outcome = {
    kind: 'outcome',
    selector_existed: true,
    matched_element: true,
    selector,
    observation: await snapshotObservation(),
  };
  if (value !== undefined) {
    outcome.value = value;
  }
  return outcome;
}

async function handleClick(cmd) {
  return groundedOutcome(cmd.selector, (handle) => handle.click());
}

async function handleType(cmd) {
  return groundedOutcome(cmd.selector, (handle) => handle.fill(cmd.text));
}

async function handleSelect(cmd) {
  return groundedOutcome(cmd.selector, (handle) => handle.selectOption(cmd.value));
}

async function handleExtract(cmd) {
  return groundedOutcome(cmd.selector, async (handle) => handle.textContent());
}

async function handleScroll(cmd) {
  await page.evaluate(
    ([x, y]) => window.scrollBy(x, y),
    [cmd.x || 0, cmd.y || 0],
  );
  return {
    kind: 'outcome',
    selector_existed: true,
    matched_element: true,
    observation: await snapshotObservation(),
  };
}

async function handleWait(cmd) {
  await page.waitForTimeout(cmd.millis || 0);
  return {
    kind: 'outcome',
    selector_existed: true,
    matched_element: true,
    observation: await snapshotObservation(),
  };
}

async function handleObserve() {
  return { kind: 'observation', observation: await snapshotObservation() };
}

async function handleScreenshot() {
  const buf = await page.screenshot({ type: 'png' });
  return { kind: 'bytes', base64: Buffer.from(buf).toString('base64') };
}

async function handleDom() {
  const text = await page.content();
  return { kind: 'text', text };
}

async function handleClose() {
  try {
    if (context) {
      await context.close();
    }
    if (browser) {
      await browser.close();
    }
  } catch (_err) {
    // Ignore teardown errors; we are closing anyway.
  }
  browser = null;
  context = null;
  page = null;
  return { kind: 'ack' };
}

async function dispatch(cmd) {
  switch (cmd.op) {
    case 'launch':
      return handleLaunch(cmd);
    case 'goto':
      return handleGoto(cmd);
    case 'click':
      return handleClick(cmd);
    case 'type':
      return handleType(cmd);
    case 'scroll':
      return handleScroll(cmd);
    case 'select':
      return handleSelect(cmd);
    case 'wait':
      return handleWait(cmd);
    case 'extract':
      return handleExtract(cmd);
    case 'observe':
      return handleObserve();
    case 'screenshot':
      return handleScreenshot();
    case 'dom':
      return handleDom();
    case 'close':
      return handleClose();
    default:
      return { kind: 'error', message: `unknown op: ${cmd.op}` };
  }
}

function main() {
  const rl = readline.createInterface({ input: process.stdin, terminal: false });
  // Serialize command handling: process one line fully before the next.
  let chain = Promise.resolve();

  // Startup banner.
  write({ id: 0, kind: 'ready', protocol_version: PROTOCOL_VERSION, engine: 'unbound' });

  rl.on('line', (line) => {
    const trimmed = line.trim();
    if (!trimmed) {
      return;
    }
    chain = chain.then(async () => {
      let cmd;
      try {
        cmd = JSON.parse(trimmed);
      } catch (err) {
        write({ id: 0, kind: 'error', message: `invalid command json: ${err.message}` });
        return;
      }
      const id = typeof cmd.id === 'number' ? cmd.id : 0;
      let payload;
      try {
        payload = await dispatch(cmd);
      } catch (err) {
        payload = { kind: 'error', message: err && err.message ? err.message : String(err) };
      }
      write({ id, ...payload });
      if (cmd.op === 'close') {
        rl.close();
      }
    });
  });

  rl.on('close', () => {
    chain.then(() => process.exit(0)).catch(() => process.exit(1));
  });
}

main();
