import { useEffect, useState } from "react";
import { config, type RevisionPrunePreview } from "../../api/config";
import type { useOperation } from "../../hooks/useOperation";

export function RevisionCleanup({
  operation: op,
  activeRevision,
}: {
  operation: ReturnType<typeof useOperation>;
  activeRevision: number | undefined;
}) {
  const [keepLatest, setKeepLatest] = useState("100");
  const [preview, setPreview] = useState<RevisionPrunePreview | null>(null);
  const [result, setResult] = useState("");
  useEffect(() => {
    setPreview((current) =>
      current && current.active_revision !== activeRevision ? null : current,
    );
  }, [activeRevision]);
  const keep = Number(keepLatest);
  const valid = Number.isInteger(keep) && keep >= 1 && keep <= 10000;
  return (
    <section className="panel" aria-labelledby="revision-cleanup-title">
      <h2 id="revision-cleanup-title">Clean up old revisions</h2>
      <p>
        Keep recent history and remove older superseded revisions. Running,
        committed and recovery revisions are protected. Deleted revisions cannot
        be restored.
      </p>
      <label>
        Latest revisions to keep
        <input
          type="number"
          min="1"
          max="10000"
          step="1"
          value={keepLatest}
          disabled={op.busy}
          aria-invalid={!valid}
          aria-describedby="revision-retention-help"
          onChange={(event) => {
            setKeepLatest(event.target.value);
            setPreview(null);
            setResult("");
          }}
        />
      </label>
      <p id="revision-retention-help">
        Choose 1–10,000. Default: 100. Cleanup runs only after preview and
        confirmation, up to 500 revisions per batch.
      </p>
      <button
        className="quiet"
        disabled={op.busy || !valid || activeRevision === undefined}
        onClick={() => {
          setPreview(null);
          setResult("");
          void op.run(async () => {
            setPreview(await config.previewRevisionPrune(keep));
          }, "Cleanup preview ready");
        }}
      >
        Preview cleanup
      </button>
      {preview && (
        <div>
          <p role="status">
            {preview.total_revisions} revisions stored. {preview.eligible_count}{" "}
            eligible for cleanup. This batch will delete{" "}
            {preview.versions.length} revision(s).
          </p>
          {preview.versions.length > 0 ? (
            <>
              <p>
                Batch: revision {preview.versions[0]} through{" "}
                {preview.versions.at(-1)} (eligible revisions only). Keep the
                latest {preview.keep_latest}, plus protected revisions.
              </p>
              <button
                disabled={op.busy}
                onClick={() => {
                  if (
                    !confirm(
                      `Permanently delete ${preview.versions.length} old revision(s)? Keep the latest ${preview.keep_latest} and all protected revisions. Deleted revisions cannot be restored.`,
                    )
                  )
                    return;
                  const confirmed = preview;
                  setPreview(null);
                  setResult("");
                  void op.run(async () => {
                    const outcome = await config.pruneRevisions(confirmed);
                    setResult(
                      `${outcome.deleted_count} revision(s) deleted. Preview again to check for more eligible revisions.`,
                    );
                  }, "Revision cleanup completed");
                }}
              >
                Delete {preview.versions.length} old revision(s)
              </button>
            </>
          ) : (
            <p>No old revisions are eligible for deletion.</p>
          )}
        </div>
      )}
      {result && <p role="status">{result}</p>}
    </section>
  );
}
