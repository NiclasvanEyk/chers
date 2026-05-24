"use client";

import { useEffect, type ReactNode } from "react";
import * as Sentry from "@sentry/tanstackstart-react";
import { initializeCredentialCleanup } from "@/lib/multiplayer";

export function AppProviders({ children }: { children: ReactNode }) {
  useEffect(() => {
    initializeCredentialCleanup();
  }, []);

  return (
    <Sentry.ErrorBoundary fallback={<p>An unexpected error occurred.</p>}>
      {children}
    </Sentry.ErrorBoundary>
  );
}
