import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { CacheStatusView } from "./CacheStatusBadge";
import type { CacheStatus } from "../../types";

const redis: CacheStatus = {
  mode: "memory_redis",
  connected: true,
  active_revision: 1,
  database_revision: 1,
  last_sync: null,
  error: null,
};
function render(data?: CacheStatus, pending = false, error = false) {
  return renderToStaticMarkup(
    <CacheStatusView data={data} pending={pending} error={error} />,
  );
}
describe("live configuration cache badge", () => {
  it("does not assume a memory mode before the API responds", () => {
    const html = render(undefined, true);
    expect(html).toContain("Checking cache...");
    expect(html).not.toContain("Memory only");
    expect(html).toContain('role="status"');
    expect(html).toContain('aria-atomic="true"');
  });
  it("identifies the actual memory-only configuration", () => {
    expect(render({ ...redis, mode: "memory" })).toContain("Memory only");
    expect(render({ ...redis, mode: "memory" })).not.toContain(
      "Redis connected",
    );
  });
  it("shows Redis mode and a successful connection separately", () => {
    const html = render(redis);
    expect(html).toContain("Memory + Redis");
    expect(html).toContain("Redis connected");
  });
  it("keeps Redis mode visible during an actual Redis outage", () => {
    const html = render({
      ...redis,
      connected: false,
      error: "Cache unavailable",
    });
    expect(html).toContain("Memory + Redis");
    expect(html).toContain("Redis unavailable; local routing continues.");
    expect(html).not.toContain("Redis connected");
  });
  it("distinguishes startup synchronization from a reported Redis failure", () => {
    const html = render({ ...redis, connected: false });
    expect(html).toContain("Memory + Redis");
    expect(html).toContain("Awaiting Redis synchronization");
    expect(html).not.toContain("Redis unavailable");
  });
  it("does not claim stale Redis connectivity after a failed status refresh", () => {
    const html = render(redis, false, true);
    expect(html).toContain("Cache unknown");
    expect(html).toContain("Status unavailable");
    expect(html).not.toContain("Redis connected");
  });
  it("shows unavailable when the initial request fails", () => {
    expect(render(undefined, false, true)).toContain("Cache unknown");
  });
  it("does not invent a mode when no status is available", () => {
    expect(render()).toContain("Cache unknown");
    expect(render()).not.toContain("Memory only");
  });
});
