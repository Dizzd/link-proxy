import { request, send, setCsrfToken } from "./client";
export interface SessionInfo {
  user: { id: string; username: string };
  csrf_token: string;
  expires_at: number;
}
function remember(session: SessionInfo) {
  setCsrfToken(session.csrf_token);
  return session;
}
export const auth = {
  session: () => request<SessionInfo>("/auth/session").then(remember),
  login: (username: string, password: string) =>
    send<SessionInfo>("/auth/login", "POST", { username, password }).then(
      remember,
    ),
  logout: () => send<void>("/auth/logout", "POST").then(() => setCsrfToken("")),
  changePassword: (current_password: string, new_password: string) =>
    send<void>("/auth/password", "POST", { current_password, new_password }),
};
