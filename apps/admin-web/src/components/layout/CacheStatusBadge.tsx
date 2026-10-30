import { useCacheStatus } from "../../hooks/useCacheStatus";
import type { CacheStatus } from "../../types";

export function CacheStatusView({
  data,
  pending,
  error,
}: {
  data?: CacheStatus;
  pending: boolean;
  error: boolean;
}) {
  let label: string;
  let detail: string | undefined;
  if (error) {
    label = "Cache unknown";
    detail = "Status unavailable";
  } else if (!data && pending) {
    label = "Checking cache...";
  } else if (data?.mode === "memory") {
    label = "Memory only";
    detail = "Local routing snapshot";
  } else if (data?.mode === "memory_redis") {
    label = "Memory + Redis";
    detail = data.connected
      ? "Redis connected"
      : data.error || data.last_sync
        ? "Redis unavailable; local routing continues."
        : "Awaiting Redis synchronization";
  } else {
    label = "Cache unknown";
    detail = "Status unavailable";
  }
  return (
    <div
      className="cache-status"
      role="status"
      aria-label="Configuration cache"
      aria-atomic="true"
      title="Request routing always uses a local memory snapshot. Redis optionally synchronizes configuration."
    >
      <span className="badge">{label}</span>
      {detail && <small>{detail}</small>}
    </div>
  );
}

export function CacheStatusBadge() {
  const status = useCacheStatus();
  return (
    <CacheStatusView
      data={status.data}
      pending={status.isPending}
      error={status.isError}
    />
  );
}
