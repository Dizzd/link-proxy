import { ChangePasswordForm } from "../features/auth/ChangePasswordForm";
import { useQuery } from "@tanstack/react-query";
import { request, send } from "../api/client";
import { useOperation } from "../hooks/useOperation";
import { useCacheStatus } from "../hooks/useCacheStatus";
import { ErrorMessage, JsonView } from "../components/common/Feedback";
export default function SettingsPage() {
  const q = useCacheStatus();
  const database = useQuery({
    queryKey: ["database"],
    queryFn: () => request("/system/database/status"),
    refetchInterval: 5000,
  });
  const op = useOperation();
  return (
    <>
      <h1>Settings & cache</h1>
      <section className="panel">
        <h2>Database</h2>
        <p>
          The configured database stores configuration, administrator accounts
          and sessions.
        </p>
        <ErrorMessage error={database.error} />
        <JsonView value={database.data} />
      </section>
      <ChangePasswordForm />
      <section className="panel">
        <h2>Configuration synchronization</h2>
        <p>
          Requests always use local memory snapshots. Redis optionally caches
          configuration and synchronizes revisions. The sidebar shows the last
          connection state reported by periodic synchronization. CACHE_MODE
          selects the mode at server startup; Redis tuning and network
          allowlists are managed in config.toml.
        </p>
        <ErrorMessage error={q.error || op.error} />
        <JsonView value={q.data} />
        <button
          disabled={op.busy}
          onClick={() =>
            void op.run(
              () => send("/system/cache/reconcile", "POST"),
              "Reconciliation completed",
            )
          }
        >
          Reconcile from database
        </button>
        <p role="status">{op.notice}</p>
      </section>
    </>
  );
}
