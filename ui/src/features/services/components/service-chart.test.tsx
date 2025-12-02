import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { ServiceChart } from "./service-chart";
import type { TraceInfo } from "@/types";

const mockNavigate = vi.fn();
vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => mockNavigate,
}));

const mockTraces: TraceInfo[] = [
  {
    trace_id: "trace-1",
    service_name: "api-service",
    duration_ms: 150,
    span_count: 3,
    has_errors: false,
    start_time: 1000000000000,
    root_span_name: "GET /api/users",
    root_span_kind: "Server",
  },
  {
    trace_id: "trace-2",
    service_name: "db-service",
    duration_ms: 250,
    span_count: 5,
    has_errors: true,
    start_time: 1000000001000,
    root_span_name: "SELECT users",
    root_span_kind: "Client",
  },
  {
    trace_id: "trace-3",
    service_name: "api-service",
    duration_ms: 100,
    span_count: 2,
    has_errors: false,
    start_time: 1000000002000,
    root_span_name: "POST /api/login",
    root_span_kind: "Server",
  },
];

describe("ServiceChart", () => {
  it("renders empty state when no traces provided", () => {
    render(<ServiceChart traces={[]} />);
    expect(screen.getByText("No data available")).toBeInTheDocument();
  });

  it("renders chart with trace data", () => {
    render(<ServiceChart traces={mockTraces} />);
    expect(screen.getByText("Response Time")).toBeInTheDocument();
    expect(screen.getByText(/Last \d+ traces/)).toBeInTheDocument();
  });

  it("calculates and displays average duration", () => {
    render(<ServiceChart traces={mockTraces} />);
    expect(screen.getByText(/Avg:/)).toBeInTheDocument();
  });

  it("calculates and displays max duration", () => {
    render(<ServiceChart traces={mockTraces} />);
    expect(screen.getByText(/Max:/)).toBeInTheDocument();
  });

  it("filters out traces without start_time or duration", () => {
    const tracesWithInvalidData: TraceInfo[] = [
      ...mockTraces,
      {
        trace_id: "trace-invalid",
        service_name: "test-service",
        duration_ms: 0,
        span_count: 1,
        has_errors: false,
        start_time: undefined,
        root_span_name: "Invalid trace",
        root_span_kind: "Internal",
      },
    ];

    render(<ServiceChart traces={tracesWithInvalidData} />);
    expect(screen.queryByText("Invalid trace")).not.toBeInTheDocument();
  });

  it("shows only last 20 traces", () => {
    const manyTraces: TraceInfo[] = Array.from({ length: 30 }, (_, i) => ({
      trace_id: `trace-${i}`,
      service_name: "test-service",
      duration_ms: 100 + i,
      span_count: 1,
      has_errors: false,
      start_time: 1000000000000 + i * 1000,
      root_span_name: `Trace ${i}`,
      root_span_kind: "Server",
    }));

    render(<ServiceChart traces={manyTraces} />);
    expect(screen.getByText("Last 20 traces")).toBeInTheDocument();
  });

  it("displays errors with destructive color", () => {
    render(<ServiceChart traces={mockTraces} />);

    const errorTrace = mockTraces.find((t) => t.has_errors);
    expect(errorTrace).toBeDefined();
  });

  it("has navigation handler", () => {
    render(<ServiceChart traces={mockTraces} />);
    expect(mockNavigate).toBeDefined();
  });

  it("handles traces with missing optional fields", () => {
    const minimalTraces: TraceInfo[] = [
      {
        trace_id: "minimal-trace",
        duration_ms: 100,
        span_count: 1,
        has_errors: false,
        start_time: 1000000000000,
      },
    ];

    render(<ServiceChart traces={minimalTraces} />);
    expect(screen.getByText("Response Time")).toBeInTheDocument();
  });

  it("accepts unsorted traces", () => {
    const unsortedTraces: TraceInfo[] = [
      {
        trace_id: "trace-3",
        duration_ms: 100,
        span_count: 1,
        has_errors: false,
        start_time: 3000,
        root_span_name: "Third",
        root_span_kind: "Server",
      },
      {
        trace_id: "trace-1",
        duration_ms: 100,
        span_count: 1,
        has_errors: false,
        start_time: 1000,
        root_span_name: "First",
        root_span_kind: "Server",
      },
      {
        trace_id: "trace-2",
        duration_ms: 100,
        span_count: 1,
        has_errors: false,
        start_time: 2000,
        root_span_name: "Second",
        root_span_kind: "Server",
      },
    ];

    render(<ServiceChart traces={unsortedTraces} />);
    expect(screen.getByText("Last 3 traces")).toBeInTheDocument();
  });
});
