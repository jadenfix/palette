"use client";

import { useState } from "react";
import {
  KeyRound,
  ShieldCheck,
  Trash2,
  AlertCircle,
  CheckCircle2,
  TriangleAlert,
  Plus,
} from "lucide-react";

import { CopyButton, CopyField } from "../../components/CopyButton";

type Scope = { id: string; name: string; desc: string };

const SCOPES: Scope[] = [
  { id: "trace_read", name: "trace:read", desc: "Read traces, spans, and span I/O." },
  { id: "trace_write", name: "trace:write", desc: "Ingest traces via OTLP and the native API." },
  { id: "dataset_write", name: "dataset:write", desc: "Create and version datasets." },
  { id: "eval_run", name: "eval:run", desc: "Run evals and judges over candidates." },
  { id: "pii_unmask", name: "pii:unmask", desc: "Reveal redacted I/O — every use is audited." },
  { id: "admin", name: "admin", desc: "Full access, including minting new keys." },
];

const DEFAULT_SCOPES = ["trace_read"];

type CreatedKey = {
  api_key_id: string;
  secret: string;
  scopes: string[];
  project_id: string;
  environment_id: string;
};

type SessionKey = {
  api_key_id: string;
  scopes: string[];
  project_id: string;
  environment_id: string;
  prefix: string;
};

function maskSecret(secret: string): string {
  return `${secret.slice(0, 11)}…${secret.slice(-4)}`;
}

