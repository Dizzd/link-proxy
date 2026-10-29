import type { Action, Entity } from "../../types";
import { defaultAction } from "../../utils/defaults";
export function ActionEditor({
  value,
  onChange,
  upstreams = [],
}: {
  value: Action;
  onChange: (a: Action) => void;
  upstreams?: Entity[];
}) {
  return (
    <fieldset>
      <legend>Destination action</legend>
      <label>
        Action
        <select
          value={value.type}
          onChange={(e) =>
            onChange(
              e.target.value === "redirect"
                ? defaultAction()
                : {
                    type: "reverse_proxy",
                    upstream_id: upstreams[0]?.id || "",
                    rewrite: null,
                    preserve_host: false,
                  },
            )
          }
        >
          <option value="redirect">HTTP redirect</option>
          <option value="reverse_proxy">Reverse proxy</option>
        </select>
      </label>
      {value.type === "redirect" ? (
        <>
          <label>
            Target URL
            <input
              type="url"
              required
              value={value.target_url}
              onChange={(e) =>
                onChange({ ...value, target_url: e.target.value })
              }
            />
          </label>
          <div className="row">
            <label>
              Status
              <select
                value={value.status}
                onChange={(e) =>
                  onChange({ ...value, status: +e.target.value })
                }
              >
                {[301, 302, 303, 307, 308].map((x) => (
                  <option key={x}>{x}</option>
                ))}
              </select>
            </label>
            <label>
              Path handling
              <select
                value={value.path_handling}
                onChange={(e) =>
                  onChange({
                    ...value,
                    path_handling: e.target.value as "replace" | "append",
                  })
                }
              >
                <option>replace</option>
                <option>append</option>
              </select>
            </label>
            <label>
              Query policy
              <select
                value={value.query_policy}
                onChange={(e) =>
                  onChange({
                    ...value,
                    query_policy: e.target.value as
                      "drop" | "preserve" | "merge",
                  })
                }
              >
                {["drop", "preserve", "merge"].map((x) => (
                  <option key={x}>{x}</option>
                ))}
              </select>
            </label>
          </div>
        </>
      ) : (
        <>
          <label>
            Upstream
            <select
              required
              value={value.upstream_id}
              onChange={(e) =>
                onChange({ ...value, upstream_id: e.target.value })
              }
            >
              <option value="">Select an upstream</option>
              {upstreams.map((u) => (
                <option key={u.id} value={u.id}>
                  {String(u.name)}
                </option>
              ))}
            </select>
          </label>
          <label>
            Path rewrite
            <input
              value={value.rewrite || ""}
              placeholder="/api/{id}"
              onChange={(e) =>
                onChange({ ...value, rewrite: e.target.value || null })
              }
            />
          </label>
          <label className="check">
            <input
              type="checkbox"
              checked={value.preserve_host}
              onChange={(e) =>
                onChange({ ...value, preserve_host: e.target.checked })
              }
            />
            Preserve incoming Host
          </label>
        </>
      )}
    </fieldset>
  );
}
