import { NextRequest, NextResponse } from "next/server";

import { dashboardApiBaseUrl } from "./api";
import { publicRequestOrigin, rewriteHostedOAuthReferences } from "./oauth-routing";

const FORWARD_REQUEST_HEADERS = [
  "accept",
  "authorization",
  "content-type",
  "cookie",
  "last-event-id",
  "mcp-protocol-version",
  "mcp-session-id",
  "x-beater-api-key",
  "x-beater-environment-id",
  "x-beater-project-id",
];

const FORWARD_RESPONSE_HEADERS = [
  "allow",
  "cache-control",
  "content-type",
  "mcp-session-id",
  "pragma",
  "vary",
  "www-authenticate",
];

export async function proxyMcp(req: NextRequest, method: "GET" | "POST"): Promise<NextResponse> {
  const url = new URL(req.url);
  const publicOrigin = publicRequestOrigin(req);
  const backendIssuer = await backendOAuthIssuer();
  const headers: Record<string, string> = {};
  for (const name of FORWARD_REQUEST_HEADERS) {
    const value = req.headers.get(name);
    if (value) headers[name] = value;
  }

  let upstream: Response;
  try {
    upstream = await fetch(`${dashboardApiBaseUrl()}/mcp${url.search}`, {
      method,
      headers,
      body: method === "POST" ? await req.text() : undefined,
      cache: "no-store",
      redirect: "manual",
    });
  } catch {
    return NextResponse.json({ error: "upstream_unreachable" }, { status: 502 });
  }

  const contentType = upstream.headers.get("content-type") ?? "";
  const isEventStream = contentType.toLowerCase().startsWith("text/event-stream");
  let body: BodyInit | null;
  if (isEventStream) {
    body = upstream.body;
  } else {
    const text = rewriteHostedOAuthReferences(await upstream.text(), backendIssuer, publicOrigin);
    body = text.length > 0 ? text : null;
  }
  const res = new NextResponse(body, { status: upstream.status });
  for (const name of FORWARD_RESPONSE_HEADERS) {
    const value = upstream.headers.get(name);
    if (value) {
      res.headers.set(name, rewriteHostedOAuthReferences(value, backendIssuer, publicOrigin));
    }
  }
  return res;
}

async function backendOAuthIssuer(): Promise<string> {
  try {
    const res = await fetch(`${dashboardApiBaseUrl()}/.well-known/oauth-authorization-server`, {
      cache: "no-store",
    });
    if (!res.ok) return dashboardApiBaseUrl();
    const metadata = await res.json();
    return typeof metadata?.issuer === "string" ? metadata.issuer : dashboardApiBaseUrl();
  } catch {
    return dashboardApiBaseUrl();
  }
}
