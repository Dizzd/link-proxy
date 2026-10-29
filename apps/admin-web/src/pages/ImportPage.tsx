import { ImportStatus } from "../features/imports/ImportStatus";
import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { imports } from "../api/imports";
import { download } from "../api/client";
import { useOperation } from "../hooks/useOperation";
import { ErrorMessage, JsonView } from "../components/common/Feedback";
export default function ImportPage() {
  const [format, F] = useState("csv"),
    [policy, P] = useState("reject"),
    [selected, S] = useState(""),
    [confirmed, C] = useState(false),
    [key, K] = useState(() => crypto.randomUUID());
  const op = useOperation();
  const jobs = useQuery({
    queryKey: ["imports"],
    queryFn: imports.list,
    refetchInterval: 2000,
  });
  const preview = useQuery({
    queryKey: ["preview", selected],
    queryFn: () => imports.preview(selected),
    enabled: !!selected,
    refetchInterval: 2000,
  });
  const errors = useQuery({
    queryKey: ["errors", selected],
    queryFn: () => imports.errors(selected),
    enabled: !!selected,
    refetchInterval: 2000,
  });
  return (
    <>
      <h1>Bulk import</h1>
      <p className="subtitle">
        Upload → validate → preview → explicit commit. Validation never
        changes active routes.
      </p>
      <ErrorMessage error={op.error || jobs.error || preview.error} />
      <form
        className="panel"
        onSubmit={(e) => {
          e.preventDefault();
          const data = new FormData(e.currentTarget);
          void op.run(async () => {
            const job = await imports.upload(data);
            S(job.id);
            C(false);
            K(crypto.randomUUID());
          }, "File staged for validation");
        }}
      >
        <div className="row">
          <label>
            Format
            <select
              name="format"
              value={format}
              onChange={(e) => F(e.target.value)}
            >
              {["csv", "xlsx", "json"].map((x) => (
                <option key={x}>{x}</option>
              ))}
            </select>
          </label>
          <label>
            Conflict policy
            <select
              name="conflict_policy"
              value={policy}
              onChange={(e) => P(e.target.value)}
            >
              {["reject", "skip", "upsert"].map((x) => (
                <option key={x}>{x}</option>
              ))}
            </select>
          </label>
          <label>
            File
            <input name="file" type="file" required accept={`.${format}`} />
          </label>
        </div>
        <button disabled={op.busy}>Upload and dry-run</button>{" "}
        <button
          type="button"
          className="quiet"
          onClick={() =>
            void op.run(
              () =>
                download(
                  `/imports/templates?format=${format}`,
                  `template.${format}`,
                ),
              "Template downloaded",
            )
          }
        >
          Download template
        </button>
      </form>
      <section className="panel">
        <h2>Import jobs</h2>
        {jobs.data?.items.map((j) => (
          <button
            className="job quiet"
            key={j.id}
            onClick={() => {
              S(j.id);
              C(false);
              K(crypto.randomUUID());
            }}
          >
            {j.file_name} · <ImportStatus value={j.status}/> · {j.total_records} records
          </button>
        ))}
      </section>
      {preview.data && (
        <section className="panel">
          <h2>Preview · {preview.data.job.status}</h2>
          <p>
            {preview.data.job.total_records} records ·{" "}
            {preview.data.job.error_count} errors · base revision{" "}
            {preview.data.job.base_revision}
          </p>
          <JsonView value={preview.data.items} />
          <h3>Validation errors</h3>
          <JsonView value={errors.data?.items || []} />
          <button
            className="quiet"
            onClick={() =>
              void op.run(
                () =>
                  download(
                    `/imports/${selected}/errors?format=csv`,
                    "import-errors.csv",
                  ),
                "Report downloaded",
              )
            }
          >
            Download error report
          </button>
          <label className="check">
            <input
              type="checkbox"
              checked={confirmed}
              onChange={(e) => C(e.target.checked)}
            />
            I reviewed the preview and authorize publishing this batch.
          </label>
          <button
            disabled={
              !confirmed || preview.data.job.status !== "ready" || op.busy
            }
            onClick={() =>
              void op.run(
                () => imports.commit(preview.data.job, key),
                "Import committed and published",
              )
            }
          >
            Commit one revision
          </button>{" "}
          <button
            disabled={
              !["pending", "validating", "ready"].includes(
                preview.data.job.status,
              ) || op.busy
            }
            className="quiet"
            onClick={() =>
              void op.run(() => imports.cancel(selected), "Import cancelled")
            }
          >
            Cancel job
          </button>
        </section>
      )}
      {op.notice && <p role="status">{op.notice}</p>}
    </>
  );
}
