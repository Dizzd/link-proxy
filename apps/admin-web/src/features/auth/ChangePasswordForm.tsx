import { useState } from "react";
import { auth } from "../../api/auth";
import { clearSession } from "../../api/client";
import { ErrorMessage } from "../../components/common/Feedback";
export function ChangePasswordForm() {
  const [current, setCurrent] = useState(""),
    [password, setPassword] = useState(""),
    [confirm, setConfirm] = useState(""),
    [busy, setBusy] = useState(false),
    [error, setError] = useState<unknown>(null);
  return (
    <form
      className="panel"
      onSubmit={async (e) => {
        e.preventDefault();
        setError(null);
        if (password !== confirm) {
          setError(new Error("New passwords do not match"));
          return;
        }
        setBusy(true);
        try {
          await auth.changePassword(current, password);
          setCurrent("");
          setPassword("");
          setConfirm("");
          clearSession();
        } catch (error) {
          setError(error);
        } finally {
          setBusy(false);
        }
      }}
    >
      <h2>Change password</h2>
      <p>Changing your password signs out all sessions, including this one.</p>
      <label>
        Current password
        <input
          type="password"
          required
          autoComplete="current-password"
          value={current}
          onChange={(e) => setCurrent(e.target.value)}
        />
      </label>
      <label>
        New password
        <input
          type="password"
          required
          minLength={12}
          maxLength={1024}
          autoComplete="new-password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
        />
      </label>
      <label>
        Confirm new password
        <input
          type="password"
          required
          minLength={12}
          maxLength={1024}
          autoComplete="new-password"
          value={confirm}
          onChange={(e) => setConfirm(e.target.value)}
        />
      </label>
      <ErrorMessage error={error} />
      <button disabled={busy}>
        {busy ? "Updating password..." : "Change password"}
      </button>
    </form>
  );
}
