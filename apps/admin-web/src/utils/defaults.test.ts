import { describe, it, expect } from "vitest";
import { defaultCondition, filterQuery, defaultAction } from "./defaults";
describe("control plane inputs", () => {
  it("encodes filters without query injection", () => {
    const p = new URLSearchParams(
      filterQuery("a&enabled=false", "example.com", "redirect", "true", 25),
    );
    expect(p.get("search")).toBe("a&enabled=false");
    expect(p.get("enabled")).toBe("true");
    expect(p.get("offset")).toBe("25");
  });
  it("creates finite nested conditions with valid defaults", () => {
    for (const type of [
      "all",
      "any",
      "not",
      "device_type",
      "time_window",
      "date_range",
      "query_parameter_equals",
    ])
      expect(defaultCondition(type).type).toBe(type);
  });
  it("uses explicit redirect policies", () =>
    expect(defaultAction()).toMatchObject({
      status: 302,
      query_policy: "drop",
      path_handling: "replace",
    }));
});
