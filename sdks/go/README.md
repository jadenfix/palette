# Beater Go SDK

Ergonomic OpenTelemetry tracing helpers for Beater (`package beater`).

Beater's default onboarding path is still zero-code OTLP export: point an
existing OpenTelemetry exporter at Beater with `BEATER_*` env vars, with no
Beater SDK and no code edits. Use this Go SDK when you want an idiomatic helper
layer after that standards-first path.

```go
ctx := context.Background()
shutdown, _ := beater.Init(ctx, beater.ConfigFromEnv()) // reads BEATER_* env
defer shutdown(ctx)

beater.Observe(ctx, "call_model", beater.KindLLMCall, func(ctx context.Context) error {
    beater.SetInput(ctx, "hello")
    beater.SetOutput(ctx, "world")
    return nil // span kind, beater.seq, release id, status, errors handled for you
})
```

Config falls back to env vars (`BEATER_BASE_URL`, `BEATER_TENANT_ID`, ...) with
local defaults (`http://127.0.0.1:8080`, `demo/demo/local`). See `example/main.go`
for an `agent.run -> agent.plan -> llm.call` trace.
