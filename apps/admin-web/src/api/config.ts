import { request, send } from "./client";
import type { Active, CacheStatus, Entity } from "../types";
export interface RevisionPrunePreview {
  keep_latest: number;
  database_revision: number;
  active_revision: number;
  total_revisions: number;
  eligible_count: number;
  versions: number[];
}
export const config = {
  active: () => request<Active>("/config/active"),
  cacheStatus: () => request<CacheStatus>("/system/cache/status"),
  revisions: () => request<{ items: Entity[] }>("/config/revisions"),
  previewRevisionPrune: (keepLatest: number) =>
    send<RevisionPrunePreview>("/config/revisions/prune/preview", "POST", {
      keep_latest: keepLatest,
    }),
  pruneRevisions: (preview: RevisionPrunePreview) =>
    send<{ deleted_count: number }>("/config/revisions/prune", "POST", preview),
  publish: (revision: number) =>
    send("/config/publish", "POST", { expected_revision: revision }),
  restore: (version: number, revision: number) =>
    send(`/config/revisions/${version}/restore`, "POST", {
      expected_revision: revision,
    }),
};
