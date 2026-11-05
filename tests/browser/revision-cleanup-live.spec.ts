import { test, expect } from "@playwright/test";

test("cleanup deletes real history while preserving the active configuration and audit", async ({
  page,
}) => {
  const username = process.env.ADMIN_USERNAME;
  const password = process.env.ADMIN_PASSWORD;
  if (!username || !password)
    throw new Error("Use an isolated backend through scripts/browser_test.py");
  await page.goto("/");
  await page.getByLabel("Username", { exact: true }).fill(username);
  await page.getByLabel("Password", { exact: true }).fill(password);
  await page.getByRole("button", { name: "Sign in", exact: true }).click();
  await expect(
    page.getByRole("heading", { name: "Traffic overview" }),
  ).toBeVisible();
  const session = await (await page.request.get("/api/v1/auth/session")).json();
  const headers = { "X-CSRF-Token": session.csrf_token };
  const created = await page.request.post("/api/v1/mappings", {
    headers,
    data: {
      id: "cleanup-stable-mapping",
      mapping_key: "cleanup-stable-mapping",
      source_matcher: {
        hosts: ["example.com"],
        path_type: "exact",
        path_value: "/preserved-url",
      },
      default_action: {
        type: "redirect",
        status: 302,
        target_url: "https://destination.example/preserved",
      },
    },
  });
  expect(created.status()).toBe(201);
  for (let i = 0; i < 3; i++) {
    const current = await (
      await page.request.get("/api/v1/config/active")
    ).json();
    const response = await page.request.post("/api/v1/config/publish", {
      headers,
      data: { expected_revision: current.revision },
    });
    expect(response.ok()).toBe(true);
  }
  const before = await (await page.request.get("/api/v1/config/active")).json();
  const history = await (
    await page.request.get("/api/v1/config/revisions")
  ).json();
  const oldVersion = history.items.at(-1).version;
  await page.getByRole("link", { name: "Revisions", exact: true }).click();
  await page.getByLabel("Latest revisions to keep").fill("1");
  await page
    .getByRole("button", { name: "Preview cleanup", exact: true })
    .click();
  const remove = page.getByRole("button", { name: /^Delete \d+ old revision/ });
  await expect(remove).toBeVisible();
  page.once("dialog", (dialog) => dialog.accept());
  await remove.click();
  await expect(
    page.getByText("Revision cleanup completed", { exact: true }),
  ).toBeVisible();
  const after = await (await page.request.get("/api/v1/config/active")).json();
  expect(after).toEqual(before);
  const remaining = await (
    await page.request.get("/api/v1/config/revisions")
  ).json();
  expect(remaining.items.map((r: { version: number }) => r.version)).toEqual([
    before.revision,
  ]);
  const restore = await page.request.post(
    `/api/v1/config/revisions/${oldVersion}/restore`,
    { headers, data: { expected_revision: before.revision } },
  );
  expect(restore.status()).toBe(404);
  const audit = await (await page.request.get("/api/v1/audit-events")).json();
  expect(
    audit.items.some(
      (event: { operation: string; actor: string }) =>
        event.operation === "revision_prune" && event.actor === username,
    ),
  ).toBe(true);
});
