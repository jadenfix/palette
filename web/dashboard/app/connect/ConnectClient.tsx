"use client";

import { useCallback, useEffect, useMemo, useState } from "react";
import {
  AlertTriangle,
  Bot,
  Braces,
  CheckCircle2,
  KeyRound,
  Link2,
  LockKeyhole,
  MonitorCog,
  RefreshCw,
  ShieldCheck,
  TerminalSquare,
} from "lucide-react";

import { CopyButton, CopyField } from "../../components/CopyButton";

type ClientTarget = {
  id: string;
  name: string;
  icon: typeof Bot;
  note: string;
};

type ReadinessStatus = "checking" | "ok" | "warn" | "error";

type ReadinessCheck = {
  id: string;
  label: string;
  detail: string;
  status: ReadinessStatus;
};

const CLIENTS: ClientTarget[] = [
  {
    id: "claude",
    name: "Claude / Claude Code",
    icon: Bot,
    note: "Use the remote MCP server URL. OAuth starts automatically when the client connects.",
  },
  {
    id: "cursor",
    name: "Cursor",
    icon: MonitorCog,
    note: "Add Beater as a remote HTTP MCP server. Cursor should discover OAuth from the endpoint.",
  },
  {
    id: "chatgpt",
    name: "ChatGPT",
    icon: Braces,
    note: "Use the hosted MCP URL in connector setup. Tool scopes are advertised per operation.",
  },
  {
    id: "openai",
    name: "OpenAI API / Agents",
    icon: TerminalSquare,
    note: "Attach Beater as a hosted MCP tool with server_url and per-call approvals.",
  },
];

const INITIAL_READINESS_CHECKS: ReadinessCheck[] = [
  {
    id: "resource",
    label: "Protected resource metadata",
    detail: "Waiting to verify hosted resource discovery.",
    status: "checking",
  },
  {
    id: "authorization",
    label: "Authorization server metadata",
    detail: "Waiting to verify login, token, and registration endpoints.",
    status: "checking",
  },
  {
    id: "challenge",
    label: "MCP OAuth challenge",
    detail: "Waiting to verify unauthenticated /mcp discovery.",
    status: "checking",
  },
];

const PERMISSION_SETS = [
  {
    name: "Trace reader",
    scopes: ["mcp:invoke", "trace:read"],
    use: "Inspect traces, spans, and span I/O.",
  },
  {
    name: "Reviewer",
    scopes: ["mcp:invoke", "trace:read", "dataset:write"],
    use: "Promote failures to datasets and submit review annotations.",
  },
  {
    name: "Evaluator",
    scopes: ["mcp:invoke", "trace:read", "dataset:write", "eval:run"],
    use: "Run evals and calibration flows from an agent client.",
  },
  {
    name: "Admin",
    scopes: ["mcp:invoke", "admin"],
    use: "Manage provider secrets, usage, queues, and administrative MCP tools.",
  },
];

function currentOrigin() {
  if (typeof window === "undefined") return "https://app.palette.dev";
  return window.location.origin;
}

function claudeConfig(mcpUrl: string) {
  return JSON.stringify(
    {
      mcpServers: {
        beater: {
          type: "http",
          url: mcpUrl,
        },
      },
    },
    null,
    2,
  );
}

function cursorConfig(mcpUrl: string) {
  return JSON.stringify(
    {
      mcpServers: {
        beater: {
          url: mcpUrl,
          transport: "http",
        },
      },
    },
    null,
    2,
  );
}

function openAiToolConfig(mcpUrl: string) {
  return JSON.stringify(
    {
      type: "mcp",
      server_label: "beater",
      server_url: mcpUrl,
      require_approval: "always",
    },
    null,
    2,
  );
}

function apiKeyFallback(origin: string) {
  return [
    `export BEATER_API_BASE_URL="${origin}"`,
    `export BEATER_API_KEY="bt_..."`,
    `export BEATER_TENANT="your-tenant-id"`,
    `export BEATER_PROJECT="default"`,
    `export BEATER_ENVIRONMENT="default"`,
    ``,
    `curl -X POST "$BEATER_API_BASE_URL/mcp" \\`,
    `  -H "content-type: application/json" \\`,
    `  -H "x-beater-api-key: $BEATER_API_KEY" \\`,
    `  -H "x-beater-project-id: $BEATER_PROJECT" \\`,
    `  -H "x-beater-environment-id: $BEATER_ENVIRONMENT" \\`,
    `  --data "{\\"jsonrpc\\":\\"2.0\\",\\"id\\":1,\\"method\\":\\"tools/call\\",\\"params\\":{\\"name\\":\\"listTraces\\",\\"arguments\\":{\\"tenant_id\\":\\"$BEATER_TENANT\\"}}}"`,
  ].join("\n");
}

