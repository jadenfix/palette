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

## Vercel

Set `BEATER_API_BASE_URL` to the hosted Beater API URL and configure either
`BEATER_API_TOKEN` or `BEATER_API_KEY` as encrypted server-side environment
variables. The dashboard is stateless; queue workers and durable state remain
in `beaterd` or the hosted control plane.
