import { useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
export function useOperation() {
  const client = useQueryClient();
  const [error, setError] = useState<unknown>(null);
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState("");
  async function run(
    action: () => Promise<unknown>,
    message = "Changes saved",
  ) {
    setError(null);
    setBusy(true);
    setNotice("");
    try {
      await action();
      await client.invalidateQueries();
      setNotice(message);
    } catch (e) {
      setError(e);
    } finally {
      setBusy(false);
    }
  }
  return { run, error, busy, notice };
}
