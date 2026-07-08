import { AppNav } from "../../components/AppNav";
import { getSession } from "../../lib/auth";
import { ConnectClient } from "./ConnectClient";

export const dynamic = "force-dynamic";

export default async function ConnectPage() {
  const account = await getSession();

  return (
    <>
      <AppNav account={account} />
      <main className="settings connect-page">
        <div className="page-head">
          <div className="page-titles">
            <h1>Connect MCP clients</h1>
            <p>
              Connect Claude, Claude Code, Cursor, ChatGPT, Codex, and other remote MCP clients with
              OAuth-first login and scoped fallback keys.
            </p>
          </div>
          <div className="page-actions">
            {account ? (
              <>
                <span className="tag mono">{account.email}</span>
                <span className="tag tag-accent mono">tenant {account.tenant_id}</span>
              </>
            ) : (
              <a className="btn btn-primary btn-sm" href="/login?return_to=/connect">
                Sign in
              </a>
            )}
          </div>
        </div>
        <ConnectClient signedIn={Boolean(account)} />
      </main>
    </>
  );
}
