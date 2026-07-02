# Composio connectors

Beater can broker **250+ third-party tools** (GitHub, Gmail, Slack, web search,
…) through [Composio](https://composio.dev), with managed OAuth so a user logs
in to an app **once** and every later agent run reuses the connection. This is
the substrate for the agent `tool_set` lever (ARCHITECTURE §6.1) and the RSI
loop's `ToolAdd`/`ToolRemove` changes (§21).

## How it fits the contract

The integration is REST-first so it rides the existing single-source-of-truth
pipeline (see [CLAUDE.md](../CLAUDE.md)):

```
crates/beater-composio          → typed Composio v3 client (ComposioClient trait)
crates/beater-api  /v1/connectors → 6 handlers (#[utoipa::path])
   → cargo xtask regen-spec      → OpenAPI + dashboard snapshot
   → scripts/regen-sdks.sh       → 7 SDK clients
   → beater-mcp build_tools()    → 6 MCP tools (auto, no hand-mirror)
```

Because the MCP catalog is derived from the `/v1` surface, an RSI agent session
calls Composio tools through **the same MCP path it already uses** for Beater's
own tools — no special-casing.

## Endpoints

All are tenant/project scoped. The Composio "entity" (which owns connections)
is keyed per project: `beater:{tenant_id}:{project_id}`.

| Operation | Method · Path | Scope | Purpose |
|---|---|---|---|
| `listConnectors` | `GET /v1/connectors/{t}/{p}` | trace:read | Catalog of connectable apps |
| `listConnectorTools` | `GET …/tools?toolkit=` | trace:read | Tools + input JSON Schemas |
| `getConnectorSkills` | `GET …/skills?toolkit=` | trace:read | Generated prompting scaffold |
| `connectConnector` | `POST …/connect` | admin | One-time managed-OAuth login link |
| `connectorStatus` | `GET …/status?toolkit=` | trace:read | Connection status |
| `invokeConnectorTool` | `POST …/invoke` | eval:run + connector policy | Request tool execution |

## Auth model ("login with Composio")

1. An agent needs a tool. Check `connectorStatus`.
2. **Connected** → `invokeConnectorTool` runs it silently.
3. **Not connected** → `connectConnector` returns a one-time `redirect_url`
   (`https://connect.composio.dev/link/…`). The user opens it once; Composio
   persists and refreshes the grant. They are never prompted again.

This complements `beater-oauth-server` (which authorizes callers *into* Beater)
and the BYOK `provider-secrets` pattern — it does not replace either.

## Enabling

Set `COMPOSIO_API_KEY` in the `beaterd` environment. Unset, the `/v1/connectors`
endpoints return `501 Not Implemented` and the rest of the server is unaffected.
**Never commit the key** — it is read from the environment only.

### Execution policy

`eval:run` is only the coarse Beater scope for requesting a connector tool.
Before Beater calls Composio, it fetches the tool metadata, classifies the tool
risk, and applies Beater's connector policy. Read-only tools are allowed by
default. External-write, destructive, messaging, payment, secret-access, and
unknown-risk tools return `403 Forbidden` unless explicitly allowlisted.

Configure the initial runtime allow/deny lists with comma-separated tool slugs:

```bash
BEATER_CONNECTOR_ALLOW_TOOLS=GITHUB_CREATE_AN_ISSUE,SLACK_SEND_MESSAGE
BEATER_CONNECTOR_DENY_TOOLS=GITHUB_DELETE_REPOSITORY
```

Deny entries win over allow entries. Policy decisions are audited when the audit
store is configured; the audit attributes include the risk class and an argument
hash, not raw connector arguments.

## RSI integration — the three seams

1. **Discovery + execution.** The `/v1/connectors` MCP tools let a running agent
   find tools (`listConnectorTools`, with schemas), connect them, and execute
   permitted tools (`invokeConnectorTool`).
2. **The `tool_set` lever.** `beater_composio::skill::tool_definition_json()`
   emits the `tools.json` entry the RSI loop's `apply_change`/`ToolAdd` writes
   into an agent repo — schema-and-skill-complete, not a bare slug.
3. **Prompting scaffold.** `skill::skill_card` / `skills_doc` turn Composio
   metadata into the "skills.md" an agent splices into its system prompt
   (what the tool does, when to use it, the argument contract, the exact
   `invokeConnectorTool` call). Served live by `getConnectorSkills`.

## Keeping up with Composio (auto-shipping updates)

Composio continuously adds and updates endpoints. `scripts/sync-composio-catalog.sh`
snapshots the live catalog into `crates/beater-composio/catalog/toolkits.json`
(public metadata only, deterministic, diff-friendly):

```bash
COMPOSIO_API_KEY=ak_... scripts/sync-composio-catalog.sh           # refresh
COMPOSIO_API_KEY=ak_... scripts/sync-composio-catalog.sh --check   # CI: fail on drift
```

A non-empty diff == new capabilities to expose. Run it on a schedule (or before
a release) and commit, so new Composio apps surface as a reviewable change
instead of being missed.

## Testing

- `cargo test -p beater-composio` — client mapping, skill scaffold, RSI entry.
  Live wire tests run only when `COMPOSIO_API_KEY` is set (else they skip).
- `cargo test -p beater-api --test connectors` — full router→handler→trait
  wiring with an in-memory fake, including connector execution policy, the RSI
  add-then-execute flow, and the 501-when-unconfigured path.
