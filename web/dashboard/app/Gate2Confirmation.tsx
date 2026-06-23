"use client";

import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { isBrowserClickProof, type BrowserClickProof } from "../lib/gate2-click-proof";
import { GATE2_CONFIRMATION_CODE } from "../lib/gate2-confirmation-contract";

const CLICK_EVENT = "beater:gate2-span-click";

type ClickDetail = {
  traceId: string;
  spanId: string;
  click: BrowserClickProof;
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
      const click = clickProof(event);
      if (!click) return;
      sessionStorage.setItem(storageKey(traceId, spanId), JSON.stringify(click));
      window.dispatchEvent(
        new CustomEvent<ClickDetail>(CLICK_EVENT, { detail: { traceId, spanId, click } })
      );
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
  const requestedNonce = useRef<string | null>(null);

  const loadCode = useCallback(async (click: BrowserClickProof) => {
    if (requestedNonce.current === click.nonce) return;
    requestedNonce.current = click.nonce;
    setStatus("loading");
    try {
      const response = await fetch("/api/gate2/confirm", {
        method: "POST",
        cache: "no-store",
        headers: {
          "content-type": "application/json"
        },
        body: JSON.stringify({ traceId, spanId, click })
      });
      if (!response.ok) throw new Error(`confirmation request failed: ${response.status}`);
      const payload = (await response.json()) as { code?: unknown };
      if (typeof payload.code !== "string" || !GATE2_CONFIRMATION_CODE.test(payload.code)) {
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
    const storedClick = readStoredClick(key);
    if (storedClick) {
      void loadCode(storedClick);
    } else {
      setCode(null);
      setStatus("hidden");
    }

    function handleSpanClick(event: Event) {
      const detail = (event as CustomEvent<ClickDetail>).detail;
      if (detail?.traceId === traceId && detail.spanId === spanId) {
        void loadCode(detail.click);
      }
    }

    window.addEventListener(CLICK_EVENT, handleSpanClick);
    return () => window.removeEventListener(CLICK_EVENT, handleSpanClick);
  }, [key, loadCode, spanId, traceId]);

  return (
    <div className="confirmation-code" data-confirmation-status={status}>
      <dt>Confirm</dt>
      <dd aria-live="polite">
        {status === "ready" && code
          ? code
          : status === "error"
            ? "unavailable"
            : status === "loading"
              ? "loading"
              : "pending"}
      </dd>
    </div>
  );
}

function storageKey(traceId: string, spanId: string): string {
  return `beater:gate2:clicked:${traceId}:${spanId}`;
}

function clickProof(event: MouseEvent): BrowserClickProof | null {
  if (!event.isTrusted) return null;
  if (event.button !== 0 || event.detail < 1) return null;
  return {
    nonce: randomHex(16),
    capturedAtMs: Date.now(),
    isTrusted: true,
    button: event.button,
    detail: event.detail,
    clientX: event.clientX,
    clientY: event.clientY,
    screenX: event.screenX,
    screenY: event.screenY
  };
}

function randomHex(byteLength: number): string {
  const bytes = new Uint8Array(byteLength);
  crypto.getRandomValues(bytes);
  return Array.from(bytes, (byte) => byte.toString(16).padStart(2, "0")).join("");
}

function readStoredClick(key: string): BrowserClickProof | null {
  try {
    const value = sessionStorage.getItem(key);
    if (!value) return null;
    const parsed = JSON.parse(value) as unknown;
    if (isBrowserClickProof(parsed)) return parsed;
    sessionStorage.removeItem(key);
  } catch {
    sessionStorage.removeItem(key);
  }
  return null;
}
