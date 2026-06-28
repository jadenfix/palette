import Link from "next/link";
import { Check, Sparkles, CreditCard, Gauge, CircleCheck, CircleDashed } from "lucide-react";

import { getSession } from "../../../lib/auth";

export const dynamic = "force-dynamic";

const LOCAL_FEATURES = [
  "Unlimited traces, spans, datasets, and evals",
  "Full trace inspection, replay, and CI gating",
  "Your data never leaves your infrastructure",
  "Apache-2.0 — free forever",
];

const HOSTED_FEATURES = [
  "Autoscaling managed ingest",
  "SSO, RBAC, and audit-grade access controls",
  "Team workspaces and shared dashboards",
  "Usage-based billing with spend controls",
];

export default async function BillingPage() {
  const account = await getSession();

  if (!account) {
    return (
      <main className="settings">
        <div className="page-head">
          <div className="page-titles">
            <h1>Billing</h1>
            <p>Your plan, payment method, and invoices.</p>
          </div>
        </div>
        <div className="panel">
          <div className="empty-state">
            <span className="empty-glyph" aria-hidden="true">
              <CreditCard />
            </span>
            <strong>Sign in to view billing</strong>
            <p>Billing is scoped to your tenant.</p>
            <Link href="/login" className="btn btn-primary" style={{ marginTop: 6 }}>
              Sign in
            </Link>
          </div>
        </div>
      </main>
    );
  }

  return (
    <main className="settings">
      <div className="page-head">
        <div className="page-titles">
          <h1>Billing</h1>
          <p>
            Beater is local-first and free to self-host. Managed hosting with
            usage-based billing is coming soon.
          </p>
        </div>
        <div className="page-actions">
          <Link href="/settings/usage" className="btn btn-sm">
            <Gauge aria-hidden="true" /> View usage
          </Link>
        </div>
      </div>

      <div className="stack">
        <div className="statgrid" style={{ gridTemplateColumns: "repeat(auto-fit, minmax(260px, 1fr))" }}>
          <section className="panel" style={{ borderColor: "var(--accent-line)" }}>
            <div className="panel-head">
              <div className="panel-titles">
                <h2>Local · self-hosted</h2>
                <p>$0 — your infrastructure</p>
              </div>
              <span className="tag tag-success">
                <CircleCheck aria-hidden="true" width={13} height={13} /> Active
              </span>
            </div>
            <div className="panel-body">
              <ul style={{ display: "grid", gap: 10, listStyle: "none", margin: 0, padding: 0 }}>
                {LOCAL_FEATURES.map((f) => (
                  <li key={f} style={{ display: "flex", gap: 9, alignItems: "flex-start", fontSize: 13 }}>
                    <Check aria-hidden="true" width={16} height={16} color="var(--accent)" style={{ flex: "0 0 auto", marginTop: 1 }} />
                    {f}
                  </li>
                ))}
              </ul>
            </div>
            <div className="panel-foot">
              <span>Running on your beaterd</span>
              <span className="tag mono">tenant {account.tenant_id}</span>
            </div>
          </section>

          <section className="panel">
            <div className="panel-head">
              <div className="panel-titles">
                <h2>
                  Hosted · managed{" "}
                  <Sparkles aria-hidden="true" width={14} height={14} color="var(--accent)" />
                </h2>
                <p>Usage-based — coming soon</p>
              </div>
              <span className="tag">
                <CircleDashed aria-hidden="true" width={13} height={13} /> Coming soon
              </span>
            </div>
            <div className="panel-body">
              <ul style={{ display: "grid", gap: 10, listStyle: "none", margin: 0, padding: 0 }}>
                {HOSTED_FEATURES.map((f) => (
                  <li key={f} style={{ display: "flex", gap: 9, alignItems: "flex-start", fontSize: 13, color: "var(--ink-soft)" }}>
                    <Check aria-hidden="true" width={16} height={16} color="var(--faint)" style={{ flex: "0 0 auto", marginTop: 1 }} />
                    {f}
                  </li>
                ))}
              </ul>
            </div>
            <div className="panel-foot">
              <span>Want early access?</span>
              <a className="btn-link" href="https://github.com/jadenfix/beater" target="_blank" rel="noreferrer">
                Follow on GitHub
              </a>
            </div>
          </section>
        </div>

        <section className="panel">
          <div className="panel-head">
            <div className="panel-titles">
              <h2>Payment &amp; invoices</h2>
              <p>Nothing to bill while you self-host.</p>
            </div>
          </div>
          <div className="panel-body">
            <div className="alert">
              <CreditCard aria-hidden="true" />
              <span>
                No payment method or invoices on the Local plan — it&apos;s free. A card and
                receipts appear here when you move to managed hosting.
              </span>
            </div>
          </div>
        </section>
      </div>
    </main>
  );
}
