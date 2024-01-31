import { Analytics } from "@vercel/analytics/react";
import "./globals.css";
import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Chers",
  applicationName: "Chers",
  description: "A rusty chess implementation for the terminal and web.",
  manifest: "manifest.json",
  creator: "Niclas van Eyk",
  keywords: "Chess, Rust, Webassembly",
  alternates: {
    canonical: "https://chers.niclasve.me",
  },
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  // Makes this feel more native on mobile, but still like a regular website on
  // desktop.
  const overScrollBehaviors = "overscroll-none md:overscroll-auto";

  return (
    <html lang="en" className={overScrollBehaviors}>
      <body>{children}</body>
      <Analytics />
    </html>
  );
}
