import { Analytics } from "@vercel/analytics/react";
import "./globals.css";
import type { Metadata } from "next";
import { Inter } from "next/font/google";

const inter = Inter({ subsets: ["latin"] });

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
  return (
    <html lang="en">
      <body className={inter.className}>{children}</body>
      <Analytics />
    </html>
  );
}
