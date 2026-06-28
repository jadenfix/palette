#!/usr/bin/env python3
"""Verify the bus backend adapter docs keep the durable-bus contracts explicit."""

from __future__ import annotations

import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "docs/bus-backends.md"

TOKENS = {
    "existing backends and planned adapters": [
        "`DurableBus` trait is built",
        "`InMemoryBus` and\n> `SqliteDurableBus` implement it",
        "NATS JetStream and Kafka adapters are **[planned]**",
        "Do **not** add broker deps to `beater-bus` itself",
    ],
    "core delivery guarantees": [
        "**At-least-once delivery**",
        "**Idempotent publish**",
        "**Partition isolation**",
        "**Poison-message isolation**",
        "**DLQ replayability**",
        "**Backpressure**",
    ],
    "nats and kafka idempotency caveats": [
        "JetStream server-side dedup alone does NOT satisfy Beater's permanent\napplication-level idempotency contract",
        "Kafka's `enable.idempotence=true` at the producer level only prevents duplicate\nrecords caused by *producer retries within a session*",
        "Producer-side dedup store",
    ],
    "vercel queue contract": [
        "### Phase 3 — Vercel Queues adapter (hosted edge)",
        "**Target crate:** `beater-bus-vercel`",
        "edge/control-plane",
        "long-running ingest listeners",
        "hosted Rust cells",
        "not a replacement runtime for `beaterd`",
        "Poll consumer fetch",
        "Push consumers may wake a cell worker",
        "must not run long-lived drain logic inside a Vercel Function",
        "The adapter must never return messages from another scope",
        "Ack must not happen from the Vercel edge before the worker has durably accepted the job",
        "permanent application-level idempotency contract still belongs to\nthe adapter",
        "queue-provider dedup field\nis useful only as a short-window optimization",
        "No Vercel client dependency is added to `beater-bus`",
    ],
    "vercel acceptance criteria": [
        "`VercelQueuesBus` passes the same bus conformance suite",
        "fake HTTP queue fixture",
        "without depending on live Vercel infrastructure",
        "Vercel-edge enqueue path can hand off to a cell\n   worker",
        "`beater-bus-vercel`",
        "`Arc<dyn DurableBus>`",
    ],
}


def main() -> int:
    failures: list[str] = []
    try:
        text = DOC.read_text(encoding="utf-8")
    except FileNotFoundError:
        print("missing docs/bus-backends.md", file=sys.stderr)
        return 1

    for section, tokens in TOKENS.items():
        for token in tokens:
            if token not in text:
                failures.append(f"{section}: missing {token!r}")

    if failures:
        print("Bus backend docs check failed:", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1

    print("Bus backend docs preserve durable-bus adapter contracts.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
