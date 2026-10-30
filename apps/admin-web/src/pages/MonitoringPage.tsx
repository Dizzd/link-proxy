import { useEffect, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { requestText } from "../api/client";
import { ErrorMessage } from "../components/common/Feedback";
import {
  formatMetric,
  meanLatency,
  parseMetrics,
  trends,
  type Sample,
} from "../features/monitoring/metrics";
import { TrendChart } from "../features/monitoring/TrendChart";

export default function MonitoringPage() {
  const [paused, setPaused] = useState(false);
  const [samples, setSamples] = useState<Sample[]>([]);
  const q = useQuery({
    queryKey: ["metrics"],
    queryFn: async () => {
      const raw = await requestText("/metrics");
      return { raw, time: Date.now(), values: parseMetrics(raw) };
    },
    enabled: !paused,
    refetchInterval: paused ? false : 5000,
  });
  useEffect(() => {
    if (q.data) {
      const sample = q.data;
      setSamples((previous) =>
        previous.at(-1)?.time === sample.time
          ? previous
          : [...previous, sample].slice(-60),
      );
    }
  }, [q.data]);
  const values = q.data?.values ?? {};
  const points = trends(samples);
  const rate = points.at(-1)?.rate;
  return (
    <>
      <div className="titlebar">
        <div>
          <h1>Monitoring</h1>
          <p className="subtitle">
            Traffic and processing performance at a glance.
          </p>
        </div>
        <button className="quiet" onClick={() => setPaused(!paused)}>
          {paused ? "Resume live updates" : "Pause live updates"}
        </button>
      </div>
      <p className="monitor-status" role="status">
        {q.isError
          ? "Refresh failed · Showing last successful data"
          : paused
            ? "Updates paused"
            : q.isPending
              ? "Loading metrics…"
              : "Auto-refresh every 5 seconds"}
        {q.data &&
          ` · Last updated ${new Date(q.data.time).toLocaleTimeString()}`}
      </p>
      <ErrorMessage error={q.error} />
      <div className="stats">
        {[
          [
            "Total requests",
            formatMetric(values.proxy_requests_total),
            "Since the proxy process started",
          ],
          [
            "Request rate",
            `${formatMetric(rate, 2)} req/s`,
            "During the latest refresh interval",
          ],
          [
            "Average response time",
            `${formatMetric(meanLatency(values), 2)} ms`,
            "Completed requests since process start",
          ],
          [
            "Active mappings",
            formatMetric(values.proxy_active_mappings),
            `Published revision: ${formatMetric(values.proxy_config_revision)}`,
          ],
        ].map(([label, value, hint]) => (
          <div className="stat" key={label}>
            <span>{label}</span>
            <strong>{value}</strong>
            <small>{hint}</small>
          </div>
        ))}
      </div>
      <p className="chart-caption">
        Live history starts when this page opens and keeps the latest 60 samples
        (about 5 minutes). Gaps indicate missing measurements or counter resets.
      </p>
      <div className="monitor-charts">
        <TrendChart
          title="Request traffic"
          unit="requests / second"
          field="rate"
          points={points}
        />
        <TrendChart
          title="Average response time"
          unit="milliseconds"
          field="latency"
          points={points}
        />
      </div>
      <section className="panel">
        <h2>Import activity</h2>
        <p>
          Cumulative totals since the proxy process started. Validation errors
          count records, not failed jobs.
        </p>
        <dl className="metric-breakdown">
          <div>
            <dt>Jobs accepted</dt>
            <dd>{formatMetric(values.proxy_import_jobs_total)}</dd>
          </div>
          <div>
            <dt>Records processed</dt>
            <dd>{formatMetric(values.proxy_import_records_processed_total)}</dd>
          </div>
          <div>
            <dt>Validation errors</dt>
            <dd>{formatMetric(values.proxy_import_validation_errors_total)}</dd>
          </div>
        </dl>
      </section>
      <details className="panel">
        <summary>Recent samples · exact chart values</summary>
        <div className="table-scroll">
          <table>
            <caption>
              Latest 60 samples. A dash means no measurement is available.
            </caption>
            <thead>
              <tr>
                <th scope="col">Time</th>
                <th scope="col">Requests / second</th>
                <th scope="col">Average response (ms)</th>
              </tr>
            </thead>
            <tbody>
              {[...points].reverse().map((point) => (
                <tr key={point.time}>
                  <td>{new Date(point.time).toLocaleTimeString()}</td>
                  <td>{formatMetric(point.rate, 2)}</td>
                  <td>{formatMetric(point.latency, 2)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </details>
      <details className="panel">
        <summary>Advanced · raw Prometheus metrics</summary>
        <pre>{q.data?.raw ?? "No metrics received yet."}</pre>
      </details>
    </>
  );
}
