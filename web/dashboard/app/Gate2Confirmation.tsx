"use client";

import { useCallback, useEffect, useMemo, useState } from "react";

const CLICK_EVENT = "beater:gate2-span-click";

type ClickDetail = {
  traceId: string;
  spanId: string;
};

export function Gate2SpanClickTracker() {
  useEffect(() => {
    function handleClick(event: MouseEvent) {
      const target = event.target;
      if (!(target instanceof Element)) return;
      const anchor = target.closest<HTMLElement>("[data-gate2-confirm-span]");
      if (!anchor) return;
      const traceId = anchor.dataset.traceId;
      const spanId = anchor.dataset.spanId;
      if (!traceId || !spanId) return;
      sessionStorage.setItem(storageKey(traceId, spanId), "clicked");
      window.dispatchEvent(new CustomEvent<ClickDetail>(CLICK_EVENT, { detail: { traceId, spanId } }));
    }

    document.addEventListener("click", handleClick, { capture: true });
    return () => document.removeEventListener("click", handleClick, { capture: true });
  }, []);

  return null;
}

export function Gate2ConfirmationCode({
  traceId,
  spanId
}: {
  traceId: string;
  spanId: string;
}) {
  const [code, setCode] = useState<string | null>(null);
  const [status, setStatus] = useState<"hidden" | "loading" | "ready" | "error">("hidden");
  const key = useMemo(() => storageKey(traceId, spanId), [traceId, spanId]);

  const loadCode = useCallback(async () => {
    setStatus("loading");
    try {
      const response = await fetch("/api/gate2/confirm", {
        method: "POST",
        cache: "no-store",
        headers: {
          "content-type": "application/json",
          "x-beater-gate2-browser-click": "1"
        },
        body: JSON.stringify({ traceId, spanId })
      });
      if (!response.ok) throw new Error(`confirmation request failed: ${response.status}`);
      const payload = (await response.json()) as { code?: unknown };
      if (typeof payload.code !== "string" || !/^[0-9A-F]{8}$/.test(payload.code)) {
        throw new Error("confirmation response did not include an 8-character code");
      }
      setCode(payload.code);
      setStatus("ready");
    } catch {
      setCode(null);
      setStatus("error");
    }
  }, [spanId, traceId]);

  useEffect(() => {
    if (sessionStorage.getItem(key) === "clicked") {
      void loadCode();
    } else {
      setCode(null);
      setStatus("hidden");
    }

    function handleSpanClick(event: Event) {
      const detail = (event as CustomEvent<ClickDetail>).detail;
      if (detail?.traceId === traceId && detail.spanId === spanId) {
        void loadCode();
      }
    }

    window.addEventListener(CLICK_EVENT, handleSpanClick);
    return () => window.removeEventListener(CLICK_EVENT, handleSpanClick);
  }, [key, loadCode, spanId, traceId]);

  if (status === "hidden") return null;

  return (
    <div className="confirmation-code" data-confirmation-status={status}>
      <dt>Confirm</dt>
      <dd>{status === "ready" && code ? code : status === "error" ? "unavailable" : "loading"}</dd>
    </div>
  );
}

function storageKey(traceId: string, spanId: string): string {
  return `beater:gate2:clicked:${traceId}:${spanId}`;
}
