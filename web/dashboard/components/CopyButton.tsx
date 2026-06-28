"use client";

import { useState } from "react";
import { Check, Copy } from "lucide-react";

/**
 * One-click copy with an inline "Copied" confirmation. Works for API keys,
 * snippets, and identifiers — the primitives an agent (or its author) needs to
 * grab fast.
 */
export function CopyButton({
  value,
  label = "Copy",
  className = "copy-btn",
}: {
  value: string;
  label?: string;
  className?: string;
}) {
  const [copied, setCopied] = useState(false);

  async function copy() {
    try {
      await navigator.clipboard.writeText(value);
    } catch {
      // Fallback for non-secure contexts.
      const el = document.createElement("textarea");
      el.value = value;
      el.style.position = "fixed";
      el.style.opacity = "0";
      document.body.appendChild(el);
      el.select();
      try {
        document.execCommand("copy");
      } catch {
        /* ignore */
      }
      document.body.removeChild(el);
    }
    setCopied(true);
    window.setTimeout(() => setCopied(false), 1600);
  }

  return (
    <button
      type="button"
      className={className}
      data-copied={copied ? "true" : undefined}
      onClick={copy}
      aria-label={copied ? "Copied" : label}
    >
      {copied ? <Check aria-hidden="true" /> : <Copy aria-hidden="true" />}
      {copied ? "Copied" : label}
    </button>
  );
}

/** Dark copy-field: a value plus a copy button, sized for keys and URLs. */
export function CopyField({ value, wrap = false }: { value: string; wrap?: boolean }) {
  return (
    <div className={wrap ? "copyfield wrap" : "copyfield"}>
      <code>{value}</code>
      <CopyButton value={value} />
    </div>
  );
}
