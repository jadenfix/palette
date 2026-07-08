export type OAuthMetadataKind = "authorization-server" | "protected-resource";

export type OriginRequest = {
  url: string;
  headers: {
    get(name: string): string | null;
  };
};

export function publicRequestOrigin(req: OriginRequest): string {
  const fallback = new URL(req.url);
  const forwardedHost = req.headers.get("x-forwarded-host");
  const host = forwardedHost ?? req.headers.get("host") ?? fallback.host;
  const forwardedProto = req.headers.get("x-forwarded-proto");
  const proto = forwardedProto ?? fallback.protocol.replace(/:$/, "");
  return `${proto}://${host}`;
}

export function rewriteResourceSearch(
  search: string,
  publicOrigin: string,
  backendIssuer: string,
): string {
  if (!search || publicOrigin === backendIssuer) return search;
  const params = new URLSearchParams(search.startsWith("?") ? search.slice(1) : search);
  let changed = false;
  const resources = params.getAll("resource");
  if (resources.length === 0) return search;
  params.delete("resource");
  for (const resource of resources) {
    if (resource === publicOrigin) {
      params.append("resource", backendIssuer);
      changed = true;
    } else {
      params.append("resource", resource);
    }
  }
  return changed ? `?${params.toString()}` : search;
}

export function rewriteResourceFormBody(
  body: string | undefined,
  contentType: string | null,
  publicOrigin: string,
  backendIssuer: string,
): string | undefined {
  if (!body || publicOrigin === backendIssuer) return body;
  if (!contentType?.toLowerCase().startsWith("application/x-www-form-urlencoded")) return body;
  const params = new URLSearchParams(body);
  if (params.get("resource") !== publicOrigin) return body;
  params.set("resource", backendIssuer);
  return params.toString();
}

export function rewriteHostedOAuthReferences(
  value: string,
  backendIssuer: string,
  publicOrigin: string,
): string {
  if (!value || backendIssuer === publicOrigin) return value;
  return value.split(backendIssuer).join(publicOrigin);
}

export function safeOAuthReturnPath(value: string | null, origin: string): string {
  if (!value) return "/";
  try {
    const target = new URL(value, origin);
    const oauthProxyPath = target.pathname.startsWith("/oauth/");
    const sameOriginSetupPath =
      target.origin === origin &&
      (target.pathname === "/" ||
        target.pathname === "/connect" ||
        target.pathname === "/settings/api-keys");
    const allowed = oauthProxyPath || sameOriginSetupPath;
    if (!allowed) return "/";
    return `${target.pathname}${target.search}${target.hash}`;
  } catch {
    return "/";
  }
}

export function rewriteOAuthMetadata(
  body: string,
  origin: string,
  kind: OAuthMetadataKind,
): string {
  let metadata: Record<string, unknown>;
  try {
    const parsed = JSON.parse(body);
    if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) return body;
    metadata = parsed as Record<string, unknown>;
  } catch {
    return body;
  }

  if (kind === "authorization-server") {
    metadata.issuer = origin;
    metadata.authorization_endpoint = `${origin}/oauth/authorize`;
    metadata.token_endpoint = `${origin}/oauth/token`;
    metadata.registration_endpoint = `${origin}/oauth/register`;
  } else {
    metadata.resource = origin;
    metadata.authorization_servers = [origin];
  }
  return JSON.stringify(metadata);
}
