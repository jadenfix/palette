import { NextRequest, NextResponse } from "next/server";

import { dashboardApiBaseUrl } from "./api";
import {
  publicRequestOrigin,
  rewriteHostedOAuthReferences,
  rewriteOAuthMetadata,
  rewriteResourceFormBody,
  rewriteResourceSearch,
  type OAuthMetadataKind,
} from "./oauth-routing";

type ProxyOptions = {
  method: "GET" | "POST";
  backendPath: string;
  body?: string;
  rewriteMetadata?: OAuthMetadataKind;
};

const RESPONSE_HEADERS = [
  "cache-control",
  "content-type",
  "location",
  "pragma",
  "vary",
  "www-authenticate",
];

/**
 * Hosted dashboard OAuth proxy.
 *
 * In a split deployment the browser session cookie is scoped to the dashboard
 * origin, while beaterd is often on a private/API origin. Proxying OAuth through
 * the dashboard keeps `/oauth/authorize` on the same origin as `/login`, so the
 * session cookie reaches the backend authorization server.
 */
export async function proxyOAuth(req: NextRequest, options: ProxyOptions): Promise<NextResponse> {
  const url = new URL(req.url);
  const publicOrigin = publicRequestOrigin(req);
  const backendIssuer = await backendOAuthIssuer();
  const cookie = req.headers.get("cookie") ?? "";
  const contentType = req.headers.get("content-type");
  const authorization = req.headers.get("authorization");
  const headers: Record<string, string> = {};
  if (cookie) headers.cookie = cookie;
  if (contentType) headers["content-type"] = contentType;
  if (authorization) headers.authorization = authorization;

  const upstreamSearch = rewriteResourceSearch(url.search, publicOrigin, backendIssuer);
  const upstreamBody = rewriteResourceFormBody(
    options.body,
    contentType,
    publicOrigin,
    backendIssuer,
  );

  let upstream: Response;
  try {
    upstream = await fetch(`${dashboardApiBaseUrl()}${options.backendPath}${upstreamSearch}`, {
      method: options.method,
      headers,
      body: upstreamBody,
      cache: "no-store",
      redirect: "manual",
    });
  } catch {
    return NextResponse.json({ error: "upstream_unreachable" }, { status: 502 });
  }

  let text = await upstream.text();
  const rewroteMetadata = Boolean(options.rewriteMetadata && upstream.ok);
  if (options.rewriteMetadata && upstream.ok) {
    text = rewriteOAuthMetadata(text, publicOrigin, options.rewriteMetadata);
  } else {
    text = rewriteHostedOAuthReferences(text, backendIssuer, publicOrigin);
  }
  const res = new NextResponse(text.length > 0 ? text : null, {
    status: upstream.status,
  });
  for (const name of RESPONSE_HEADERS) {
    const value = upstream.headers.get(name);
    if (value) {
      res.headers.set(name, rewriteHostedOAuthReferences(value, backendIssuer, publicOrigin));
    }
  }
  if (rewroteMetadata) {
    res.headers.set("content-type", "application/json");
  }
  copySetCookie(upstream.headers, res.headers);
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

function copySetCookie(from: Headers, to: Headers) {
  const getSetCookie = (from as Headers & { getSetCookie?: () => string[] }).getSetCookie;
  if (typeof getSetCookie === "function") {
    for (const value of getSetCookie.call(from)) {
      to.append("set-cookie", value);
    }
    return;
  }
  const value = from.get("set-cookie");
  if (value) to.append("set-cookie", value);
}
