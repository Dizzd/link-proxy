let csrfToken = "";
export function setCsrfToken(value: string) {
  csrfToken = value;
}
export function getCsrfToken() {
  return csrfToken;
}
export function clearSession() {
  csrfToken = "";
  if (typeof window !== "undefined")
    window.dispatchEvent(new Event("admin-session-expired"));
}
async function fetchAuthenticated(
  path: string,
  init: RequestInit = {},
): Promise<Response> {
  const url =
    path.startsWith("/api/") || path === "/metrics" ? path : `/api/v1${path}`;
  const headers = new Headers(init.headers);
  const method = (init.method || "GET").toUpperCase();
  if (init.body && !(init.body instanceof FormData))
    headers.set("Content-Type", "application/json");
  if (!["GET", "HEAD", "OPTIONS"].includes(method) && csrfToken)
    headers.set("X-CSRF-Token", csrfToken);
  const response = await fetch(url, {
    ...init,
    headers,
    credentials: "same-origin",
  });
  if (!response.ok) {
    if (response.status === 401 && path !== "/auth/login") clearSession();
    const text = await response.text();
    let message = text;
    try {
      message = JSON.parse(text).error?.message || text;
    } catch {}
    throw new Error(`${response.status}: ${message}`);
  }
  return response;
}
export async function request<T>(
  path: string,
  init: RequestInit = {},
): Promise<T> {
  const response = await fetchAuthenticated(path, init);
  if (response.status === 204) return undefined as T;
  return (await response.json()) as T;
}
export async function requestText(path: string) {
  return (await fetchAuthenticated(path)).text();
}
export const send = <T>(
  path: string,
  method: string,
  body?: unknown,
  headers?: HeadersInit,
) =>
  request<T>(path, {
    method,
    body: body === undefined ? undefined : JSON.stringify(body),
    headers,
  });
export async function download(path: string, name: string) {
  const response = await fetchAuthenticated(path);
  const url = URL.createObjectURL(await response.blob());
  const link = document.createElement("a");
  link.href = url;
  link.download = name;
  link.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}
