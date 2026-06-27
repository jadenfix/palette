import { NextRequest, NextResponse } from "next/server";

import { proxyPost } from "../../../../lib/proxy";

export const dynamic = "force-dynamic";

export function POST(req: NextRequest): Promise<NextResponse> {
  return proxyPost(req, "/auth/api-keys");
}
