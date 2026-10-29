import { request, send } from "./client";
import type { ImportJob, Entity } from "../types";
export const imports = {
  list: () => request<{ items: ImportJob[] }>("/imports"),
  preview: (id: string) =>
    request<{ job: ImportJob; items: Entity[] }>(`/imports/${id}/preview`),
  errors: (id: string) => request<{ items: Entity[] }>(`/imports/${id}/errors`),
  upload: (body: FormData) =>
    request<{ id: string }>("/imports", { method: "POST", body }),
  cancel: (id: string) => send(`/imports/${id}/cancel`, "POST"),
  commit: (job: ImportJob, key: string) =>
    send(
      `/imports/${job.id}/commit`,
      "POST",
      {
        import_job_id: job.id,
        expected_revision: job.base_revision,
        conflict_policy: job.conflict_policy,
        confirmation: true,
      },
      { "Idempotency-Key": key },
    ),
};
