"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import {
  Plug,
  ListTree,
  FlaskConical,
  ShieldCheck,
  Activity,
  Eye,
  EyeOff,
  AlertCircle,
  ArrowRight,
} from "lucide-react";

import { BrandLockup, BrandVitals } from "../../components/BeaterMark";

type Mode = "login" | "register";

const LOOP = [
  { icon: Plug, label: "Instrument any agent with stock OpenTelemetry" },
  { icon: ListTree, label: "Inspect the trace as an agent-native span tree" },
  { icon: FlaskConical, label: "Promote failures to datasets and run evals" },
  { icon: ShieldCheck, label: "Gate CI on experiment reports" },
  { icon: Activity, label: "Monitor production — same loop, end to end" },
];

function humanize(status: number, code: unknown): string {
  if (code === "email_taken") return "That email is already registered — try signing in.";
  if (code === "invalid_credentials") return "Incorrect email or password.";
  if (status === 400) return "Enter a valid email and a password of at least 8 characters.";
  if (status === 502) return "Can't reach beaterd — is it running?";
  return "Couldn't sign you in. Try again in a moment.";
}

export default function LoginPage() {
  const router = useRouter();
  const [mode, setMode] = useState<Mode>("login");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [reveal, setReveal] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const isRegister = mode === "register";

  async function submit(event: React.FormEvent) {
    event.preventDefault();
    setBusy(true);
    setError(null);
    try {
      const res = await fetch(`/api/auth/${mode}`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ email, password }),
      });
      if (!res.ok) {
        const data = (await res.json().catch(() => ({}))) as { error?: string };
        setError(humanize(res.status, data.error));
        return;
      }
      router.push("/");
      router.refresh();
    } catch {
      setError("Can't reach beaterd. Check it's running, then retry.");
    } finally {
      setBusy(false);
    }
  }

  function switchMode(next: Mode) {
    setMode(next);
    setError(null);
  }

  return (
    <main className="auth">
      <aside className="auth-brand">
        <BrandLockup size={30} />
        <div className="auth-pitch">
          <h2>Every agent run, traced, replayed, and graded.</h2>
          <p>
            Local-first observability for AI agents — the whole loop in one Rust
            binary, on your machine.
          </p>
          <div className="vitals">
            <div className="vitals-top">
              <span className="vitals-dot" aria-hidden="true" />
              agent vitals
              <span className="vitals-state">live</span>
            </div>
            <BrandVitals />
          </div>
          <div className="loop" style={{ marginTop: 22 }}>
            {LOOP.map(({ icon: Icon, label }) => (
              <div className="loop-step" key={label}>
                <span className="loop-dot" aria-hidden="true">
                  <Icon />
                </span>
                {label}
              </div>
            ))}
          </div>
        </div>
        <p className="auth-foot">
          Self-hosted by default. Your traces stay in your <code>beaterd</code>.
        </p>
      </aside>

      <section className="auth-panel">
        <div className="auth-card">
          <div className="auth-card-head">
            <h1>{isRegister ? "Create your account" : "Welcome back"}</h1>
            <p className="auth-sub">
              {isRegister
                ? "Create a tenant and your first API key."
                : "Sign in to inspect traces, run evals, and manage keys."}
            </p>
          </div>

          <div className="segmented" role="group" aria-label="Choose sign in or create account">
            <button type="button" aria-pressed={!isRegister} onClick={() => switchMode("login")}>
              Sign in
            </button>
            <button type="button" aria-pressed={isRegister} onClick={() => switchMode("register")}>
              Create account
            </button>
          </div>

          <form className="auth-form" onSubmit={submit}>
            <label className="field">
              <span className="field-label">
                Email <span className="req">*</span>
              </span>
              <input
                type="email"
                name="email"
                autoComplete="email"
                placeholder="you@company.com"
                required
                aria-invalid={error ? true : undefined}
                aria-describedby={error ? "auth-error" : undefined}
                value={email}
                onChange={(e) => setEmail(e.target.value)}
              />
            </label>

            <label className="field">
              <span className="field-label">
                Password <span className="req">*</span>
              </span>
              <div className="input-affix">
                <input
                  type={reveal ? "text" : "password"}
                  name="password"
                  autoComplete={isRegister ? "new-password" : "current-password"}
                  placeholder={isRegister ? "At least 8 characters" : "Your password"}
                  required
                  minLength={8}
                  aria-invalid={error ? true : undefined}
                  aria-describedby={error ? "auth-error" : undefined}
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                />
                <button
                  type="button"
                  className="affix-btn"
                  aria-label={reveal ? "Hide password" : "Show password"}
                  onClick={() => setReveal((v) => !v)}
                >
                  {reveal ? <EyeOff aria-hidden="true" /> : <Eye aria-hidden="true" />}
                </button>
              </div>
              {isRegister ? (
                <span className="field-hint">Use at least 8 characters.</span>
              ) : null}
            </label>

            {error ? (
              <div className="alert alert-danger" role="alert" id="auth-error">
                <AlertCircle aria-hidden="true" />
                <span>{error}</span>
              </div>
            ) : null}

            <button type="submit" className="btn btn-primary btn-lg btn-block" disabled={busy}>
              {busy ? "Working…" : isRegister ? "Create account" : "Sign in"}
              {!busy ? <ArrowRight aria-hidden="true" /> : null}
            </button>
          </form>

          <p className="auth-alt">
            {isRegister ? "Already have an account? " : "New to Beater? "}
            <button
              type="button"
              className="btn-link"
              onClick={() => switchMode(isRegister ? "login" : "register")}
            >
              {isRegister ? "Sign in" : "Create one"}
            </button>
          </p>
        </div>
      </section>
    </main>
  );
}
