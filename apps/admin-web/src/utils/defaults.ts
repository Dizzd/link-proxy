import type { Action, Condition, Mapping } from "../types";
export const defaultAction = (): Action => ({
  type: "redirect",
  status: 302,
  target_url: "https://example.org/",
  path_handling: "replace",
  query_policy: "drop",
});
export function defaultCondition(type = "device_type"): Condition {
  switch (type) {
    case "all":
    case "any":
      return { type, conditions: [defaultCondition()] };
    case "not":
      return { type, condition: defaultCondition() };
    case "time_window":
      return {
        type,
        timezone: "Asia/Ho_Chi_Minh",
        days_of_week: ["mon", "tue", "wed", "thu", "fri"],
        start: "08:00",
        end: "18:00",
      };
    case "date_range":
      return { type, timezone: "UTC", start: "2026-01-01", end: "2026-12-31" };
    case "query_parameter_equals":
      return { type, name: "campaign", value: "" };
    default:
      return { type: "device_type", in: ["mobile"] };
  }
}
export const newMapping = (): Mapping => ({
  id: crypto.randomUUID(),
  mapping_key: "",
  name: "",
  description: "",
  enabled: true,
  priority: 0,
  source_matcher: {
    hosts: [""],
    path_type: "exact",
    path_value: "/",
    methods: [],
  },
  default_action: defaultAction(),
  rules: [],
});
export function filterQuery(
  search: string,
  domain: string,
  action: string,
  enabled: string,
  offset: number,
) {
  const p = new URLSearchParams({
    search,
    offset: String(offset),
    limit: "25",
  });
  if (domain) p.set("domain", domain);
  if (action) p.set("action", action);
  if (enabled) p.set("enabled", enabled);
  return p.toString();
}
