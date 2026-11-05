import { test, expect } from "@playwright/test";

test("bulk selection is page scoped, cancellable and supports partial failure retry", async ({
  page,
}) => {
  await page.route("**/api/v1/auth/session", (route) =>
    route.fulfill({
      json: {
        user: { id: "bulk-test", username: "bulk-test" },
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
  let items = Array.from({ length: 26 }, (_, i) => ({
    id: `id-${i + 1}`,
    mapping_key: `mapping-${i + 1}`,
    name: `Mapping ${i + 1}`,
    source_matcher: {
      hosts: ["example.com"],
      path_value: `/path-${i + 1}`,
      path_type: "exact",
    },
    default_action: { type: "redirect" },
    rules: [],
    enabled: true,
  }));
  const deleted: string[] = [];
  let fail = true;
  let releaseFirstDeletion: (() => void) | undefined;
  await page.route("**/api/v1/mappings**", async (route) => {
    const request = route.request();
    if (request.method() === "DELETE") {
      expect(request.headers()["x-csrf-token"]).toBe("fixture");
      const id = new URL(request.url()).pathname.split("/").pop()!;
      deleted.push(id);
      if (id === "id-1")
        await new Promise<void>((resolve) => {
          releaseFirstDeletion = resolve;
        });
      if (fail && id === "id-2")
        return route.fulfill({
          status: 500,
          json: { error: { message: "Fixture failure" } },
        });
      items = items.filter((item) => item.id !== id);
      return route.fulfill({ status: 204 });
    }
    const url = new URL(request.url());
    const offset = Number(url.searchParams.get("offset") || 0);
    return route.fulfill({
      json: { items: items.slice(offset, offset + 25), total: items.length },
    });
  });
  await page.goto("/mappings");
  const selectAll = page.getByRole("checkbox", {
    name: "Select all mappings on this page",
  });
  const bulk = page.getByRole("button", { name: /^Delete selected/ });
  await expect(bulk).toBeDisabled();
  await page
    .getByRole("checkbox", { name: "Select mapping-1", exact: true })
    .check();
  await expect(selectAll).toHaveJSProperty("indeterminate", true);
  await selectAll.check();
  await expect(bulk).toHaveText("Delete selected (25)");
  await page.getByRole("button", { name: "Clear selection" }).click();
  await expect(bulk).toBeDisabled();
  await selectAll.check();
  await page.getByRole("button", { name: "Next", exact: true }).click();
  await expect(
    page.getByRole("link", { name: "mapping-26", exact: true }),
  ).toBeVisible();
  await expect(bulk).toBeDisabled();
  await page.getByRole("button", { name: "Previous", exact: true }).click();
  await expect(bulk).toBeDisabled();
  await selectAll.check();
  await page.getByLabel("Search", { exact: true }).fill("mapping");
  await expect(bulk).toBeDisabled();
  await expect(selectAll).toBeEnabled();
  await page
    .getByRole("checkbox", { name: "Select mapping-1", exact: true })
    .check();
  await page
    .getByRole("checkbox", { name: "Select mapping-2", exact: true })
    .check();
  await page
    .getByRole("checkbox", { name: "Select mapping-3", exact: true })
    .check();
  page.once("dialog", (dialog) => dialog.dismiss());
  await bulk.click();
  expect(deleted).toEqual([]);
  await expect(bulk).toHaveText("Delete selected (3)");
  page.once("dialog", async (dialog) => {
    expect(dialog.message()).toContain("3 selected mapping(s)");
    await dialog.accept();
  });
  await bulk.click();
  await expect(page.getByRole("button", { name: "Deleting…" })).toBeDisabled();
  await expect(page.getByLabel("Search", { exact: true })).toBeDisabled();
  await expect(
    page.getByRole("button", { name: "Next", exact: true }),
  ).toBeDisabled();
  await expect.poll(() => typeof releaseFirstDeletion).toBe("function");
  releaseFirstDeletion!();
  await expect(page.getByRole("alert")).toContainText("Deletion stopped");
  await expect(
    page.getByText("1 of 3 mapping(s) deleted", { exact: false }),
  ).toBeVisible();
  expect(deleted).toEqual(["id-1", "id-2"]);
  await expect(bulk).toHaveText("Delete selected (2)");
  await expect(
    page.getByRole("link", { name: "mapping-1", exact: true }),
  ).toHaveCount(0);
  fail = false;
  page.once("dialog", (dialog) => dialog.accept());
  await bulk.click();
  await expect(
    page.getByText("2 of 2 mapping(s) deleted", { exact: false }),
  ).toBeVisible();
  await expect(bulk).toBeDisabled();
  expect(deleted).toEqual(["id-1", "id-2", "id-2", "id-3"]);
  items.push(
    ...Array.from({ length: 3 }, (_, i) => ({
      ...items[0],
      id: `id-${27 + i}`,
      mapping_key: `mapping-${27 + i}`,
    })),
  );
  await page.reload();
  await page.getByRole("button", { name: "Next", exact: true }).click();
  await expect(
    page.getByRole("link", { name: "mapping-29", exact: true }),
  ).toBeVisible();
  await selectAll.check();
  page.once("dialog", (dialog) => dialog.accept());
  await bulk.click();
  await expect(
    page.getByRole("link", { name: "mapping-4", exact: true }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Previous", exact: true }),
  ).toBeDisabled();
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
      path: `docs/verification/mapping-bulk-delete-${size.width}.png`,
      fullPage: true,
    });
  }
  await page.screenshot({
    path: "docs/verification/mapping-bulk-delete-mobile.png",
    fullPage: true,
  });
});
