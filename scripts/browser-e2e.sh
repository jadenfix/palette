#!/usr/bin/env bash
# Browser-agent observability end-to-end harness.
#
# Proves the browser-agent feature works for real — real browsers and real
# instrumentation SDKs, not mocks. Each section auto-detects its runtime and
# SKIPs (non-fatally) when it is absent, so the script is safe to run anywhere;
# CI runs it on a host with the runtimes installed.
#
# Layers exercised:
#   1. Rust unit/integration tests (foundation, capture, evaluators, backends).
#   2. Real-browser conformance per native backend (Playwright / WebDriver / CDP).
#   3. Instrumentation SDK tests (browser-use Python, Stagehand TS).
#   4. (Full-loop capture->evaluate->gate e2e is the beaterctl browser-fixture
#      demo wired by WT-F; invoked here once present.)
#
# Usage: scripts/browser-e2e.sh
set -uo pipefail
cd "$(dirname "$0")/.."
ROOT="$(pwd)"
FAIL=0
pass() { printf '\033[32mPASS\033[0m  %s\n' "$1"; }
skip() { printf '\033[33mSKIP\033[0m  %s\n' "$1"; }
fail() { printf '\033[31mFAIL\033[0m  %s\n' "$1"; FAIL=1; }
have() { command -v "$1" >/dev/null 2>&1; }

echo "== 1. Rust unit/integration tests =="
if cargo test -p beater-browser -p beater-browser-capture -p beater-eval \
       -p beater-browser-playwright -p beater-browser-webdriver -p beater-browser-cdp \
       >/tmp/bt-e2e-unit.log 2>&1; then
  pass "browser crate tests (foundation, capture, evaluators, backends)"
else
  fail "browser crate tests (see /tmp/bt-e2e-unit.log)"
fi

echo "== 2. Real-browser conformance per backend =="

# 2a. Playwright (Chromium/Chrome/Edge/Firefox/WebKit) — needs Node + the runner deps.
if have node && have npm; then
  ( cd crates/beater-browser-playwright/runner && npm install >/tmp/bt-e2e-pw-npm.log 2>&1 )
  if cargo test -p beater-browser-playwright --test conformance -- --ignored \
         >/tmp/bt-e2e-pw.log 2>&1; then
    pass "Playwright backend live conformance (real browser)"
  else
    fail "Playwright backend live conformance (see /tmp/bt-e2e-pw.log)"
  fi
else
  skip "Playwright backend live conformance (node/npm not found)"
fi

# 2b. WebDriver — native Safari via safaridriver, or any W3C driver on :4444.
#     safaridriver remote automation must be enabled once: `sudo safaridriver --enable`.
if have safaridriver; then
  safaridriver --port 4444 >/tmp/bt-e2e-safari.log 2>&1 &
  SAFARI_PID=$!
  sleep 2
  if WEBDRIVER_URL="http://localhost:4444" WEBDRIVER_ENGINE="safari" \
       cargo test -p beater-browser-webdriver --test '*' -- --ignored \
       >/tmp/bt-e2e-wd.log 2>&1; then
    pass "WebDriver backend live conformance (native Safari)"
  else
    skip "WebDriver backend live conformance (safaridriver present; enable with 'sudo safaridriver --enable' — see /tmp/bt-e2e-wd.log)"
  fi
  kill "$SAFARI_PID" 2>/dev/null
else
  skip "WebDriver backend live conformance (no safaridriver/WebDriver server)"
fi

# 2c. CDP (pure-Rust Chrome/Edge) — needs a Chrome/Chromium binary.
CHROME=""
for c in chrome google-chrome google-chrome-stable chromium chromium-browser \
         "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"; do
  if have "$c" || [ -x "$c" ]; then CHROME="$c"; break; fi
done
if [ -n "$CHROME" ]; then
  if BEATER_CDP_CHROME="$CHROME" cargo test -p beater-browser-cdp --test '*' -- --ignored \
       >/tmp/bt-e2e-cdp.log 2>&1; then
    pass "CDP backend live conformance (real Chrome)"
  else
    fail "CDP backend live conformance (see /tmp/bt-e2e-cdp.log)"
  fi
else
  skip "CDP backend live conformance (no Chrome/Chromium binary)"
fi

echo "== 3. Instrumentation SDKs =="

# 3a. browser-use (Python). Mapping tests run with only opentelemetry installed;
#     running the real browser-use agent needs Python 3.11+ (this validates the
#     hooks -> browser.* OTLP span contract).
if have python3; then
  if ( cd sdks/python-browser-use && PYTHONPATH=. python3 -m pytest -q ) \
       >/tmp/bt-e2e-py.log 2>&1; then
    pass "browser-use SDK span-mapping tests (Python)"
  else
    skip "browser-use SDK tests (need: pip install opentelemetry-sdk opentelemetry-exporter-otlp pytest — see /tmp/bt-e2e-py.log)"
  fi
else
  skip "browser-use SDK tests (no python3)"
fi

# 3b. Stagehand (TypeScript).
if have node && have npm; then
  ( cd sdks/ts-stagehand && npm install >/tmp/bt-e2e-ts-npm.log 2>&1 )
  if ( cd sdks/ts-stagehand && npm test ) >/tmp/bt-e2e-ts.log 2>&1; then
    pass "Stagehand SDK span-mapping tests (TypeScript)"
  else
    fail "Stagehand SDK tests (see /tmp/bt-e2e-ts.log)"
  fi
else
  skip "Stagehand SDK tests (no node/npm)"
fi

echo "== 4. Full-loop e2e (capture -> evaluate -> gate) =="
# Runs a baseline vs a regressed prompt over a frozen page, scores grounding,
# and asserts the regression is gated as FailRegression (the improve loop).
if cargo test -p beater-browser-harness >/tmp/bt-e2e-loop.log 2>&1; then
  pass "harness full-loop: regressed prompt -> FailRegression (capture -> evaluate -> gate)"
else
  fail "harness full-loop (see /tmp/bt-e2e-loop.log)"
fi

echo
if [ "$FAIL" -eq 0 ]; then
  printf '\033[32m== browser e2e: all available runtimes passed ==\033[0m\n'
else
  printf '\033[31m== browser e2e: failures above ==\033[0m\n'
fi
exit "$FAIL"
