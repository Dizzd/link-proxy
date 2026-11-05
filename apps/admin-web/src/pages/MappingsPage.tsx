import { useState } from "react";
import { useMappings } from "../features/mappings/useMappings";
import { Pagination } from "../components/tables/Pagination";
import { Link } from "react-router-dom";
import { mappings } from "../api/mappings";
import { filterQuery } from "../utils/defaults";
import { useOperation } from "../hooks/useOperation";
import { ErrorMessage } from "../components/common/Feedback";
import { deleteSelected } from "../features/mappings/deleteSelected";
export default function MappingsPage() {
  const [search, S] = useState(""),
    [domain, D] = useState(""),
    [action, A] = useState(""),
    [enabled, E] = useState(""),
    [offset, O] = useState(0);
  const query = filterQuery(search, domain, action, enabled, offset);
  const q = useMappings(query);
  const op = useOperation();
  const [selection, setSelection] = useState<{ query: string; ids: string[] }>({
    query: "",
    ids: [],
  });
  const [result, setResult] = useState("");
  const items = q.data?.items || [];
  const selected =
    selection.query === query
      ? selection.ids.filter((id) => items.some((item) => item.id === id))
      : [];
  const allSelected = items.length > 0 && selected.length === items.length;
  const locked = op.busy || q.isFetching;
  async function removeSelected() {
    const ids = [...selected];
    if (
      !ids.length ||
      !confirm(
        `Delete ${ids.length} selected mapping(s) from the draft? Publish is required to update live routing.`,
      )
    )
      return;
    setResult("");
    await op.run(async () => {
      const outcome = await deleteSelected(ids, mappings.remove);
      setSelection({
        query,
        ids: ids.filter((id) => !outcome.deletedIds.includes(id)),
      });
      setResult(
        `${outcome.deletedIds.length} of ${ids.length} mapping(s) deleted from the draft. Publish when ready to update live routing.`,
      );
      const refreshed = await q.refetch();
      if (refreshed.data && offset >= refreshed.data.total)
        O(Math.max(0, Math.ceil(refreshed.data.total / 25) - 1) * 25);
      if (outcome.error)
        throw new Error(
          `Deletion stopped. Review the remaining selection before retrying: ${outcome.error instanceof Error ? outcome.error.message : String(outcome.error)}`,
        );
    });
  }
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
            disabled={op.busy}
            value={search}
            onChange={(e) => {
              S(e.target.value);
              setSelection({ query, ids: [] });
              O(0);
            }}
          />
        </label>
        <label>
          Host
          <input
            disabled={op.busy}
            value={domain}
            onChange={(e) => {
              D(e.target.value);
              setSelection({ query, ids: [] });
              O(0);
            }}
          />
        </label>
        <label>
          Action
          <select
            disabled={op.busy}
            value={action}
            onChange={(e) => {
              A(e.target.value);
              setSelection({ query, ids: [] });
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
            disabled={op.busy}
            value={enabled}
            onChange={(e) => {
              E(e.target.value);
              setSelection({ query, ids: [] });
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
      <div className="bulk-actions" aria-label="Mapping selection actions">
        <span role="status">
          {selected.length} selected on this page (up to 25)
        </span>
        <button
          disabled={locked || !selected.length}
          onClick={() => void removeSelected()}
        >
          {op.busy ? "Deleting…" : `Delete selected (${selected.length})`}
        </button>
        <button
          className="quiet"
          disabled={op.busy || !selected.length}
          onClick={() => setSelection({ query, ids: [] })}
        >
          Clear selection
        </button>
      </div>
      {result && <p role="status">{result}</p>}
      <div className="table-scroll">
        <table>
          <thead>
            <tr>
              <th scope="col">
                <label className="mapping-select">
                  <input
                    type="checkbox"
                    aria-label="Select all mappings on this page"
                    checked={allSelected}
                    disabled={locked || !items.length}
                    ref={(input) => {
                      if (input)
                        input.indeterminate =
                          selected.length > 0 && !allSelected;
                    }}
                    onChange={() =>
                      setSelection({
                        query,
                        ids: allSelected ? [] : items.map((item) => item.id),
                      })
                    }
                  />
                </label>
              </th>
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
                  <label className="mapping-select">
                    <input
                      type="checkbox"
                      aria-label={`Select ${m.mapping_key}`}
                      checked={selected.includes(m.id)}
                      disabled={locked}
                      onChange={(event) =>
                        setSelection({
                          query,
                          ids: event.target.checked
                            ? [...selected, m.id]
                            : selected.filter((id) => id !== m.id),
                        })
                      }
                    />
                  </label>
                </td>
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
                    disabled={locked}
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
        disabled={op.busy}
        offset={offset}
        total={q.data?.total || 0}
        pageSize={25}
        onChange={(value) => {
          if (!op.busy) {
            setSelection({ query, ids: [] });
            O(value);
          }
        }}
      />
    </>
  );
}
