import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { useOperation } from "../../hooks/useOperation";
import { ErrorMessage, JsonView } from "../common/Feedback";
import type { Entity } from "../../types";
export function EntityManager({
  name,
  api,
  defaults,
}: {
  name: string;
  api: {
    list: () => Promise<{ items: Entity[] }>;
    save: (e: Entity, exists: boolean) => Promise<unknown>;
    remove: (id: string) => Promise<unknown>;
  };
  defaults: () => Entity;
}) {
  const [editing, S] = useState<Entity | null>(null),
    [exists, E] = useState(false);
  const op = useOperation();
  const q = useQuery({ queryKey: [name.toLowerCase()], queryFn: api.list });
  return (
    <>
      <div className="titlebar">
        <h1>{name}</h1>
        <button
          onClick={() => {
            S(defaults());
            E(false);
          }}
        >
          Create {name.toLowerCase().slice(0, -1)}
        </button>
      </div>
      <ErrorMessage error={op.error || q.error} />
      {q.data?.items.length === 0 && <p>No {name.toLowerCase()} configured.</p>}
      {q.data?.items.map((e) => (
        <article className="panel" key={e.id}>
          <h2>{String(e.name || e.hostname)}</h2>
          <JsonView value={e} />
          <button
            onClick={() => {
              S(e);
              E(true);
            }}
          >
            Edit
          </button>{" "}
          <button
            className="quiet"
            disabled={op.busy}
            onClick={() => {
              if (confirm("Delete this draft resource?"))
                void op.run(() => api.remove(e.id));
            }}
          >
            Delete
          </button>
        </article>
      ))}
      {editing && (
        <form
          className="panel"
          onSubmit={(e) => {
            e.preventDefault();
            void op.run(async () => {
              await api.save(editing, exists);
              S(null);
            });
          }}
        >
          <h2>{exists ? "Edit" : "Create"}</h2>
          {Object.entries(editing)
            .filter(
              ([key]) => !["id", "created_at", "updated_at"].includes(key),
            )
            .map(([key, value]) => (
              <label key={key}>
                {key.replaceAll("_", " ")}
                {typeof value === "boolean" ? (
                  <input
                    type="checkbox"
                    checked={value}
                    onChange={(e) => S({ ...editing, [key]: e.target.checked })}
                  />
                ) : (
                  <input
                    required
                    type={typeof value === "number" ? "number" : "text"}
                    value={String(value)}
                    onChange={(e) =>
                      S({
                        ...editing,
                        [key]:
                          typeof value === "number"
                            ? +e.target.value
                            : e.target.value,
                      })
                    }
                  />
                )}
              </label>
            ))}
          <button disabled={op.busy}>Save draft</button>{" "}
          <button type="button" className="quiet" onClick={() => S(null)}>
            Cancel
          </button>
        </form>
      )}
    </>
  );
}
