import { request, send } from "./client";
import type { Active, Entity } from "../types";
export const config = {
  active: () => request<Active>("/config/active"),
  revisions: () => request<{ items: Entity[] }>("/config/revisions"),
  publish: (revision: number) =>
    send("/config/publish", "POST", { expected_revision: revision }),
  restore: (version: number, revision: number) =>
    send(`/config/revisions/${version}/restore`, "POST", {
      expected_revision: revision,
    }),
};