function readinessLabel(status: ReadinessStatus) {
  if (status === "ok") return "Ready";
  if (status === "error") return "Needs attention";
  if (status === "warn") return "Needs backend";
  return "Checking";
}

function readinessTagClass(status: ReadinessStatus) {
  if (status === "ok") return "tag-success";
  if (status === "error") return "tag-danger";
  if (status === "warn") return "tag-warn";
  return "tag-accent";
}

function checkRank(status: ReadinessStatus) {
  if (status === "error") return 3;
  if (status === "warn") return 2;
  if (status === "checking") return 1;
  return 0;
}

function overallStatus(results: ReadinessCheck[]): ReadinessStatus {
  return results.reduce<ReadinessStatus>(
    (current, check) => (checkRank(check.status) > checkRank(current) ? check.status : current),
    "ok",
  );
}

function stringArray(value: unknown): string[] {
  return Array.isArray(value) ? value.filter((item): item is string => typeof item === "string") : [];
}

async function fetchJsonObject(url: string): Promise<{
  ok: boolean;
  status: number;
  data: Record<string, unknown> | null;
}> {
  const response = await fetch(url, {
    cache: "no-store",
    credentials: "same-origin",
    headers: { accept: "application/json" },
  });
  let data: Record<string, unknown> | null = null;
  try {
    const parsed: unknown = await response.json();
    if (parsed && typeof parsed === "object" && !Array.isArray(parsed)) {
      data = parsed as Record<string, unknown>;
    }
  } catch {
    data = null;
  }
  return { ok: response.ok, status: response.status, data };
}

function backendReachabilityDetail(status: number) {
  if (status === 502) return "Dashboard route is live, but the OAuth/MCP backend is unreachable.";
  return `Endpoint returned HTTP ${status}.`;
}

function metadataUrl(value: unknown): string {
  return typeof value === "string" ? value : "";
}

function hostedEndpointOk(value: unknown, expected: string): boolean {
  return metadataUrl(value) === expected;
}

function expectedResourceMetadataUrl(baseOrigin: string) {
  return `${baseOrigin}/.well-known/oauth-protected-resource`;
}

