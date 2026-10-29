import { useState } from "react";
import { download } from "../api/client";
import { useOperation } from "../hooks/useOperation";
import { ErrorMessage } from "../components/common/Feedback";
export default function ExportPage() {
  const [format, F] = useState("json");
  const op = useOperation();
  return (
    <>
      <h1>Bulk export</h1>
      <section className="panel">
        <h2>Export draft configuration</h2>
        <p>
          Export nested mappings and conditional rules in JSON, CSV or XLSX.
        </p>
        <label>
          Format
          <select value={format} onChange={(e) => F(e.target.value)}>
            {["json", "csv", "xlsx"].map((x) => (
              <option key={x}>{x}</option>
            ))}
          </select>
        </label>
        <button
          disabled={op.busy}
          onClick={() =>
            void op.run(
              () =>
                download(
                  `/exports/mappings?format=${format}`,
                  `mappings.${format}`,
                ),
              "Export downloaded",
            )
          }
        >
          Download export
        </button>
        <ErrorMessage error={op.error} />
        <p role="status">{op.notice}</p>
      </section>
    </>
  );
}
