import { createHash } from "node:crypto";

export const dynamic = "force-dynamic";

export async function POST(request: Request) {
  if (request.headers.get("x-beater-gate2-browser-click") !== "1") {
    return Response.json({ error: "missing browser confirmation marker" }, { status: 403 });
  }

  let payload: unknown;
  try {
    payload = await request.json();
  } catch {
    return Response.json({ error: "invalid json body" }, { status: 400 });
  }

  if (!isConfirmationRequest(payload)) {
    return Response.json({ error: "traceId and spanId are required" }, { status: 400 });
  }

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

function isConfirmationRequest(value: unknown): value is { traceId: string; spanId: string } {
  if (!value || typeof value !== "object" || Array.isArray(value)) return false;
  const record = value as Record<string, unknown>;
  return (
    typeof record.traceId === "string" &&
    /^[0-9a-f]{32}$/.test(record.traceId) &&
    typeof record.spanId === "string" &&
    /^[0-9a-f]{16}$/.test(record.spanId)
  );
}