export function ConnectClient({ signedIn }: { signedIn: boolean }) {
  const [origin, setOrigin] = useState("https://app.palette.dev");
  const [readiness, setReadiness] = useState<ReadinessCheck[]>(INITIAL_READINESS_CHECKS);
  const [readinessStatus, setReadinessStatus] = useState<ReadinessStatus>("checking");
  const [checkingReadiness, setCheckingReadiness] = useState(false);

  const runReadiness = useCallback(async (baseOrigin: string) => {
    const protectedResourceUrl = `${baseOrigin}/.well-known/oauth-protected-resource`;
    const authServerUrl = `${baseOrigin}/.well-known/oauth-authorization-server`;
    const mcpUrl = `${baseOrigin}/mcp`;

    setCheckingReadiness(true);
    setReadiness(INITIAL_READINESS_CHECKS);
    setReadinessStatus("checking");

    const results: ReadinessCheck[] = [];
    try {
      const resource = await fetchJsonObject(protectedResourceUrl);
      const authorizationServers = stringArray(resource.data?.authorization_servers);
      const resourceName = typeof resource.data?.resource === "string" ? resource.data.resource : "";
      const resourceOk =
        resource.ok && resourceName === baseOrigin && authorizationServers.includes(baseOrigin);
      results.push({
        id: "resource",
        label: "Protected resource metadata",
        status: resourceOk ? "ok" : resource.status === 502 ? "warn" : "error",
        detail: resourceOk
          ? "Advertises this hosted dashboard origin and its authorization server."
          : !resource.ok
            ? backendReachabilityDetail(resource.status)
            : "Metadata loaded, but resource or authorization_servers does not match this origin.",
      });
    } catch {
      results.push({
        id: "resource",
        label: "Protected resource metadata",
        status: "error",
        detail: "Could not reach protected resource metadata from this browser.",
      });
    }

    try {
      const auth = await fetchJsonObject(authServerUrl);
      const scopes = stringArray(auth.data?.scopes_supported);
      const hasEndpoints =
        hostedEndpointOk(auth.data?.issuer, baseOrigin) &&
        hostedEndpointOk(auth.data?.authorization_endpoint, `${baseOrigin}/oauth/authorize`) &&
        hostedEndpointOk(auth.data?.token_endpoint, `${baseOrigin}/oauth/token`) &&
        hostedEndpointOk(auth.data?.registration_endpoint, `${baseOrigin}/oauth/register`);
      const authOk = auth.ok && hasEndpoints && scopes.includes("mcp:invoke");
      results.push({
        id: "authorization",
        label: "Authorization server metadata",
        status: authOk ? "ok" : auth.status === 502 ? "warn" : "error",
        detail: authOk
          ? "Exposes authorize, token, dynamic registration, and mcp:invoke scope."
          : !auth.ok
            ? backendReachabilityDetail(auth.status)
            : "Metadata loaded, but hosted OAuth endpoints or mcp:invoke scope are missing.",
      });
    } catch {
      results.push({
        id: "authorization",
        label: "Authorization server metadata",
        status: "error",
        detail: "Could not reach authorization server metadata from this browser.",
      });
    }

    try {
      const challengeResponse = await fetch(mcpUrl, {
        cache: "no-store",
        credentials: "same-origin",
        headers: { accept: "application/json, text/event-stream" },
      });
      const challenge = challengeResponse.headers.get("www-authenticate") ?? "";
      const expectedResourceMetadata = expectedResourceMetadataUrl(baseOrigin);
      const hasOAuthChallenge =
        challengeResponse.status === 401 &&
        /bearer/i.test(challenge) &&
        /resource_metadata/i.test(challenge) &&
        challenge.includes(`resource_metadata="${expectedResourceMetadata}"`);
      results.push({
        id: "challenge",
        label: "MCP OAuth challenge",
        status: hasOAuthChallenge ? "ok" : challengeResponse.status === 502 ? "warn" : "error",
        detail: hasOAuthChallenge
          ? "/mcp returns a 401 Bearer challenge with resource_metadata for client login."
          : backendReachabilityDetail(challengeResponse.status),
      });
    } catch {
      results.push({
        id: "challenge",
        label: "MCP OAuth challenge",
        status: "error",
        detail: "Could not reach /mcp from this browser.",
      });
    }

    setReadiness(results);
    setReadinessStatus(overallStatus(results));
    setCheckingReadiness(false);
  }, []);

  useEffect(() => {
    const nextOrigin = currentOrigin();
    setOrigin(nextOrigin);
    void runReadiness(nextOrigin);
  }, [runReadiness]);

  const mcpUrl = `${origin}/mcp`;
  const protectedResourceUrl = `${origin}/.well-known/oauth-protected-resource`;
  const authServerUrl = `${origin}/.well-known/oauth-authorization-server`;
  const configs = useMemo(
    () => ({
      claude: claudeConfig(mcpUrl),
      cursor: cursorConfig(mcpUrl),
      openai: openAiToolConfig(mcpUrl),
      chatgpt: mcpUrl,
      fallback: apiKeyFallback(origin),
    }),
    [mcpUrl, origin],
  );

  return (
    <div className="connect-grid">
      <section className="panel connect-primary" aria-labelledby="connect-title">
        <div className="panel-head">
          <div className="panel-titles">
            <h2 id="connect-title">Hosted MCP endpoint</h2>
            <p>OAuth is the default path. API keys remain available when a client cannot launch OAuth.</p>
          </div>
          <span className={`tag ${readinessTagClass(readinessStatus)}`}>
            {readinessStatus === "ok" ? (
              <CheckCircle2 aria-hidden="true" width={13} height={13} />
            ) : readinessStatus === "checking" ? (
              <RefreshCw aria-hidden="true" width={13} height={13} />
            ) : (
              <AlertTriangle aria-hidden="true" width={13} height={13} />
            )}
            {readinessLabel(readinessStatus)}
          </span>
        </div>
        <div className="panel-body">
          <div className="field">
            <span className="field-label">Remote MCP URL</span>
            <CopyField value={mcpUrl} />
          </div>
          <div className="connect-endpoints" aria-label="OAuth discovery endpoints">
            <div>
              <span>Protected resource</span>
              <code>{protectedResourceUrl}</code>
            </div>
            <div>
              <span>Authorization server</span>
              <code>{authServerUrl}</code>
            </div>
          </div>
          <div className="alert alert-info">
            <LockKeyhole aria-hidden="true" />
            <span>
              Clients discover login from <code>/mcp</code>, request scoped consent, and send a bearer
              token on protected tool calls.
            </span>
          </div>
        </div>
      </section>

      <section className="panel connect-readiness" aria-labelledby="readiness-title">
        <div className="panel-head">
          <div className="panel-titles">
            <h2 id="readiness-title">Live OAuth check</h2>
            <p>Run from this browser against the same hosted URLs that MCP clients discover.</p>
          </div>
          <button
            className="btn btn-sm"
            type="button"
            onClick={() => void runReadiness(origin)}
            disabled={checkingReadiness}
          >
            <RefreshCw aria-hidden="true" />
            Run check
          </button>
        </div>
        <div className="panel-body readiness-list">
          {readiness.map((check) => (
            <div className="readiness-row" data-status={check.status} key={check.id}>
              <div className="readiness-icon" aria-hidden="true">
                {check.status === "ok" ? (
                  <CheckCircle2 />
                ) : check.status === "checking" ? (
                  <RefreshCw />
                ) : (
                  <AlertTriangle />
                )}
              </div>
              <div>
                <h3>{check.label}</h3>
                <p>{check.detail}</p>
              </div>
            </div>
          ))}
        </div>
        <div className="panel-foot">
          <span>
            A warning usually means the dashboard is deployed but beaterd or its OAuth proxy is not reachable.
          </span>
        </div>
      </section>

      <section className="panel" aria-labelledby="client-configs-title">
        <div className="panel-head">
          <div className="panel-titles">
            <h2 id="client-configs-title">Client configs</h2>
            <p>Use the hosted URL directly; the endpoint advertises OAuth and per-tool scopes.</p>
          </div>
        </div>
        <div className="panel-body connect-clients">
          {CLIENTS.map(({ id, name, icon: Icon, note }) => {
            const value =
              id === "cursor"
                ? configs.cursor
                : id === "chatgpt"
                  ? configs.chatgpt
                  : id === "openai"
                    ? configs.openai
                    : configs.claude;
            return (
              <article className="connect-client" key={id}>
                <div className="connect-client-head">
                  <span className="connect-client-icon" aria-hidden="true">
                    <Icon />
                  </span>
                  <div>
                    <h3>{name}</h3>
                    <p>{note}</p>
                  </div>
                </div>
                {id === "chatgpt" ? (
                  <CopyField value={value} />
                ) : (
                  <pre className="codeblock">{value}</pre>
                )}
                <CopyButton value={value} label="Copy config" className="btn btn-sm" />
              </article>
            );
          })}
          <div className="alert alert-warn">
            <TerminalSquare aria-hidden="true" />
            <span>
              Codex compatibility depends on the surface you use: hosted OpenAI model calls use the
              MCP tool object above; local clients that cannot run remote HTTP OAuth should use the
              scoped API-key fallback.
            </span>
          </div>
        </div>
      </section>

      <section className="panel" aria-labelledby="permissions-title">
        <div className="panel-head">
          <div className="panel-titles">
            <h2 id="permissions-title">Delegated permissions</h2>
            <p>Start narrow. The client asks for scopes, and Beater enforces them on every MCP call.</p>
          </div>
          <span className="tag tag-accent">
            <ShieldCheck aria-hidden="true" width={13} height={13} /> Least privilege
          </span>
        </div>
        <div className="panel-body permission-list">
          {PERMISSION_SETS.map((preset) => (
            <article className="permission-row" key={preset.name}>
              <div>
                <h3>{preset.name}</h3>
                <p>{preset.use}</p>
              </div>
              <div className="permission-scopes" aria-label={`${preset.name} scopes`}>
                {preset.scopes.map((scope) => (
                  <code key={scope}>{scope}</code>
                ))}
              </div>
            </article>
          ))}
        </div>
      </section>

      <section className="panel" aria-labelledby="fallback-title">
        <div className="panel-head">
          <div className="panel-titles">
            <h2 id="fallback-title">API-key fallback</h2>
            <p>Use this only for clients or automation that cannot complete OAuth.</p>
          </div>
          <span className="tag">
            <KeyRound aria-hidden="true" width={13} height={13} /> Scoped key
          </span>
        </div>
        <div className="panel-body">
          {signedIn ? (
            <a className="btn btn-primary" href="/settings/api-keys">
              <KeyRound aria-hidden="true" />
              Create scoped key
            </a>
          ) : (
            <a className="btn btn-primary" href="/login?return_to=/settings/api-keys">
              <KeyRound aria-hidden="true" />
              Sign in for keys
            </a>
          )}
          <pre className="codeblock">{configs.fallback}</pre>
        </div>
        <div className="panel-foot">
          <span>Fallback calls must include project and environment headers.</span>
          <CopyButton value={configs.fallback} label="Copy fallback" className="btn btn-sm" />
        </div>
      </section>

      <section className="connect-checklist" aria-label="Connection checks">
        <div>
          <Link2 aria-hidden="true" />
          <span>/mcp returns an OAuth challenge before protected calls run.</span>
        </div>
        <div>
          <LockKeyhole aria-hidden="true" />
          <span>Bearer tokens bind tenant, project, environment, and scopes.</span>
        </div>
        <div>
          <TerminalSquare aria-hidden="true" />
          <span>API-key fallback is scoped and explicit when OAuth is unavailable.</span>
        </div>
      </section>
    </div>
  );
}
