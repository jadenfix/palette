"use client";

import { useEffect, useRef, useState } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import { ChevronDown, KeyRound, LogOut, Gauge, CreditCard } from "lucide-react";

type Account = { email: string; tenant_id: string };

function initials(email: string): string {
  const name = email.split("@")[0] ?? email;
  const parts = name.split(/[.\-_+]/).filter(Boolean);
  const letters = (parts.length >= 2 ? parts[0][0] + parts[1][0] : name.slice(0, 2)) || "?";
  return letters.toUpperCase();
}

export function AccountMenu({ account }: { account: Account }) {
  const router = useRouter();
  const [open, setOpen] = useState(false);
  const [busy, setBusy] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    function onDoc(event: MouseEvent) {
      if (ref.current && !ref.current.contains(event.target as Node)) setOpen(false);
    }
    function onKey(event: KeyboardEvent) {
      if (event.key === "Escape") setOpen(false);
    }
    document.addEventListener("mousedown", onDoc);
    document.addEventListener("keydown", onKey);
    return () => {
      document.removeEventListener("mousedown", onDoc);
      document.removeEventListener("keydown", onKey);
    };
  }, [open]);

  async function signOut() {
    setBusy(true);
    try {
      await fetch("/api/auth/logout", { method: "POST" });
    } catch {
      /* best effort */
    }
    router.push("/login");
    router.refresh();
  }

  return (
    <div className="account" ref={ref}>
      <button
        type="button"
        className="account-chip"
        aria-haspopup="true"
        aria-expanded={open}
        aria-label={`Account: ${account.email}`}
        onClick={() => setOpen((v) => !v)}
      >
        <span className="avatar" aria-hidden="true">
          {initials(account.email)}
        </span>
        <ChevronDown aria-hidden="true" width={14} height={14} />
      </button>
      {open ? (
        // A disclosure of plain links/buttons — not role="menu", which would
        // demand roving arrow-key focus management we don't implement. Tab works.
        <div className="menu">
          <div className="menu-head">
            <strong>{account.email}</strong>
            <small>tenant {account.tenant_id}</small>
          </div>
          <div className="menu-sep" />
          <Link href="/settings/api-keys" onClick={() => setOpen(false)}>
            <KeyRound aria-hidden="true" /> API keys
          </Link>
          <Link href="/settings/usage" onClick={() => setOpen(false)}>
            <Gauge aria-hidden="true" /> Usage
          </Link>
          <Link href="/settings/billing" onClick={() => setOpen(false)}>
            <CreditCard aria-hidden="true" /> Billing
          </Link>
          <div className="menu-sep" />
          <button type="button" className="danger" onClick={signOut} disabled={busy}>
            <LogOut aria-hidden="true" /> {busy ? "Signing out…" : "Sign out"}
          </button>
        </div>
      ) : null}
    </div>
  );
}
