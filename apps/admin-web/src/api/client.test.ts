import { afterEach, describe, it, expect, vi } from "vitest";
import { request, send, setCsrfToken, getCsrfToken } from "./client";
afterEach(() => {
  setCsrfToken("");
  vi.unstubAllGlobals();
});
describe("cookie and CSRF API boundary", () => {
  it("sends same-origin cookies and CSRF on JSON mutations without bearer credentials", async () => {
    const fetcher = vi
      .fn()
      .mockResolvedValue(new Response('{"ok":true}', { status: 200 }));
    vi.stubGlobal("fetch", fetcher);
    setCsrfToken("csrf-value");
    await send("/config/publish", "POST", { expected_revision: 2 });
    const [url, init] = fetcher.mock.calls[0];
    expect(url).toBe("/api/v1/config/publish");
    expect(init.credentials).toBe("same-origin");
    expect(init.headers.has("Authorization")).toBe(false);
    expect(init.headers.get("X-CSRF-Token")).toBe("csrf-value");
    expect(JSON.parse(init.body).expected_revision).toBe(2);
  });
  it("preserves multipart boundary generation while authenticating mutations", async () => {
    const fetcher = vi
      .fn()
      .mockResolvedValue(new Response('{"id":"job"}', { status: 202 }));
    vi.stubGlobal("fetch", fetcher);
    setCsrfToken("csrf-value");
    await request("/imports", { method: "POST", body: new FormData() });
    expect(fetcher.mock.calls[0][1].headers.has("Content-Type")).toBe(false);
    expect(fetcher.mock.calls[0][1].headers.get("X-CSRF-Token")).toBe(
      "csrf-value",
    );
  });
  it("does not add CSRF headers to safe reads", async () => {
    const fetcher = vi.fn().mockResolvedValue(new Response("{}"));
    vi.stubGlobal("fetch", fetcher);
    setCsrfToken("csrf-value");
    await request("/config/active");
    expect(fetcher.mock.calls[0][1].headers.has("X-CSRF-Token")).toBe(false);
  });
  it("clears session state on expired authentication", async () => {
    setCsrfToken("expired");
    vi.stubGlobal(
      "fetch",
      vi
        .fn()
        .mockResolvedValue(
          new Response('{"error":{"message":"Session expired"}}', {
            status: 401,
          }),
        ),
    );
    await expect(request("/config/active")).rejects.toThrow(
      "401: Session expired",
    );
    expect(getCsrfToken()).toBe("");
  });
  it("surfaces structured conflicts", async () => {
    vi.stubGlobal(
      "fetch",
      vi
        .fn()
        .mockResolvedValue(
          new Response('{"error":{"message":"revision conflict"}}', {
            status: 409,
          }),
        ),
    );
    await expect(request("/config/publish")).rejects.toThrow(
      "409: revision conflict",
    );
  });
  it("handles empty deletion responses", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue(new Response(null, { status: 204 })),
    );
    await expect(send("/mappings/id", "DELETE")).resolves.toBeUndefined();
  });
});