export default function ApiKeyManager() {
  const [selected, setSelected] = useState<Set<string>>(new Set(DEFAULT_SCOPES));
  const [project, setProject] = useState("default");
  const [environment, setEnvironment] = useState("default");
  const [created, setCreated] = useState<CreatedKey | null>(null);
  const [sessionKeys, setSessionKeys] = useState<SessionKey[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [revoked, setRevoked] = useState<string | null>(null);

  function toggle(id: string) {
    setSelected((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }

  async function generate(event: React.FormEvent) {
    event.preventDefault();
    setBusy(true);
    setError(null);
    setCreated(null);
    setRevoked(null);
    try {
      const res = await fetch("/api/auth/api-keys", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          scopes: Array.from(selected),
          project_id: project,
          environment_id: environment,
        }),
      });
      if (!res.ok) {
        const data = (await res.json().catch(() => ({}))) as { error?: string };
        if (res.status === 401) setError("Your session expired — please sign in again.");
        else if (data.error === "api_keys_unavailable")
          setError("API keys require the backend to run with strict auth enabled.");
        else setError("Could not create the key. Check your selected scopes and try again.");
        return;
      }
      const key = (await res.json()) as CreatedKey;
      setCreated(key);
      setSessionKeys((prev) => [
        {
          api_key_id: key.api_key_id,
          scopes: key.scopes,
          project_id: key.project_id,
          environment_id: key.environment_id,
          prefix: maskSecret(key.secret),
        },
        ...prev,
      ]);
    } catch {
      setError("Network error — please try again.");
    } finally {
      setBusy(false);
    }
  }

  async function revoke(apiKeyId: string) {
    setRevoked(null);
    try {
      const res = await fetch("/api/auth/api-keys/revoke", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ api_key_id: apiKeyId }),
      });
      if (res.status === 204) {
        setSessionKeys((prev) => prev.filter((k) => k.api_key_id !== apiKeyId));
        if (created?.api_key_id === apiKeyId) setCreated(null);
        setRevoked("Key revoked.");
      } else if (res.status === 404) {
        setRevoked("No such key in your tenant.");
      } else if (res.status === 403) {
        setRevoked("That key belongs to another tenant.");
      } else {
        setRevoked("Could not revoke the key.");
      }
    } catch {
      setRevoked("Network error — please try again.");
    }
  }

  const snippet = created
    ? [
        `export BEATER_API_KEY="${created.secret}"`,
        `export BEATER_API_BASE_URL="http://127.0.0.1:8080"`,
        ``,
        `curl -H "x-beater-api-key: $BEATER_API_KEY" \\`,
        `  "$BEATER_API_BASE_URL/v1/traces/$BEATER_TENANT"`,
      ].join("\n")
    : "";

  return (
    <div className="stack">
      {created ? (
        <section className="panel" style={{ borderColor: "var(--accent-line)" }}>
          <div className="panel-head">
            <div className="panel-titles">
              <h2>Your new API key</h2>
              <p>Copy it now — for your security, the secret is shown only once.</p>
            </div>
            <span className="tag tag-success">
              <CheckCircle2 aria-hidden="true" width={13} height={13} /> Created
            </span>
          </div>
          <div className="panel-body">
            <CopyField value={created.secret} wrap />
            <div className="alert alert-warn">
              <TriangleAlert aria-hidden="true" />
              <span>
                Store it in a secret manager. If you lose it, revoke this key and create a new one.
              </span>
            </div>
            <div className="field">
              <span className="field-label">Use it from an agent</span>
              <pre className="codeblock">{snippet}</pre>
            </div>
            <div className="key-meta">
              <span>
                key id <code>{created.api_key_id}</code>
              </span>
              <span>
                project <code>{created.project_id}</code>
              </span>
              <span>
                env <code>{created.environment_id}</code>
              </span>
            </div>
          </div>
          <div className="panel-foot">
            <span>Done copying?</span>
            <CopyButton value={snippet} label="Copy snippet" className="btn btn-sm" />
          </div>
        </section>
      ) : null}

      <form className="panel" onSubmit={generate}>
        <div className="panel-head">
          <div className="panel-titles">
            <h2>Create a key</h2>
            <p>Scope it to exactly what your agent needs. Least privilege by default.</p>
          </div>
        </div>
        <div className="panel-body">
          <div className="field">
            <span className="field-label">
              Scopes <span className="opt">{selected.size} selected</span>
            </span>
            <div className="option-grid">
              {SCOPES.map((s) => (
                <label className="option-card" key={s.id} data-selected={selected.has(s.id)}>
                  <input
                    type="checkbox"
                    checked={selected.has(s.id)}
                    onChange={() => toggle(s.id)}
                  />
                  <span className="option-name">
                    <code>{s.name}</code>
                  </span>
                  <span className="option-desc">{s.desc}</span>
                </label>
              ))}
            </div>
          </div>

          <div className="form-row">
            <label className="field">
              <span className="field-label">Project</span>
              <input value={project} onChange={(e) => setProject(e.target.value)} />
            </label>
            <label className="field">
              <span className="field-label">Environment</span>
              <input value={environment} onChange={(e) => setEnvironment(e.target.value)} />
            </label>
          </div>

          {error ? (
            <div className="alert alert-danger" role="alert">
              <AlertCircle aria-hidden="true" />
              <span>{error}</span>
            </div>
          ) : null}
        </div>
        <div className="panel-foot">
          <span className="key-meta">
            <ShieldCheck aria-hidden="true" width={13} height={13} /> Keys are hashed at rest; the
            secret never leaves your browser after this.
          </span>
          <button
            type="submit"
            className="btn btn-primary"
            aria-busy={busy}
            disabled={busy || selected.size === 0}
          >
            <Plus aria-hidden="true" />
            {busy ? "Creating…" : "Create key"}
          </button>
        </div>
      </form>

      <section className="panel">
        <div className="panel-head">
          <div className="panel-titles">
            <h2>Keys created this session</h2>
            <p>Revoke instantly. Older keys can be revoked by id below.</p>
          </div>
        </div>
        <div className="panel-body">
          {revoked ? (
            <div className="alert alert-info" role="status">
              <CheckCircle2 aria-hidden="true" />
              <span>{revoked}</span>
            </div>
          ) : null}
          {sessionKeys.length === 0 ? (
            <div className="empty-state">
              <span className="empty-glyph" aria-hidden="true">
                <KeyRound />
              </span>
              <strong>No keys created here yet</strong>
              <p>Keys you create in this session show up here. Older keys can be revoked by id below.</p>
            </div>
          ) : (
            <div className="keylist">
              {sessionKeys.map((k) => (
                <div className="key-row" key={k.api_key_id}>
                  <div className="key-id">
                    <strong>{k.prefix}</strong>
                    <code>{k.api_key_id}</code>
                  </div>
                  <button
                    type="button"
                    className="btn btn-danger btn-sm"
                    onClick={() => revoke(k.api_key_id)}
                  >
                    <Trash2 aria-hidden="true" /> Revoke
                  </button>
                  <div className="key-scopes">
                    <span className="tag mono">{k.project_id}</span>
                    <span className="tag mono">{k.environment_id}</span>
                    {k.scopes.map((s) => (
                      <span className="tag tag-accent" key={s}>
                        {s}
                      </span>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          )}
          <RevokeById onRevoke={revoke} />
        </div>
      </section>
    </div>
  );
}

function RevokeById({ onRevoke }: { onRevoke: (id: string) => void }) {
  const [id, setId] = useState("");
  return (
    <form
      className="form-row"
      style={{ alignItems: "end", gridTemplateColumns: "minmax(0, 1fr) auto" }}
      onSubmit={(e) => {
        e.preventDefault();
        if (id.trim()) onRevoke(id.trim());
        setId("");
      }}
    >
      <label className="field">
        <span className="field-label">Revoke by key id</span>
        <input value={id} onChange={(e) => setId(e.target.value)} placeholder="api key id" />
      </label>
      <button type="submit" className="btn btn-danger" disabled={id.trim().length === 0}>
        <Trash2 aria-hidden="true" /> Revoke
      </button>
    </form>
  );
}
