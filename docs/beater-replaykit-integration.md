# Beater and ReplayKit Integration

This repository keeps beater standalone: ingest remains plain OpenTelemetry, and no
ReplayKit component is required to run beater, collect traces, evaluate runs, or gate
CI.

Within the broader ecosystem, beater can absorb the deep fork, patch, replay-affected,
and diff debugging loop described by the ecosystem-level ReplayKit integration design:

- collect traces through the same OTEL boundary used by any other runtime;
- evaluate replay outcomes with beater's existing scorer and gate contracts;
- keep cross-project coupling at documented trace and artifact boundaries.

The canonical ecosystem design lives in the ecosystem repository:
https://github.com/jadenfix/ecosystem/blob/main/docs/beater-replaykit-integration.md
