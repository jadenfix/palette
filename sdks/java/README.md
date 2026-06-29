# Beater Java SDK

Ergonomic, OpenTelemetry-native agent observability for Beater. This is the
**Layer 2** (hand-written) SDK: span helpers built on the OpenTelemetry Java SDK
that emit Beater-conformant spans over OTLP/HTTP.

## Onboarding order

Beater's default onboarding remains the zero-code OTLP path: point an existing
OpenTelemetry/OpenInference exporter at Beater with environment variables and no
Beater SDK code. Use this Java SDK when a service wants in-process helpers for
span boundaries, Beater semantic-convention constants, and short-lived process
flush behavior while still exporting standard OTLP/HTTP.

## Quickstart (5 lines)

```java
import ai.beater.sdk.*;

Beater.init(BeaterConfig.fromEnv());                       // reads BEATER_* env vars
String answer = Beater.observe("handle", SemConv.AGENT_RUN, () -> {
    Beater.setInput(query);                                // input.value
    String out = runAgent(query);
    Beater.setOutput(out);                                 // output.value
    return out;
});
Beater.flush();                                            // before a short-lived process exits
```

`BeaterConfig.fromEnv()` reads `BEATER_BASE_URL` (default `http://127.0.0.1:8080`),
`BEATER_TENANT_ID`/`BEATER_PROJECT_ID` (default `demo`), `BEATER_ENVIRONMENT_ID`
(default `local`), `BEATER_API_KEY`, `BEATER_SERVICE_NAME`, and `BEATER_RELEASE_ID`.
Spans export to `{base}/v1/otlp/{tenant}/{project}/{environment}/v1/traces`.
Span kinds and attribute keys live in one place — `SemConv` — mirroring the
server's OTLP normalizer and the Python/TypeScript SDKs.
