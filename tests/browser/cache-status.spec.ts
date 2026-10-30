import { test, expect } from "@playwright/test";

test("cache badge follows API state and recovers without stale connectivity", async ({
  page,
}) => {
  // Only this test's browser context receives fixtures; no server configuration is changed.
  await page.route("**/api/v1/auth/session", (route) =>
    route.fulfill({
      json: {
        user: { id: "badge-test", username: "badge-test" },
        csrf_token: "badge-test",
        expires_at: 4102444800,
      },
    }),
  );
  await page.route("**/api/v1/config/active", (route) =>
    route.fulfill({
      json: {
        revision: 0,
        draft_generation: 0,
        configuration: { mappings: [], domains: [], upstreams: [] },
      },
    }),
  );
  let releaseInitial: () => void = () => {};
  const initial = new Promise<void>((resolve) => {
    releaseInitial = resolve;
  });
  let responseError = false;
  let mode = "memory_redis";
  let connected = true;
  await page.route("**/api/v1/system/cache/status", async (route) => {
    await initial;
    await route.fulfill(
      responseError
        ? {
            status: 503,
            json: { error: { message: "Test status unavailable" } },
          }
        : {
            json: {
              mode,
              connected,
              active_revision: 0,
              database_revision: 0,
              last_sync: "2026-10-30T00:00:00Z",
              error: connected ? null : "Cache unavailable",
            },
          },
    );
  });
  await page.goto("/");
  const badge = page.getByRole("status", { name: "Configuration cache" });
  await expect(badge).toContainText("Checking cache...");
  await expect(badge).not.toContainText("Memory only");
  releaseInitial();
  await expect(badge).toContainText("Memory + Redis");
  await expect(badge).toContainText("Redis connected");

  responseError = true;
  await expect(badge).toContainText("Status unavailable", { timeout: 15000 });
  await expect(badge).not.toContainText("Redis connected");

  connected = false;
  responseError = false;
  await expect(badge).toContainText("Memory + Redis", { timeout: 10000 });
  await expect(badge).toContainText(
    "Redis unavailable; local routing continues.",
  );
  connected = true;
  await expect(badge).toContainText("Redis connected", { timeout: 10000 });

  mode = "memory";
  await expect(badge).toContainText("Memory only", { timeout: 10000 });
  await expect(badge).not.toContainText("Redis connected");
  await page.setViewportSize({ width: 850, height: 900 });
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= window.innerWidth,
    ),
  ).toBe(true);
});
