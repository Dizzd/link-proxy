import { useQuery } from "@tanstack/react-query";
import { requestText } from "../api/client";
import { ErrorMessage } from "../components/common/Feedback";
export default function MonitoringPage() {
  const q = useQuery({
    queryKey: ["metrics"],
    queryFn: () => requestText("/metrics"),
    refetchInterval: 5000,
  });
  return (
    <>
      <h1>Monitoring</h1>
      <p>Live Prometheus metrics. Refreshes every five seconds.</p>
      <ErrorMessage error={q.error} />
      <pre>{q.data || "Loading metrics…"}</pre>
    </>
  );
}
