import { describe, expect, it } from "vitest";
import {
  formatMetric,
  meanLatency,
  parseMetrics,
  trends,
  type Sample,
} from "./metrics";

const sample = (
  time: number,
  requests: number,
  count: number,
  sum: number,
): Sample => ({
  time,
  values: {
    proxy_requests_total: requests,
    proxy_request_duration_seconds_count: count,
    proxy_request_duration_seconds_sum: sum,
  },
});

describe("monitoring measurements", () => {
  it("reads scalar metrics without treating histogram buckets or non-finite values as scalars", () => {
    expect(
      parseMetrics(
        '# HELP proxy_requests_total Total\r\nproxy_requests_total 12\r\nproxy_request_duration_seconds_sum 2.5e-3\nproxy_request_duration_seconds_bucket{le="1"} 12\nbad NaN\nother +Inf',
      ),
    ).toEqual({
      proxy_requests_total: 12,
      proxy_request_duration_seconds_sum: 0.0025,
    });
  });
  it("uses elapsed time and histogram deltas, rather than cumulative totals, for charts", () => {
    const result = trends([
      sample(1000, 100, 50, 5),
      sample(6000, 120, 60, 5.5),
    ]);
    expect(result[0].rate).toBeNull();
    expect(result[1].rate).toBe(4);
    expect(result[1].latency).toBe(50);
    expect(meanLatency(sample(0, 0, 60, 5.5).values)).toBeCloseTo(91.6667);
  });
  it("shows zero traffic but no latency when no requests complete", () => {
    expect(trends([sample(0, 0, 0, 0), sample(5000, 0, 0, 0)])[1]).toEqual({
      time: 5000,
      rate: 0,
      latency: null,
    });
    expect(meanLatency({})).toBeNull();
    expect(meanLatency(sample(0, 0, 0, 0).values)).toBeNull();
  });
  it("breaks the trend across restarts, long refresh gaps and absent metrics", () => {
    expect(
      trends([sample(0, 100, 100, 5), sample(5000, 1, 1, 0.1)])[1],
    ).toEqual({ time: 5000, rate: null, latency: null });
    expect(
      trends([sample(0, 1, 1, 1), sample(20000, 20, 20, 2)])[1].rate,
    ).toBeNull();
    expect(
      trends([sample(0, 1, 1, 1), { time: 5000, values: {} }])[1].rate,
    ).toBeNull();
  });
  it("distinguishes missing measurements from zero", () => {
    expect(formatMetric(undefined)).toBe("—");
    expect(formatMetric(Infinity)).toBe("—");
    expect(formatMetric(0)).toBe("0");
  });
});
