import Link from "next/link";

import { getSession } from "../../../lib/auth";
import ApiKeyManager from "./ApiKeyManager";

export const dynamic = "force-dynamic";

export default async function ApiKeysPage() {
  const account = await getSession();

  return (
    <main style={{ maxWidth: 640, margin: "0 auto", padding: "2rem 1.5rem" }}>
      <h1 style={{ fontSize: "1.4rem", marginBottom: "0.25rem" }}>API keys</h1>
      {account ? (
        <>
          <p style={{ opacity: 0.8, marginTop: 0 }}>
            Signed in as <strong>{account.email}</strong> · tenant{" "}
            <code>{account.tenant_id}</code>
          </p>
          <ApiKeyManager />
        </>
      ) : (
        <p>
          Please <Link href="/login">sign in</Link> to create and manage API keys.
        </p>
      )}
    </main>
  );
}
