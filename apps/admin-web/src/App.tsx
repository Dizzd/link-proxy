import { useState, useEffect, lazy, Suspense } from "react";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import { useQueryClient } from "@tanstack/react-query";
import { Layout } from "./components/layout/Layout";
import { auth, type SessionInfo } from "./api/auth";
import { setCsrfToken } from "./api/client";
import { ErrorMessage } from "./components/common/Feedback";
const Dashboard = lazy(() => import("./pages/DashboardPage"));
const Mappings = lazy(() => import("./pages/MappingsPage"));
const Editor = lazy(() => import("./pages/MappingEditorPage"));
const Upstreams = lazy(() => import("./pages/UpstreamsPage"));
const Domains = lazy(() => import("./pages/DomainsPage"));
const Imports = lazy(() => import("./pages/ImportPage"));
const Exports = lazy(() => import("./pages/ExportPage"));
const Revisions = lazy(() => import("./pages/RevisionsPage"));
const Monitoring = lazy(() => import("./pages/MonitoringPage"));
const Settings = lazy(() => import("./pages/SettingsPage"));
export default function App() {
  const [session, setSession] = useState<SessionInfo | null>(null),
    [checking, setChecking] = useState(true),
    [username, setUsername] = useState(""),
    [password, setPassword] = useState(""),
    [error, setError] = useState<unknown>(null),
    [busy, setBusy] = useState(false);
  const client = useQueryClient();
  useEffect(() => {
    let mounted = true;
    const expired = () => {
      setCsrfToken("");
      client.clear();
      setSession(null);
    };
    window.addEventListener("admin-session-expired", expired);
    void auth
      .session()
      .then((value) => {
        if (mounted) setSession(value);
      })
      .catch(() => {
        if (mounted) setSession(null);
      })
      .finally(() => {
        if (mounted) setChecking(false);
      });
    return () => {
      mounted = false;
      window.removeEventListener("admin-session-expired", expired);
    };
  }, [client]);
  async function signOut() {
    setBusy(true);
    setError(null);
    try {
      await auth.logout();
      client.clear();
      setSession(null);
    } catch (error) {
      setError(error);
    } finally {
      setBusy(false);
    }
  }
  if (checking)
    return (
      <main className="login">
        <p role="status">Checking your session...</p>
      </main>
    );
  if (!session)
    return (
      <main className="login">
        <form
          className="panel"
          onSubmit={async (e) => {
            e.preventDefault();
            setBusy(true);
            setError(null);
            try {
              setSession(await auth.login(username, password));
              setPassword("");
            } catch (error) {
              setPassword("");
              setError(error);
            } finally {
              setBusy(false);
            }
          }}
        >
          <div className="eyebrow">DYNAMIC URL PROXY</div>
          <h1>Sign in to your control plane</h1>
          <p>
            Manage mappings and publish configuration with your administrator
            account.
          </p>
          <label>
            Username
            <input
              required
              autoComplete="username"
              maxLength={64}
              value={username}
              onChange={(e) => setUsername(e.target.value)}
            />
          </label>
          <label>
            Password
            <input
              required
              type="password"
              autoComplete="current-password"
              maxLength={1024}
              value={password}
              onChange={(e) => setPassword(e.target.value)}
            />
          </label>
          <ErrorMessage error={error} />
          <button disabled={busy}>{busy ? "Signing in..." : "Sign in"}</button>
        </form>
      </main>
    );
  return (
    <BrowserRouter>
      <ErrorMessage error={error} />
      <Suspense fallback={<p role="status">Loading page...</p>}>
        <Routes>
          <Route
            element={
              <Layout
                username={session.user.username}
                signingOut={busy}
                logout={() => void signOut()}
              />
            }
          >
            <Route index element={<Dashboard />} />
            <Route path="mappings" element={<Mappings />} />
            <Route path="mappings/:id" element={<Editor />} />
            <Route path="upstreams" element={<Upstreams />} />
            <Route path="domains" element={<Domains />} />
            <Route path="imports" element={<Imports />} />
            <Route path="exports" element={<Exports />} />
            <Route path="revisions" element={<Revisions />} />
            <Route path="monitoring" element={<Monitoring />} />
            <Route path="settings" element={<Settings />} />
            <Route path="*" element={<p>Page not found.</p>} />
          </Route>
        </Routes>
      </Suspense>
    </BrowserRouter>
  );
}
