import { request, send } from "./client";
import type { Mapping, Page } from "../types";
export const mappings = {
  list: (query = "") => request<Page<Mapping>>(`/mappings?${query}`),
  get: (id: string) => request<Mapping>(`/mappings/${encodeURIComponent(id)}`),
  save: (mapping: Mapping, existing: boolean) =>
    send<Mapping>(
      existing ? `/mappings/${mapping.id}` : "/mappings",
      existing ? "PATCH" : "POST",
      existing
        ? Object.fromEntries(
            Object.entries(mapping).filter(
              ([k]) => !["id", "created_at", "updated_at"].includes(k),
            ),
          )
        : mapping,
    ),
  remove: (id: string) => send<void>(`/mappings/${id}`, "DELETE"),
};
