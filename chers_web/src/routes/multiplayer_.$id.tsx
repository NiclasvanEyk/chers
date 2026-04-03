import { createFileRoute } from "@tanstack/react-router";
import { MatchClient } from "@/components/MatchClient";

function MatchPage() {
  const { id } = Route.useParams();
  return <MatchClient id={id} />;
}

export const Route = createFileRoute("/multiplayer_/$id")({
  component: MatchPage,
});
