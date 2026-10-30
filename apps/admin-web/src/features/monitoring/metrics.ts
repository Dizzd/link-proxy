export type Metrics = Record<string, number>;
export type Sample = { time: number; values: Metrics };
export type Trend = {
  time: number;
  rate: number | null;
  latency: number | null;
};

// Consume only unlabelled scalar samples emitted by our registry, never buckets.
export function parseMetrics(text: string): Metrics {
  const values: Metrics = {};
  for (const line of text.split(/\r?\n/)) {
    const match = line.match(
      /^([a-zA-Z_:][a-zA-Z0-9_:]*)\s+([^\s]+)(?:\s+\d+)?\s*$/,
    );
    if (match && Number.isFinite(Number(match[2])))
      values[match[1]] = Number(match[2]);
  }
  return values;
}

export function meanLatency(values: Metrics): number | null {
  const count = values.proxy_request_duration_seconds_count;
  const sum = values.proxy_request_duration_seconds_sum;
  return count > 0 && sum >= 0 ? (sum / count) * 1000 : null;
}

export function trends(samples: Sample[]): Trend[] {
  return samples.map((sample, index) => {
    const previous = samples[index - 1];
    const empty = { time: sample.time, rate: null, latency: null };
    if (!previous) return empty;
    const seconds = (sample.time - previous.time) / 1000;
    // Do not bridge outages, paused tabs, or process counter resets.
    if (seconds <= 0 || seconds > 15) return empty;
    const delta = (name: string) => sample.values[name] - previous.values[name];
    const requests = delta("proxy_requests_total");
    const count = delta("proxy_request_duration_seconds_count");
    const sum = delta("proxy_request_duration_seconds_sum");
    return {
      time: sample.time,
      rate:
        Number.isFinite(requests) && requests >= 0 ? requests / seconds : null,
      latency:
        count > 0 && sum >= 0 && Number.isFinite(sum)
          ? (sum / count) * 1000
          : null,
    };
  });
}

export function formatMetric(
  value: number | null | undefined,
  digits = 0,
): string {
  return value == null || !Number.isFinite(value)
    ? "—"
    : value.toLocaleString("en-US", { maximumFractionDigits: digits });
}
