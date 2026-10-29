import { useQuery } from "@tanstack/react-query";
import { config } from "../api/config";
import { ErrorMessage } from "../components/common/Feedback";
import { Simulator } from "../features/simulation/Simulator";
import { Link } from "react-router-dom";
export default function DashboardPage() {
  const q = useQuery({ queryKey: ["active"], queryFn: config.active });
  return (
    <>
      <h1>Traffic overview</h1>
      <p className="subtitle">
        Unified mappings. Immutable configuration. Live control.
      </p>
      <ErrorMessage error={q.error} />
      {q.isPending ? (
        <p role="status">Loading configuration…</p>
      ) : (
        q.data && (
          <div className="stats">
            {[
              ["Active revision", q.data.revision],
              ["Published mappings", q.data.configuration.mappings.length],
              ["Upstreams", q.data.configuration.upstreams.length],
              ["Domains", q.data.configuration.domains.length],
            ].map(([k, v]) => (
              <div className="stat" key={k}>
                <span>{k}</span>
                <strong>{v}</strong>
              </div>
            ))}
          </div>
        )
      )}
      <section className="panel">
        <h2>Configuration workflow</h2>
        <p>
          Create mappings in the draft, validate with the simulator, then
          publish one complete snapshot.
        </p>
        <Link className="button" to="/mappings/new">
          Create mapping
        </Link>{" "}
        <Link to="/revisions">Review revisions</Link>
      </section>
      <Simulator />
    </>
  );
}
