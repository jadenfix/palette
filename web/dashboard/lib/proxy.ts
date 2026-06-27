import { NextRequest, NextResponse } from "next/server";

import { dashboardApiBaseUrl } from "./api";

/**
 * Proxy a POST from a dashboard `/api/...` route to a backend root path,
 * forwarding the session cookie and relaying status, body, and any `Set-Cookie`.
 * Keeps the backend URL server-side.
 */
export async function proxyPost(req: NextRequest, backendPath: string): Promise<NextResponse> {
  const cookie = req.headers.get("cookie") ?? "";
  const body = await req.text();
  let upstream: Response;
  try {
    upstream = await fetch(`${dashboardApiBaseUrl()}${backendPath}`, {
      method: "POST",
      headers: { "content-type": "application/json", cookie },
      body: body.length > 0 ? body : "{}",
      cache: "no-store",
    });
  } catch {
    return NextResponse.json({ error: "upstream_unreachable" }, { status: 502 });
  }
  const text = await upstream.text();
  const res = new NextResponse(text.length > 0 ? text : null, {
    status: upstream.status,
    headers: {
      "content-type": upstream.headers.get("content-type") ?? "application/json",
    },
  });
  const setCookie = upstream.headers.get("set-cookie");
  if (setCookie) res.headers.set("set-cookie", setCookie);
  return res;
}
