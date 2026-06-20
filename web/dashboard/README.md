# Beater Dashboard

Next.js dashboard for the Beater trace-debugging vertical slice.

## Local Run

Start `beaterd` in one terminal:

```bash
cargo run -q -p beaterd -- --data-dir /tmp/beaterd-ui
```

Send an OTLP smoke trace:

```bash
cargo run -q -p beaterctl -- smoke --http-url http://127.0.0.1:8080 --otlp-grpc-url http://127.0.0.1:4317
```

Start the dashboard:

```bash
cd web/dashboard
npm install
NEXT_PUBLIC_BEATER_API_BASE_URL=http://127.0.0.1:8080 npm run dev
```

Open `http://127.0.0.1:3000/?tenant=demo&project=demo&environment=local`.
The filter bar supports status, canonical span kind, RFC3339 start window,
model, release, cost micros, and latency milliseconds.

For a clean local clone-to-browser proof from the repository root:

```bash
scripts/gate2-proof.sh
```

For the Docker Compose path from the repository root:

```bash
scripts/smoke-compose.sh
```

For the clean-machine stopwatch path with the literal five-line OTEL snippet:

```bash
scripts/gate2-compose-stopwatch.sh
```

That script runs the five-line snippet from the host through a temporary Python
venv against `localhost:4317`, then waits for the trace to be visible in the
dashboard. It pulls prebuilt GHCR images through `docker-compose.prebuilt.yml`
by default; set `BEATER_GATE2_LOCAL_BUILD=1` to build images from source for
development checks.

For a strict-auth `beaterd`, set one server-only credential before starting the
dashboard:

```bash
BEATER_API_BASE_URL=http://127.0.0.1:8080 \
BEATER_API_TOKEN=bt_... \
npm run dev
```

`BEATER_API_TOKEN` is sent as `Authorization: Bearer ...`. `BEATER_API_KEY` is
also supported and is sent as `x-beater-api-key`. The dashboard derives
`x-beater-project-id` and `x-beater-environment-id` from the selected scope.

## Generated API Client

The dashboard read client is generated from the Rust-owned OpenAPI surface:

```bash
cargo run -q -p beater-api --example dump_openapi > web/dashboard/openapi/beater-read-api.json
cd web/dashboard
npm run generate:api
```

Do not hand-edit `lib/generated/api-types.ts`.

To fail on OpenAPI/client drift from the repository root:

```bash
scripts/check-openapi-drift.sh
```

## Browser E2E

With a live dashboard already pointed at a Beater API that has the stock OTLP
Python smoke trace:

```bash
PLAYWRIGHT_BASE_URL=http://127.0.0.1:3000 npm run test:e2e
```

The Gate 2 proof script installs Chromium and runs this test unless
`BEATER_GATE2_SKIP_BROWSER=1` is set.

To also record the browser demo artifact under `docs/demos/`:

```bash
BEATER_GATE2_RECORD_DEMO=1 scripts/gate2-proof.sh
```

## Vercel

Set `BEATER_API_BASE_URL` to the hosted Beater API URL and configure either
`BEATER_API_TOKEN` or `BEATER_API_KEY` as encrypted server-side environment
variables. The dashboard is stateless; queue workers and durable state remain
in `beaterd` or the hosted control plane.
