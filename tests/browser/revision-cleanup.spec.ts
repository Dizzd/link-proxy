import { test, expect } from "@playwright/test";

test("revision cleanup previews, cancels, handles stale history and disables concurrent actions", async ({
  page,
}) => {
  await page.route("**/api/v1/auth/session", (route) =>
    route.fulfill({
      json: {
        user: { id: "cleanup-test", username: "cleanup-test" },
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
        active_revision: 120,
        database_revision: 120,
      },
    }),
  );
  let activeRevision = 119;
  await page.route("**/api/v1/config/active", (route) =>
    route.fulfill({
      json: {
        revision: activeRevision,
        draft_generation: 0,
        configuration: {},
      },
    }),
  );
  await page.route("**/api/v1/config/revisions", (route) =>
    route.fulfill({
      json: {
        items: [
          { version: 120, status: "active", created_at: "2026-11-05 10:00:00" },
          {
            version: 119,
            status: "superseded",
            created_at: "2026-11-04 10:00:00",
          },
        ],
      },
    }),
  );
  let empty = false;
  await page.route("**/api/v1/config/revisions/prune/preview", (route) => {
    activeRevision = 120;
    const keep = route.request().postDataJSON().keep_latest;
    const count = empty ? 0 : 120 - keep;
    return route.fulfill({
      json: {
        keep_latest: keep,
        database_revision: 120,
        active_revision: 120,
        total_revisions: empty ? keep : 120,
        eligible_count: count,
        versions: Array.from({ length: count }, (_, i) => i + 1),
      },
    });
  });
  let commits = 0;
  let release: (() => void) | undefined;
  await page.route("**/api/v1/config/revisions/prune", async (route) => {
    commits++;
    expect(route.request().headers()["x-csrf-token"]).toBe("fixture");
    expect(route.request().postDataJSON().keep_latest).toBe(90);
    if (commits === 1) {
      await new Promise<void>((resolve) => {
        release = resolve;
      });
      return route.fulfill({
        status: 409,
        json: {
          error: {
            message:
              "revision conflict: cleanup preview changed; preview again before deleting",
          },
        },
      });
    }
    empty = true;
    return route.fulfill({ json: { deleted_count: 30 } });
  });
  await page.goto("/revisions");
  const keep = page.getByLabel("Latest revisions to keep");
  const previewButton = page.getByRole("button", {
    name: "Preview cleanup",
    exact: true,
  });
  const deleteButton = page.getByRole("button", {
    name: /^Delete \d+ old revision/,
  });
  await expect(keep).toHaveValue("100");
  await keep.fill("0");
  await expect(previewButton).toBeDisabled();
  await keep.fill("100");
  await previewButton.click();
  await expect(deleteButton).toHaveText("Delete 20 old revision(s)");
  await keep.fill("90");
  await expect(deleteButton).toHaveCount(0);
  await previewButton.click();
  await expect(deleteButton).toHaveText("Delete 30 old revision(s)");
  page.once("dialog", (dialog) => dialog.dismiss());
  await deleteButton.click();
  expect(commits).toBe(0);
  await expect(deleteButton).toBeEnabled();
  page.once("dialog", async (dialog) => {
    expect(dialog.message()).toContain("Permanently delete 30");
    await dialog.accept();
  });
  await deleteButton.click();
  await expect.poll(() => typeof release).toBe("function");
  await expect(keep).toBeDisabled();
  await expect(
    page.getByRole("button", { name: "Publish draft", exact: true }),
  ).toBeDisabled();
  await expect(
    page.getByRole("button", { name: "Restore", exact: true }).first(),
  ).toBeDisabled();
  release!();
  await expect(page.getByRole("alert")).toContainText("preview again");
  await expect(deleteButton).toHaveCount(0);
  await previewButton.click();
  page.once("dialog", (dialog) => dialog.accept());
  await deleteButton.click();
  await expect(
    page.getByText("30 revision(s) deleted.", { exact: false }),
  ).toBeVisible();
  await previewButton.click();
  await expect(
    page.getByText("No old revisions are eligible for deletion."),
  ).toBeVisible();
  expect(commits).toBe(2);
  await page.emulateMedia({ reducedMotion: "reduce" });
  for (const size of [
    { width: 1440, height: 1000 },
    { width: 375, height: 812 },
    { width: 812, height: 375 },
  ]) {
    await page.setViewportSize(size);
    expect(
      await page.evaluate(
        () => document.documentElement.scrollWidth <= innerWidth,
      ),
    ).toBe(true);
    await page.screenshot({
      path: `docs/verification/revision-cleanup-${size.width}.png`,
      fullPage: true,
    });
  }
});
