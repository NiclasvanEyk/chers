import * as Sentry from "@sentry/tanstackstart-react";
import { createRouter } from "@tanstack/react-router";
import { routeTree } from "./routeTree.gen";

export function getRouter() {
    const router = createRouter({
        routeTree,
        scrollRestoration: true,
    });

    const dsn = import.meta.env.VITE_SENTRY_DSN;
    if (!router.isServer && dsn) {
        Sentry.init({
            dsn,
            environment: import.meta.env.VITE_CHERS_ENV,
            tracesSampleRate: 1.0,
        });
    }

    return router;
}
