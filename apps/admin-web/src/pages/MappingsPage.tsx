import { useState } from "react";
import { useMappings } from "../features/mappings/useMappings";
import { Pagination } from "../components/tables/Pagination";
import { Link } from "react-router-dom";
import { mappings } from "../api/mappings";
import { filterQuery } from "../utils/defaults";
import { useOperation } from "../hooks/useOperation";
import { ErrorMessage } from "../components/common/Feedback";
export default function MappingsPage() {
  const [search, S] = useState(""),
    [domain, D] = useState(""),
    [action, A] = useState(""),
    [enabled, E] = useState(""),
    [offset, O] = useState(0);
  const query = filterQuery(search, domain, action, enabled, offset);
  const q = useMappings(query);
  const op = useOperation();
  return (
    <>
      <div className="titlebar">
        <div>
          <h1>Mappings</h1>
          <p className="subtitle">Edit the draft. Publish when it is ready.</p>
        </div>
        <Link className="button" to="/mappings/new">
          Create mapping
        </Link>
      </div>
      <div className="filters">
        <label>
          Search
          <input
            value={search}
            onChange={(e) => {
              S(e.target.value);
              O(0);
            }}
          />
        </label>
        <label>
          Host
          <input
            value={domain}
            onChange={(e) => {
              D(e.target.value);
              O(0);
            }}
          />
        </label>
        <label>
          Action
          <select
            value={action}
            onChange={(e) => {
              A(e.target.value);
              O(0);
            }}
          >
            <option value="">All actions</option>
            <option value="redirect">Redirect</option>
            <option value="reverse_proxy">Reverse proxy</option>
          </select>
        </label>
        <label>
          Status
          <select
            value={enabled}
            onChange={(e) => {
              E(e.target.value);
              O(0);
            }}
          >
            <option value="">All statuses</option>
            <option value="true">Enabled</option>
            <option value="false">Disabled</option>
          </select>
        </label>
      </div>
      <ErrorMessage error={q.error || op.error} />
      <div className="table-scroll">
        <table>
          <thead>
            <tr>
              <th>Mapping</th>
              <th>Source</th>
              <th>Action</th>
              <th>Rules</th>
              <th>Status</th>
              <th>Manage</th>
            </tr>
          </thead>
          <tbody>
            {q.data?.items.map((m) => (
              <tr key={m.id}>
                <td>
                  <Link to={`/mappings/${m.id}`}>{m.mapping_key}</Link>
                  <small>{m.name}</small>
                </td>
                <td>
                  <code>
                    {m.source_matcher.hosts.join(", ")}
                    {m.source_matcher.path_value}
                  </code>
                  <small>{m.source_matcher.path_type}</small>
                </td>
                <td>{m.default_action.type.replaceAll("_", " ")}</td>
                <td>{m.rules.length}</td>
                <td>
                  <span className="badge">
                    {m.enabled ? "Enabled" : "Disabled"}
                  </span>
                </td>
                <td>
                  <button
                    disabled={op.busy}
                    className="quiet"
                    onClick={() => {
                      if (confirm(`Delete ${m.mapping_key} from the draft?`))
                        void op.run(() => mappings.remove(m.id));
                    }}
                  >
                    Delete
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      {q.isPending && <p role="status">Loading mappings…</p>}
      {q.data?.total === 0 && (
        <p>No matching mappings. Create one or import a file.</p>
      )}
      <Pagination
        offset={offset}
        total={q.data?.total || 0}
        pageSize={25}
        onChange={O}
      />
    </>
  );
}
