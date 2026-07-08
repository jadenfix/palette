import { NextRequest } from "next/server";

import { proxyOAuth } from "../../../lib/oauth-proxy";

export const dynamic = "force-dynamic";

export async function GET(req: NextRequest) {
  return proxyOAuth(req, {
    method: "GET",
    backendPath: "/.well-known/oauth-protected-resource",
    rewriteMetadata: "protected-resource",
  });
}
