import { request, send } from "./client";
import type { Entity } from "../types";
export const upstreams = {
  list: () => request<{ items: Entity[] }>("/upstreams"),
  save: (data: Entity, exists: boolean) =>
    send(
      exists ? `/upstreams/${data.id}` : "/upstreams",
      exists ? "PATCH" : "POST",
      exists
        ? Object.fromEntries(
            Object.entries(data).filter(
              ([k]) => !["id", "created_at", "updated_at"].includes(k),
            ),
          )
        : data,
    ),
  remove: (id: string) => send("/upstreams/" + id, "DELETE"),
};
