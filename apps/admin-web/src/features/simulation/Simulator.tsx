import { useState } from "react";
import { useMutation } from "@tanstack/react-query";
import { send } from "../../api/client";
import { ErrorMessage, JsonView } from "../../components/common/Feedback";
export function Simulator() {
  const [input, S] = useState({
    host: "abc.example.com",
    path: "/QR001",
    method: "GET",
    user_agent: "",
    query: "",
    now: "",
    draft: false,
  });
  const run = useMutation({
    mutationFn: () =>
      send("/mappings/simulate", "POST", {
        ...input,
        now: input.now || undefined,
      }),
  });
  return (
    <section className="panel">
      <h2>Route simulator</h2>
      <p>
        Uses the production evaluator without contacting upstreams or recording
        traffic.
      </p>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          run.mutate();
        }}
      >
        <div className="row">
          {(
            ["host", "path", "method", "user_agent", "query", "now"] as const
          ).map((k) => (
            <label key={k}>
              {k.replaceAll("_", " ")}
              <input
                value={input[k]}
                onChange={(e) => S({ ...input, [k]: e.target.value })}
              />
            </label>
          ))}
        </div>
        <label className="check">
          <input
            type="checkbox"
            checked={input.draft}
            onChange={(e) => S({ ...input, draft: e.target.checked })}
          />
          Evaluate draft
        </label>
        <button disabled={run.isPending}>Simulate</button>
      </form>
      <ErrorMessage error={run.error} />
      {run.isSuccess && <JsonView value={run.data} />}
    </section>
  );
}
