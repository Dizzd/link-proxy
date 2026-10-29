import type { Condition } from "../../types";
import { defaultCondition } from "../../utils/defaults";
export function ConditionBuilder({
  value,
  onChange,
  depth = 0,
}: {
  value: Condition;
  onChange: (c: Condition) => void;
  depth?: number;
}) {
  const field = (key: string, label: string, type = "text") => (
    <label key={key}>
      {label}
      <input
        type={type}
        value={String(value[key] || "")}
        onChange={(e) => onChange({ ...value, [key]: e.target.value })}
      />
    </label>
  );
  let content;
  if (value.type === "all" || value.type === "any") {
    content = (
      <>
        {value.conditions?.map((c, i) => (
          <div key={i}>
            <ConditionBuilder
              depth={depth + 1}
              value={c}
              onChange={(next) =>
                onChange({
                  ...value,
                  conditions: value.conditions?.map((old, j) =>
                    i === j ? next : old,
                  ),
                })
              }
            />
            <button
              type="button"
              className="quiet"
              onClick={() =>
                onChange({
                  ...value,
                  conditions: value.conditions?.filter((_, j) => i !== j),
                })
              }
            >
              Remove condition
            </button>
          </div>
        ))}
        <button
          type="button"
          disabled={depth >= 8}
          onClick={() =>
            onChange({
              ...value,
              conditions: [...(value.conditions || []), defaultCondition()],
            })
          }
        >
          Add condition
        </button>
      </>
    );
  } else if (value.type === "not") {
    content =
      depth >= 8 ? (
        <p>Maximum editor depth reached.</p>
      ) : (
        <ConditionBuilder
          depth={depth + 1}
          value={value.condition || defaultCondition()}
          onChange={(condition) => onChange({ ...value, condition })}
        />
      );
  } else if (value.type === "device_type") {
    content = (
      <label>
        Devices (comma separated)
        <input
          value={(value.in as string[]).join(",")}
          onChange={(e) =>
            onChange({
              ...value,
              in: e.target.value.split(",").map((x) => x.trim()),
            })
          }
        />
      </label>
    );
  } else if (value.type === "query_parameter_equals") {
    content = (
      <>
        {field("name", "Parameter")}
        {field("value", "Exact value")}
      </>
    );
  } else {
    content = (
      <>
        {field("timezone", "IANA timezone")}
        {field(
          "start",
          "Start",
          value.type === "time_window" ? "time" : "date",
        )}
        {field("end", "End", value.type === "time_window" ? "time" : "date")}
        {value.type === "time_window" && (
          <label>
            Days (mon,tue,wed,thu,fri,sat,sun)
            <input
              value={(value.days_of_week as string[]).join(",")}
              onChange={(e) =>
                onChange({
                  ...value,
                  days_of_week: e.target.value.split(",").map((x) => x.trim()),
                })
              }
            />
          </label>
        )}
      </>
    );
  }
  return (
    <fieldset>
      <legend>Condition</legend>
      <label>
        Operator
        <select
          value={value.type}
          onChange={(e) => onChange(defaultCondition(e.target.value))}
        >
          {[
            "all",
            "any",
            "not",
            "time_window",
            "date_range",
            "device_type",
            "query_parameter_equals",
          ].map((x) => (
            <option key={x} value={x}>
              {x.replaceAll("_", " ")}
            </option>
          ))}
        </select>
      </label>
      {content}
    </fieldset>
  );
}
