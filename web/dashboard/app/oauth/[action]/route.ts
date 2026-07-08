import { NextRequest, NextResponse } from "next/server";

import { proxyOAuth } from "../../../lib/oauth-proxy";

const GET_ACTIONS = new Set(["authorize"]);
const POST_ACTIONS = new Set(["register", "token"]);

export const dynamic = "force-dynamic";

export async function GET(
  req: NextRequest,
  ctx: { params: Promise<{ action: string }> },
): Promise<NextResponse> {
  const { action } = await ctx.params;
  if (!GET_ACTIONS.has(action)) {
    return NextResponse.json({ error: "not_found" }, { status: 404 });
  }
  return proxyOAuth(req, { method: "GET", backendPath: `/oauth/${action}` });
}

export async function POST(
  req: NextRequest,
  ctx: { params: Promise<{ action: string }> },
): Promise<NextResponse> {
  const { action } = await ctx.params;
  if (!POST_ACTIONS.has(action)) {
    return NextResponse.json({ error: "not_found" }, { status: 404 });
  }
  return proxyOAuth(req, {
    method: "POST",
    backendPath: `/oauth/${action}`,
    body: await req.text(),
  });
}
