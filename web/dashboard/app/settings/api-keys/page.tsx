import Link from "next/link";
import { KeyRound } from "lucide-react";

import { getSession } from "../../../lib/auth";
import ApiKeyManager from "./ApiKeyManager";

export const dynamic = "force-dynamic";

export default async function ApiKeysPage() {
  const account = await getSession();

  return (
    <main className="settings">
      <div className="page-head">
        <div className="page-titles">
          <h1>API keys</h1>
          <p>
            Programmatic access for your agents and CI. Keys carry scopes and are
            bound to a project and environment.
          </p>
        </div>
        {account ? (
          <div className="page-actions">
            <span className="tag mono">{account.email}</span>
            <span className="tag tag-accent mono">tenant {account.tenant_id}</span>
          </div>
        ) : null}
      </div>

      {account ? (
        <ApiKeyManager />
      ) : (
        <div className="panel">
          <div className="empty-state">
            <span className="empty-glyph" aria-hidden="true">
              <KeyRound />
            </span>
            <strong>Sign in to manage API keys</strong>
            <p>You need an account to create and revoke keys for your tenant.</p>
            <Link href="/login" className="btn btn-primary" style={{ marginTop: 6 }}>
              Sign in
            </Link>
          </div>
        </div>
      )}
    </main>
  );
}
