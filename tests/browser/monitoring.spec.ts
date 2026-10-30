import { test, expect } from "@playwright/test";

test("grouped navigation and live monitoring remain readable on desktop and mobile", async ({
  page,
}) => {
  await page.route("**/api/v1/auth/session", (route) =>
    route.fulfill({
      json: {
        user: { id: "monitor-test", username: "monitor-test" },
        csrf_token: "fixture",
        expires_at: 4102444800,
      },
    }),
  );
  await page.route("**/api/v1/system/cache/status", (route) =>
    route.fulfill({
      json: {
        mode: "memory",
        connected: false,
        active_revision: 1,
        database_revision: 1,
        last_sync: null,
        error: null,
      },
    }),
  );
  let count = 0;
  let unavailable = false;
  await page.route("**/metrics", (route) => {
    if (unavailable)
      return route.fulfill({
        status: 503,
        body: "Fixture metrics unavailable",
      });
    count += 10;
    return route.fulfill({
      contentType: "text/plain",
      body: `# HELP proxy_requests_total Total requests
proxy_requests_total ${count}
proxy_request_duration_seconds_count ${count}
proxy_request_duration_seconds_sum ${count / 100}
proxy_active_mappings 42
proxy_config_revision 7
proxy_import_jobs_total 2
proxy_import_records_processed_total 100
proxy_import_validation_errors_total 3
`,
    });
  });
  await page.setViewportSize({ width: 1440, height: 1000 });
  await page.goto("/monitoring");
  const routing = page.getByRole("button", { name: "Routing", exact: true });
  await expect(routing).toHaveAttribute("aria-expanded", "true");
  await routing.click();
  await expect(
    page.getByRole("link", { name: "Mappings", exact: true }),
  ).toBeHidden();
  await routing.focus();
  await page.keyboard.press("Enter");
  await expect(
    page.getByRole("link", { name: "Mappings", exact: true }),
  ).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "Import activity" }),
  ).toBeVisible();
  await expect(page.getByRole("img", { name: /^Request traffic/ })).toBeVisible(
    { timeout: 12000 },
  );
  await expect(
    page.getByRole("img", { name: /^Average response time/ }),
  ).toBeVisible();
  await expect.poll(() => count, { timeout: 12000 }).toBeGreaterThanOrEqual(30);
  await page.getByRole("button", { name: "Pause live updates" }).click();
  await expect(page.locator(".monitor-status")).toContainText("Updates paused");
  const pausedCount = count;
  await page.waitForTimeout(5500);
  expect(count).toBe(pausedCount);
  await page
    .getByText("Recent samples · exact chart values", { exact: true })
    .click();
  await expect(
    page.getByRole("cell", { name: "10", exact: true }).first(),
  ).toBeVisible();
  await page.screenshot({
    path: "docs/verification/monitoring-desktop.png",
    fullPage: true,
  });
  for (const width of [375, 850]) {
    await page.setViewportSize({ width, height: 900 });
    expect(
      await page.evaluate(
        () => document.documentElement.scrollWidth <= innerWidth,
      ),
    ).toBe(true);
  }
  await page.setViewportSize({ width: 375, height: 812 });
  await page.screenshot({
    path: "docs/verification/monitoring-mobile.png",
    fullPage: true,
  });
  unavailable = true;
  await page.getByRole("button", { name: "Resume live updates" }).click();
  await expect(page.locator(".monitor-status")).toContainText(
    "Refresh failed",
    { timeout: 20000 },
  );
  await expect(page.getByRole("alert")).toContainText("503");
  unavailable = false;
  await expect(page.locator(".monitor-status")).toContainText("Auto-refresh", {
    timeout: 15000,
  });
});
