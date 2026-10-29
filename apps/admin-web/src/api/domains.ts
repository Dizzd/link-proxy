import { request, send } from "./client";
import type { Entity } from "../types";
export const domains = {
  list: () => request<{ items: Entity[] }>("/domains"),
  save: (data: Entity, exists: boolean) =>
    send(
      exists ? `/domains/${data.id}` : "/domains",
      exists ? "PATCH" : "POST",
      exists
        ? Object.fromEntries(
            Object.entries(data).filter(
              ([k]) => !["id", "created_at", "updated_at"].includes(k),
            ),
          )
        : data,
    ),
  remove: (id: string) => send("/domains/" + id, "DELETE"),
};
