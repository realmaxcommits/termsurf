import { createFileRoute } from "@tanstack/react-router";
import { CommitLog } from "../components/CommitLog";
import commitsData from "../../data/commits.json";

export const Route = createFileRoute("/commits")({
  component: CommitsPage,
});

function CommitsPage() {
  return <CommitLog commits={commitsData.commits} />;
}
