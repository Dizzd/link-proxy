import { formatMetric, type Trend } from "./metrics";
import { useEffect, useRef, useState } from "react";

export function TrendChart({
  title,
  unit,
  field,
  points,
}: {
  title: string;
  unit: string;
  field: "rate" | "latency";
  points: Trend[];
}) {
  const container = useRef<HTMLDivElement>(null);
  const [width, setWidth] = useState(600);
  useEffect(() => {
    const element = container.current;
    if (!element) return;
    const observer = new ResizeObserver(([entry]) => {
      setWidth(Math.max(260, entry.contentRect.width));
    });
    observer.observe(element);
    return () => observer.disconnect();
  }, []);
  const values = points.flatMap((point) =>
    point[field] == null ? [] : [point[field]!],
  );
  const max = Math.max(...values, 1);
  const start = points[0]?.time ?? 0;
  const duration = Math.max((points.at(-1)?.time ?? start) - start, 1);
  const x = (time: number) => 50 + ((time - start) / duration) * (width - 70);
  const y = (value: number) => 180 - (value / max) * 150;
  let connected = false;
  const path = points
    .map((point) => {
      const value = point[field];
      if (value == null) {
        connected = false;
        return "";
      }
      const segment = `${connected ? "L" : "M"}${x(point.time)},${y(value)}`;
      connected = true;
      return segment;
    })
    .join(" ");
  return (
    <section className="panel trend-panel">
      <h2>{title}</h2>
      <p className="chart-caption">
        {unit} · Each point covers the previous refresh interval.
      </p>
      <div ref={container}>
        {values.length === 0 ? (
          <div className="chart-empty" role="status">
            {field === "rate"
              ? "Waiting for two successful refreshes to show traffic."
              : "No completed requests in recent intervals yet."}
          </div>
        ) : (
          <svg
            className={`trend-chart trend-${field}`}
            viewBox={`0 0 ${width} 220`}
            role="img"
            aria-label={`${title}. Latest measured value: ${formatMetric(values.at(-1), 2)} ${unit}. Exact values are available in Recent samples below.`}
          >
            {[0, 0.5, 1].map((fraction) => (
              <g key={fraction}>
                <line
                  x1="50"
                  x2={width - 20}
                  y1={y(max * fraction)}
                  y2={y(max * fraction)}
                  className="chart-grid"
                />
                <text x="42" y={y(max * fraction) + 4} textAnchor="end">
                  {formatMetric(max * fraction, 2)}
                </text>
              </g>
            ))}
            <path
              d={path}
              fill="none"
              stroke="currentColor"
              strokeWidth="2.5"
            />
            {points.map((point) =>
              point[field] == null ? null : (
                <circle
                  key={point.time}
                  cx={x(point.time)}
                  cy={y(point[field]!)}
                  r="3"
                  fill="currentColor"
                >
                  <title>
                    {new Date(point.time).toLocaleTimeString()}:{" "}
                    {formatMetric(point[field], 2)} {unit}
                  </title>
                </circle>
              ),
            )}
            <text x="50" y="210">
              {new Date(start).toLocaleTimeString()}
            </text>
            <text x={width - 20} y="210" textAnchor="end">
              {new Date(points.at(-1)!.time).toLocaleTimeString()}
            </text>
          </svg>
        )}
      </div>
    </section>
  );
}
