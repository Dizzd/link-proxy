import { ChangePasswordForm } from "../features/auth/ChangePasswordForm";
import { useQuery } from "@tanstack/react-query";
import { request, send } from "../api/client";
import { useOperation } from "../hooks/useOperation";
import { ErrorMessage, JsonView } from "../components/common/Feedback";
export default function SettingsPage() {
  const q = useQuery({
    queryKey: ["cache"],
    queryFn: () => request("/system/cache/status"),
    refetchInterval: 5000,
  });
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
          The data plane always uses its local immutable snapshot. Listener,
          network allowlist and Redis settings are managed in server TOML.
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
