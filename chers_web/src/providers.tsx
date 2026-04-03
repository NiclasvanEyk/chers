"use client";

import { useEffect, type ReactNode } from "react";
import { initializeCredentialCleanup } from "@/lib/multiplayer";

export function AppProviders({ children }: { children: ReactNode }) {
  useEffect(() => {
    initializeCredentialCleanup();
  }, []);

  return <>{children}</>;
}
