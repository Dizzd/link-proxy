import { test, expect } from "@playwright/test";
test("real administration workflow, routing and password session lifecycle", async ({
  page,
  request,
}) => {
  const username = process.env.ADMIN_USERNAME;
  const password = process.env.ADMIN_PASSWORD;
  if (!username || !password)
    throw new Error(
      "Set ADMIN_USERNAME and ADMIN_PASSWORD for the isolated backend",
    );
  await page.goto("/");
  await page.getByLabel("Username", { exact: true }).fill(username);
  await page.getByLabel("Password", { exact: true }).fill("invalid");
  await page.getByRole("button", { name: "Sign in", exact: true }).click();
  await expect(page.getByRole("alert")).toContainText(
    "Invalid username or password",
  );
  await page.getByLabel("Password", { exact: true }).fill(password);
  await page.getByRole("button", { name: "Sign in", exact: true }).click();
  await expect(
    page.getByRole("heading", { name: "Traffic overview" }),
  ).toBeVisible();
  await expect(
    page.getByRole("status", { name: "Configuration cache" }),
  ).toContainText("Memory only");
  await page.getByRole("link", { name: "Mappings", exact: true }).click();
  await page.getByRole("link", { name: "Create mapping" }).click();
  await page.getByLabel("Mapping key").fill("browser-mapping");
  await page.getByLabel("Name", { exact: true }).fill("Browser mapping");
  await page.getByLabel("Hosts (comma separated)").fill("abc.example.com");
  await page.getByLabel("Path", { exact: true }).fill("/browser");
  await page.getByLabel("Target URL").fill("https://fallback.example/");
  await page.getByRole("button", { name: "Add rule", exact: true }).click();
  await page.getByLabel("Rule name").fill("Mobile");
  await page.getByLabel("Target URL").nth(1).fill("https://mobile.example/");
  await page.getByRole("button", { name: "Save draft mapping" }).click();
  await expect(
    page.getByRole("link", { name: "browser-mapping" }),
  ).toBeVisible();
  await page.getByLabel("Search", { exact: true }).fill("browser");
  await expect(
    page.getByRole("link", { name: "browser-mapping" }),
  ).toBeVisible();
  await page.getByRole("link", { name: "Revisions", exact: true }).click();
  await page
    .getByRole("button", { name: "Publish draft", exact: true })
    .click();
  await expect(page.locator("#content").getByRole("status")).toContainText(
    "Draft published",
  );
  const response = await request.get(
    `${process.env.PROXY_BROWSER_URL || "http://127.0.0.1:8080"}/browser`,
    {
      headers: { Host: "abc.example.com", "User-Agent": "iPhone Mobile" },
      maxRedirects: 0,
    },
  );
  expect(response.status()).toBe(302);
  expect(response.headers().location).toBe("https://mobile.example/");
  await page.getByRole("link", { name: "Overview", exact: true }).click();
  await page.getByLabel("host", { exact: true }).fill("abc.example.com");
  await page.getByLabel("path", { exact: true }).fill("/browser");
  await page.getByLabel("user agent").fill("iPhone Mobile");
  await page.getByRole("button", { name: "Simulate", exact: true }).click();
  await expect(page.locator("pre")).toContainText("https://mobile.example/");
  await page.getByRole("link", { name: "Bulk import", exact: true }).click();
  await page
    .getByLabel("File", { exact: true })
    .setInputFiles("tests/fixtures/mappings.csv");
  await page.getByRole("button", { name: "Upload and dry-run" }).click();
  await expect(
    page.getByRole("heading", { name: /Preview.*ready/ }),
  ).toBeVisible({ timeout: 15000 });
  await page.getByLabel("I reviewed the preview").check();
  await page.getByRole("button", { name: "Commit one revision" }).click();
  await expect(
    page.getByRole("heading", { name: /Preview.*completed/ }),
  ).toBeVisible({ timeout: 15000 });
  await page.getByRole("link", { name: "Bulk export", exact: true }).click();
  const downloading = page.waitForEvent("download");
  await page.getByRole("button", { name: "Download export" }).click();
  expect((await downloading).suggestedFilename()).toBe("mappings.json");
  await page.getByRole("link", { name: "Settings", exact: true }).click();
  await page.getByRole("button", { name: "Reconcile from database" }).click();
  await expect(page.locator("#content").getByRole("status")).toContainText(
    "Reconciliation completed",
  );
  await page.getByRole("link", { name: "Monitoring", exact: true }).click();
  await expect(
    page.getByRole("heading", { name: "Monitoring", exact: true }),
  ).toBeVisible();
  await expect(page.locator("pre")).toContainText("proxy_requests_total");
  await page.setViewportSize({ width: 390, height: 844 });
  await page.getByRole("link", { name: "Mappings", exact: true }).click();
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= window.innerWidth,
    ),
  ).toBe(true);
  await page.screenshot({
    path: "docs/verification/admin-mobile.png",
    fullPage: true,
  });
  await page.setViewportSize({ width: 1440, height: 1000 });
  await page.getByRole("link", { name: "Overview", exact: true }).click();
  await page.screenshot({
    path: "docs/verification/admin-desktop.png",
    fullPage: true,
  });
  await page.reload();
  await expect(
    page.getByRole("heading", { name: "Traffic overview" }),
  ).toBeVisible();
  expect(await page.evaluate(() => localStorage.length)).toBe(0);
  await page.getByRole("link", { name: "Settings", exact: true }).click();
  await expect(page.locator("pre").first()).toContainText(
    process.env.EXPECT_DATABASE_BACKEND || "sqlite",
  );
  await page.getByLabel("Current password", { exact: true }).fill(password);
  const newPassword = password + "-changed";
  await page.getByLabel("New password", { exact: true }).fill(newPassword);
  await page
    .getByLabel("Confirm new password", { exact: true })
    .fill(newPassword);
  await page
    .getByRole("button", { name: "Change password", exact: true })
    .click();
  await expect(
    page.getByRole("heading", { name: "Sign in to your control plane" }),
  ).toBeVisible();
  await page.getByLabel("Username", { exact: true }).fill(username);
  await page.getByLabel("Password", { exact: true }).fill(password);
  await page.getByRole("button", { name: "Sign in", exact: true }).click();
  await expect(page.getByRole("alert")).toContainText(
    "Invalid username or password",
  );
  await page.getByLabel("Password", { exact: true }).fill(newPassword);
  await page.getByRole("button", { name: "Sign in", exact: true }).click();
  await expect(
    page.getByRole("heading", { name: "Settings & cache" }),
  ).toBeVisible();
  const cookies = await page.context().cookies();
  const sessionCookie = cookies.find(
    (cookie) => cookie.name === "dup_admin_session",
  );
  expect(sessionCookie?.httpOnly).toBe(true);
  expect(sessionCookie?.sameSite).toBe("Strict");
  await page.getByRole("button", { name: "Sign out", exact: true }).click();
  await expect(
    page.getByRole("heading", { name: "Sign in to your control plane" }),
  ).toBeVisible();
  await page.reload();
  await expect(
    page.getByRole("heading", { name: "Sign in to your control plane" }),
  ).toBeVisible();
});
