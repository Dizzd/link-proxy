import { useEffect, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { mappings } from "../api/mappings";
import { upstreams } from "../api/upstreams";
import { newMapping, defaultAction, defaultCondition } from "../utils/defaults";
import { ActionEditor } from "../components/forms/ActionEditor";
import { ConditionBuilder } from "../features/conditions/ConditionBuilder";
import { Simulator } from "../features/simulation/Simulator";
import { useOperation } from "../hooks/useOperation";
import { ErrorMessage } from "../components/common/Feedback";
import type { Mapping } from "../types";
export default function MappingEditorPage() {
  const { id } = useParams();
  const existing = !!id && id !== "new";
  const navigate = useNavigate();
  const [m, M] = useState<Mapping>(newMapping);
  const op = useOperation();
  const q = useQuery({
    queryKey: ["mapping", id],
    queryFn: () => mappings.get(id!),
    enabled: existing,
  });
  const u = useQuery({ queryKey: ["upstreams"], queryFn: upstreams.list });
  useEffect(() => {
    if (q.data) M(q.data);
  }, [q.data]);
  function rule(i: number, change: Partial<Mapping["rules"][number]>) {
    M({
      ...m,
      rules: m.rules.map((r, j) => (i === j ? { ...r, ...change } : r)),
    });
  }
  return (
    <>
      <h1>{existing ? "Edit mapping" : "Create mapping"}</h1>
      <ErrorMessage error={op.error || q.error} />
      <form
        onSubmit={(e) => {
          e.preventDefault();
          void op.run(async () => {
            await mappings.save(m, existing);
            navigate("/mappings");
          });
        }}
      >
        <section className="panel">
          <h2>Source matcher</h2>
          <div className="row">
            <label>
              Mapping key
              <input
                required
                pattern="[A-Za-z0-9_-]+"
                value={m.mapping_key}
                onChange={(e) => M({ ...m, mapping_key: e.target.value })}
              />
            </label>
            <label>
              Name
              <input
                value={m.name}
                onChange={(e) => M({ ...m, name: e.target.value })}
              />
            </label>
            <label>
              Priority
              <input
                type="number"
                value={m.priority}
                onChange={(e) => M({ ...m, priority: +e.target.value })}
              />
            </label>
          </div>
          <label>
            Description
            <input
              value={m.description}
              onChange={(e) => M({ ...m, description: e.target.value })}
            />
          </label>
          <label>
            Hosts (comma separated)
            <input
              required
              value={m.source_matcher.hosts.join(",")}
              onChange={(e) =>
                M({
                  ...m,
                  source_matcher: {
                    ...m.source_matcher,
                    hosts: e.target.value.split(",").map((x) => x.trim()),
                  },
                })
              }
            />
          </label>
          <div className="row">
            <label>
              Path type
              <select
                value={m.source_matcher.path_type}
                onChange={(e) =>
                  M({
                    ...m,
                    source_matcher: {
                      ...m.source_matcher,
                      path_type: e.target.value as
                        "exact" | "prefix" | "template",
                    },
                  })
                }
              >
                {["exact", "prefix", "template"].map((x) => (
                  <option key={x}>{x}</option>
                ))}
              </select>
            </label>
            <label>
              Path
              <input
                required
                value={m.source_matcher.path_value}
                onChange={(e) =>
                  M({
                    ...m,
                    source_matcher: {
                      ...m.source_matcher,
                      path_value: e.target.value,
                    },
                  })
                }
              />
            </label>
            <label>
              Methods (empty = any)
              <input
                value={m.source_matcher.methods.join(",")}
                onChange={(e) =>
                  M({
                    ...m,
                    source_matcher: {
                      ...m.source_matcher,
                      methods: e.target.value
                        ? e.target.value
                            .split(",")
                            .map((x) => x.trim().toUpperCase())
                        : [],
                    },
                  })
                }
              />
            </label>
          </div>
          <label className="check">
            <input
              type="checkbox"
              checked={m.enabled}
              onChange={(e) => M({ ...m, enabled: e.target.checked })}
            />
            Enabled
          </label>
        </section>
        <section className="panel">
          <h2>Fallback</h2>
          <ActionEditor
            value={m.default_action}
            onChange={(a) => M({ ...m, default_action: a })}
            upstreams={u.data?.items}
          />
        </section>
        <section className="panel">
          <h2>Conditional rules</h2>
          <p>First matching enabled rule wins, in descending priority.</p>
          {m.rules.map((r, i) => (
            <article className="rule" key={r.id}>
              <div className="row">
                <label>
                  Rule name
                  <input
                    value={r.name}
                    onChange={(e) => rule(i, { name: e.target.value })}
                  />
                </label>
                <label>
                  Priority
                  <input
                    type="number"
                    value={r.priority}
                    onChange={(e) => rule(i, { priority: +e.target.value })}
                  />
                </label>
                <label className="check">
                  <input
                    type="checkbox"
                    checked={r.enabled}
                    onChange={(e) => rule(i, { enabled: e.target.checked })}
                  />
                  Enabled
                </label>
              </div>
              <ConditionBuilder
                value={r.condition}
                onChange={(c) => rule(i, { condition: c })}
              />
              <ActionEditor
                value={r.action}
                onChange={(a) => rule(i, { action: a })}
                upstreams={u.data?.items}
              />
              <button
                type="button"
                className="quiet"
                onClick={() =>
                  M({ ...m, rules: m.rules.filter((_, j) => j !== i) })
                }
              >
                Remove rule
              </button>
            </article>
          ))}
          <button
            type="button"
            onClick={() =>
              M({
                ...m,
                rules: [
                  ...m.rules,
                  {
                    id: crypto.randomUUID(),
                    name: "New rule",
                    enabled: true,
                    priority: (m.rules.length + 1) * 10,
                    condition: defaultCondition(),
                    action: defaultAction(),
                    condition_schema_version: 1,
                  },
                ],
              })
            }
          >
            Add rule
          </button>
        </section>
        <button disabled={op.busy || q.isFetching}>Save draft mapping</button>
      </form>
      <Simulator />
    </>
  );
}
