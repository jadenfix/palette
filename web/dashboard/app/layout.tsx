import "./globals.css";
import "./beater-ui.css";
import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Beater — observability for AI agents",
  description:
    "Trace, replay, and grade every AI-agent run. Local-first observability, eval, and CI gating in one Rust binary."
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
