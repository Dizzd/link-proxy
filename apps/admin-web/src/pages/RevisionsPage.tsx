import { useQuery } from "@tanstack/react-query";
import { config } from "../api/config";
import { useOperation } from "../hooks/useOperation";
import { ErrorMessage } from "../components/common/Feedback";
export default function RevisionsPage() {
  const active = useQuery({ queryKey: ["active"], queryFn: config.active });
  const q = useQuery({ queryKey: ["revisions"], queryFn: config.revisions });
  const op = useOperation();
  return (
    <>
      <h1>Configuration revisions</h1>
      <p>
        Active revision: {active.data?.revision ?? "Loading…"}. Publish compiles
        the full draft, commits SQLite, then activates the snapshot.
      </p>
      <button
        disabled={op.busy || !active.data}
        onClick={() =>
          void op.run(
            () => config.publish(active.data!.revision),
            "Draft published",
          )
        }
      >
        Publish draft
      </button>
      <ErrorMessage error={op.error || q.error} />
      <p role="status">{op.notice}</p>
      <div className="table-scroll">
        <table>
          <thead>
            <tr>
              <th>Version</th>
              <th>Status</th>
              <th>Created</th>
              <th>Restore</th>
            </tr>
          </thead>
          <tbody>
            {q.data?.items.map((r) => (
              <tr key={String(r.version)}>
                <td>{String(r.version)}</td>
                <td>{String(r.status)}</td>
                <td>{String(r.created_at)}</td>
                <td>
                  <button
                    className="quiet"
                    disabled={op.busy || !active.data}
                    onClick={() => {
                      if (
                        confirm(
                          `Restore revision ${r.version} as a new revision?`,
                        )
                      )
                        void op.run(
                          () =>
                            config.restore(
                              Number(r.version),
                              active.data!.revision,
                            ),
                          "Revision restored",
                        );
                    }}
                  >
                    Restore
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </>
  );
}
