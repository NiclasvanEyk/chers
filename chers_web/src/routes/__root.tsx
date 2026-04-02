import { Outlet, createRootRoute, HeadContent, Scripts } from "@tanstack/react-router";
import { Analytics } from "@vercel/analytics/react";
import appCss from "../globals.css?url";
import { AppProviders } from "../providers";

export const Route = createRootRoute({
  head: () => ({
    meta: [
      { charSet: "utf-8" },
      {
        name: "viewport",
        content: "width=device-width, initial-scale=1",
      },
      { title: "Chers" },
      { name: "application-name", content: "Chers" },
      { name: "description", content: "A rusty chess implementation for the terminal and web." },
      { name: "creator", content: "Niclas van Eyk" },
      { name: "keywords", content: "Chess, Rust, Webassembly" },
    ],
    links: [
      {
        rel: "stylesheet",
        href: appCss,
      },
      {
        rel: "manifest",
        href: "/manifest.json",
      },
      {
        rel: "canonical",
        href: "https://chers.niclasve.me",
      },
    ],
  }),
  component: RootLayout,
});

function RootLayout() {
  const overScrollBehaviors = "overscroll-none md:overscroll-auto";

  return (
    <html lang="en" className={overScrollBehaviors}>
      <head>
        <HeadContent />
      </head>
      <body>
        <AppProviders>
          <main className="flex min-h-screen flex-col items-center justify-center">
            <Outlet />
          </main>
        </AppProviders>
        <Analytics />
        <Scripts />
      </body>
    </html>
  );
}
