import { getSession } from "../../lib/auth";
import { AppNav } from "../../components/AppNav";

export const dynamic = "force-dynamic";

/**
 * Shared shell for every account surface (API keys, usage, billing). Provides
 * the global nav + account menu so the product feels like one place, and so a
 * signed-in user can move between settings without typing URLs.
 */
export default async function SettingsLayout({ children }: { children: React.ReactNode }) {
  const account = await getSession();
  return (
    <>
      <AppNav account={account} />
      {children}
    </>
  );
}
