"use client";

import { useState } from "react";

const SCOPES: { id: string; label: string }[] = [
  { id: "trace_read", label: "trace:read — read traces & spans" },
  { id: "trace_write", label: "trace:write — ingest traces" },
  { id: "dataset_write", label: "dataset:write — manage datasets" },
  { id: "eval_run", label: "eval:run — run evals/judges" },
  { id: "pii_unmask", label: "pii:unmask — reveal redacted I/O" },
  { id: "admin", label: "admin — full access (mint keys)" },
];

type CreatedKey = {
  api_key_id: string;
  secret: string;
  scopes: string[];
  project_id: string;
  environment_id: string;
};

const inputStyle = { padding: "0.5rem", borderRadius: 6, border: "1px solid #333" };
const cardStyle = {
  border: "1px solid #2a2a2a",
  borderRadius: 12,
  padding: "1.25rem",
  marginTop: "1.25rem",
  display: "flex",
  flexDirection: "column" as const,
  gap: "0.75rem",
};

export default function ApiKeyManager() {
  const [selected, setSelected] = useState<Set<string>>(new Set(["trace_read"]));
  const [project, setProject] = useState("default");
  const [environment, setEnvironment] = useState("default");
  const [created, setCreated] = useState<CreatedKey | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const [revokeId, setRevokeId] = useState("");
  const [revokeMsg, setRevokeMsg] = useState<string | null>(null);

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
          setError("API keys require the backend to run with strict auth.");
        else setError("Could not create the key. Check your selected scopes.");
        return;
      }
      setCreated((await res.json()) as CreatedKey);
    } catch {
      setError("Network error — please try again.");
    } finally {
      setBusy(false);
    }
  }

  async function revoke(event: React.FormEvent) {
    event.preventDefault();
    setRevokeMsg(null);
    try {
      const res = await fetch("/api/auth/api-keys/revoke", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ api_key_id: revokeId.trim() }),
      });
      if (res.status === 204) {
        setRevokeMsg("Key revoked.");
        setRevokeId("");
      } else if (res.status === 404) {
        setRevokeMsg("No such key in your tenant.");
      } else if (res.status === 403) {
        setRevokeMsg("That key belongs to another tenant.");
      } else {
        setRevokeMsg("Could not revoke the key.");
      }
    } catch {
      setRevokeMsg("Network error — please try again.");
    }
  }

  return (
    <>
      <form onSubmit={generate} style={cardStyle}>
        <h2 style={{ fontSize: "1.05rem", margin: 0 }}>Create a key</h2>
        <fieldset style={{ border: "none", padding: 0, margin: 0, display: "grid", gap: 6 }}>
          {SCOPES.map((s) => (
            <label key={s.id} style={{ display: "flex", gap: 8, alignItems: "center" }}>
              <input
                type="checkbox"
                checked={selected.has(s.id)}
                onChange={() => toggle(s.id)}
              />
              <span style={{ fontSize: "0.9rem" }}>{s.label}</span>
            </label>
          ))}
        </fieldset>
        <div style={{ display: "flex", gap: "0.75rem" }}>
          <label style={{ display: "flex", flexDirection: "column", gap: 4, flex: 1 }}>
            <span style={{ fontSize: "0.8rem", opacity: 0.8 }}>Project</span>
            <input value={project} onChange={(e) => setProject(e.target.value)} style={inputStyle} />
          </label>
          <label style={{ display: "flex", flexDirection: "column", gap: 4, flex: 1 }}>
            <span style={{ fontSize: "0.8rem", opacity: 0.8 }}>Environment</span>
            <input
              value={environment}
              onChange={(e) => setEnvironment(e.target.value)}
              style={inputStyle}
            />
          </label>
        </div>
        {error ? (
          <p role="alert" style={{ color: "#f87171", fontSize: "0.85rem", margin: 0 }}>
            {error}
          </p>
        ) : null}
        <button
          type="submit"
          disabled={busy || selected.size === 0}
          style={{
            padding: "0.6rem",
            borderRadius: 6,
            border: "none",
            background: "#3b82f6",
            color: "white",
            cursor: busy ? "default" : "pointer",
            opacity: busy || selected.size === 0 ? 0.7 : 1,
          }}
        >
          {busy ? "Creating…" : "Generate key"}
        </button>
      </form>

      {created ? (
        <div style={{ ...cardStyle, borderColor: "#3b82f6" }}>
          <h2 style={{ fontSize: "1.05rem", margin: 0 }}>Your new key</h2>
          <p style={{ fontSize: "0.8rem", opacity: 0.8, margin: 0 }}>
            Copy it now — it is shown only once. Scopes: {created.scopes.join(", ")} · project{" "}
            {created.project_id} · env {created.environment_id}
          </p>
          <code
            style={{
              display: "block",
              padding: "0.6rem",
              background: "#111",
              borderRadius: 6,
              wordBreak: "break-all",
              userSelect: "all",
            }}
          >
            {created.secret}
          </code>
          <p style={{ fontSize: "0.75rem", opacity: 0.7, margin: 0 }}>
            Key id: <code>{created.api_key_id}</code>
          </p>
        </div>
      ) : null}

      <form onSubmit={revoke} style={cardStyle}>
        <h2 style={{ fontSize: "1.05rem", margin: 0 }}>Revoke a key</h2>
        <label style={{ display: "flex", flexDirection: "column", gap: 4 }}>
          <span style={{ fontSize: "0.8rem", opacity: 0.8 }}>Key id</span>
          <input
            value={revokeId}
            onChange={(e) => setRevokeId(e.target.value)}
            placeholder="api key id"
            style={inputStyle}
          />
        </label>
        {revokeMsg ? (
          <p style={{ fontSize: "0.85rem", margin: 0 }}>{revokeMsg}</p>
        ) : null}
        <button
          type="submit"
          disabled={revokeId.trim().length === 0}
          style={{
            padding: "0.6rem",
            borderRadius: 6,
            border: "1px solid #555",
            background: "transparent",
            color: "#eee",
            cursor: revokeId.trim().length === 0 ? "default" : "pointer",
          }}
        >
          Revoke
        </button>
      </form>
    </>
  );
}
