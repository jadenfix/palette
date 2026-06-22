import { createHash } from "node:crypto";
import type { NextRequest } from "next/server";

export const dynamic = "force-dynamic";

const SESSION_COOKIE = "beater_gate2_session";
const CLICK_MAX_AGE_MS = 60_000;
const USED_NONCES = new Map<string, number>();

export async function POST(request: NextRequest) {
  const sessionId = request.cookies.get(SESSION_COOKIE)?.value;
  if (!sessionId || !/^[0-9a-f]{32}$/.test(sessionId)) {
    return Response.json({ error: "missing browser session" }, { status: 403 });
  }
  const origin = request.headers.get("origin");
  if (!origin || !allowedOrigins(request).has(origin)) {
    return Response.json({ error: "invalid request origin" }, { status: 403 });
  }
  if (
    request.headers.get("sec-fetch-site") !== "same-origin" ||
    request.headers.get("sec-fetch-mode") !== "cors" ||
    request.headers.get("sec-fetch-dest") !== "empty"
  ) {
    return Response.json({ error: "missing browser fetch metadata" }, { status: 403 });
  }

  let payload: unknown;
  try {
    payload = await request.json();
  } catch {
    return Response.json({ error: "invalid json body" }, { status: 400 });
  }

  if (!isConfirmationRequest(payload)) {
    return Response.json({ error: "traceId, spanId, and browser click proof are required" }, { status: 400 });
  }

  const now = Date.now();
  pruneUsedNonces(now);
  if (now - payload.click.capturedAtMs > CLICK_MAX_AGE_MS || payload.click.capturedAtMs - now > 5_000) {
    return Response.json({ error: "browser click proof expired" }, { status: 403 });
  }
  const nonceKey = createHash("sha256")
    .update(`${sessionId}:${payload.traceId}:${payload.spanId}:${payload.click.nonce}`)
    .digest("hex");
  if (USED_NONCES.has(nonceKey)) {
    return Response.json({ error: "browser click proof was already used" }, { status: 409 });
  }
  USED_NONCES.set(nonceKey, now);

  const salt = process.env.BEATER_GATE2_CONFIRMATION_SALT ?? "";
  const code = createHash("sha256")
    .update(`gate2:${salt}:${payload.traceId}:${payload.spanId}`)
    .digest("hex")
    .slice(0, 8)
    .toUpperCase();

  return Response.json(
    { code },
    {
      headers: {
        "cache-control": "no-store"
      }
    }
  );
}

type ConfirmationRequest = {
  traceId: string;
  spanId: string;
  click: {
    nonce: string;
    capturedAtMs: number;
    isTrusted: true;
    button: number;
    detail: number;
    clientX: number;
    clientY: number;
    screenX: number;
    screenY: number;
  };
};

function isConfirmationRequest(value: unknown): value is ConfirmationRequest {
  if (!value || typeof value !== "object" || Array.isArray(value)) return false;
  const record = value as Record<string, unknown>;
  const click = record.click;
  return (
    typeof record.traceId === "string" &&
    /^[0-9a-f]{32}$/.test(record.traceId) &&
    typeof record.spanId === "string" &&
    /^[0-9a-f]{16}$/.test(record.spanId) &&
    isClickProof(click)
  );
}

function isClickProof(value: unknown): value is ConfirmationRequest["click"] {
  if (!value || typeof value !== "object" || Array.isArray(value)) return false;
  const record = value as Record<string, unknown>;
  return (
    typeof record.nonce === "string" &&
    /^[0-9a-f]{32}$/.test(record.nonce) &&
    Number.isInteger(record.capturedAtMs) &&
    record.isTrusted === true &&
    finiteNumber(record.button) &&
    finiteNumber(record.detail) &&
    finiteNumber(record.clientX) &&
    finiteNumber(record.clientY) &&
    finiteNumber(record.screenX) &&
    finiteNumber(record.screenY)
  );
}

function finiteNumber(value: unknown): value is number {
  return typeof value === "number" && Number.isFinite(value);
}

function allowedOrigins(request: NextRequest): Set<string> {
  const origins = new Set([new URL(request.url).origin]);
  const host = request.headers.get("host");
  if (host) {
    origins.add(`http://${host}`);
    origins.add(`https://${host}`);
  }
  return origins;
}

function pruneUsedNonces(now: number) {
  for (const [key, usedAt] of USED_NONCES) {
    if (now - usedAt > CLICK_MAX_AGE_MS) USED_NONCES.delete(key);
  }
}
