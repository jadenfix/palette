"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { Activity, KeyRound, Gauge, CreditCard, BookOpen, PlugZap } from "lucide-react";

import { BrandLockup } from "./BeaterMark";
import { AccountMenu } from "./AccountMenu";

type Account = { email: string; tenant_id: string };

const LINKS = [
  { href: "/", label: "Traces", icon: Activity, match: (p: string) => p === "/" },
  {
    href: "/connect",
    label: "Connect",
    icon: PlugZap,
    match: (p: string) => p.startsWith("/connect"),
  },
  {
    href: "/settings/api-keys",
    label: "API keys",
    icon: KeyRound,
    match: (p: string) => p.startsWith("/settings/api-keys"),
  },
  {
    href: "/settings/usage",
    label: "Usage",
    icon: Gauge,
    match: (p: string) => p.startsWith("/settings/usage"),
  },
  {
    href: "/settings/billing",
    label: "Billing",
    icon: CreditCard,
    match: (p: string) => p.startsWith("/settings/billing"),
  },
  {
    href: "/docs",
    label: "Docs",
    icon: BookOpen,
    match: (p: string) => p.startsWith("/docs"),
  },
];

export function AppNav({ account }: { account: Account | null }) {
  const pathname = usePathname() ?? "/";

  return (
    <header className="app-nav">
      <Link href="/" aria-label="Beater home">
        <BrandLockup />
      </Link>
      <nav className="nav-links" aria-label="Primary">
        {LINKS.map(({ href, label, icon: Icon, match }) => (
          <Link
            key={href}
            href={href}
            className="nav-link"
            aria-label={label}
            aria-current={match(pathname) ? "page" : undefined}
          >
            <Icon aria-hidden="true" />
            <span>{label}</span>
          </Link>
        ))}
      </nav>
      <div className="nav-right">
        {account ? (
          <AccountMenu account={account} />
        ) : (
          <Link href="/login" className="btn btn-primary btn-sm">
            Sign in
          </Link>
        )}
      </div>
    </header>
  );
}
