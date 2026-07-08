import { NextRequest } from "next/server";

import { proxyMcp } from "../../lib/mcp-proxy";

export async function GET(req: NextRequest) {
  return proxyMcp(req, "GET");
}

export async function POST(req: NextRequest) {
  return proxyMcp(req, "POST");
}
