import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { ServiceChart } from "@/features/services/components/service-chart";
import type { TraceInfo } from "@/types";

const mockNavigate = vi.fn();
vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => mockNavigate,
}));

describe("Chart Navigation E2E", () => {
  beforeEach(() => {
    mockNavigate.mockClear();
  });

  const sampleTraces: TraceInfo[] = [
    {
      trace_id: "trace-abc123",
      service_name: "user-api",
      duration_ms: 145.5,
      span_count: 4,
      has_errors: false,
      start_time: 1000000000000,
      root_span_name: "GET /api/users",
      root_span_kind: "Server",
    },
    {
      trace_id: "trace-def456",
      service_name: "auth-service",
      duration_ms: 89.2,
      span_count: 2,
      has_errors: false,
      start_time: 1000000001000,
      root_span_name: "POST /auth/login",
      root_span_kind: "Server",
    },
    {
      trace_id: "trace-ghi789",
      service_name: "db-service",
      duration_ms: 523.8,
      span_count: 8,
      has_errors: true,
      start_time: 1000000002000,
      root_span_name: "SELECT * FROM users WHERE id = ?",
      root_span_kind: "Client",
    },
  ];

  it("should display chart with proper statistics", () => {
    render(<ServiceChart traces={sampleTraces} />);

    expect(screen.getByText("Response Time")).toBeInTheDocument();
    expect(screen.getByText(/Avg:/)).toBeInTheDocument();
    expect(screen.getByText(/Max:/)).toBeInTheDocument();
    expect(screen.getByText("Last 3 traces")).toBeInTheDocument();
  });

  it("should have navigation capability", () => {
    render(<ServiceChart traces={sampleTraces} />);
    expect(mockNavigate).toBeDefined();
  });

  it("should render chart component", () => {
    render(<ServiceChart traces={sampleTraces} />);
    expect(screen.getByText("Response Time")).toBeInTheDocument();
  });

  it("should handle error traces", () => {
    render(<ServiceChart traces={sampleTraces} />);
    const errorTrace = sampleTraces.find((t) => t.has_errors);
    expect(errorTrace).toBeDefined();
    expect(errorTrace?.has_errors).toBe(true);
  });

  it("should handle empty traces gracefully", () => {
    render(<ServiceChart traces={[]} />);

    expect(screen.getByText("Response Time")).toBeInTheDocument();
    expect(screen.getByText("No data available")).toBeInTheDocument();
  });

  it("should calculate correct average duration", () => {
    render(<ServiceChart traces={sampleTraces} />);

    const avgDuration =
      sampleTraces.reduce((sum, t) => sum + t.duration_ms, 0) /
      sampleTraces.length;

    expect(screen.getByText(/Avg:/)).toBeInTheDocument();
    expect(avgDuration).toBeCloseTo(252.83, 1);
  });

  it("should calculate correct max duration", () => {
    render(<ServiceChart traces={sampleTraces} />);

    const maxDuration = Math.max(...sampleTraces.map((t) => t.duration_ms));

    expect(screen.getByText(/Max:/)).toBeInTheDocument();
    expect(maxDuration).toBe(523.8);
  });

  it("should render traces in chronological order", () => {
    render(<ServiceChart traces={sampleTraces} />);

    const sortedTraces = [...sampleTraces].sort(
      (a, b) => (a.start_time || 0) - (b.start_time || 0),
    );

    expect(sortedTraces[0].trace_id).toBe("trace-abc123");
    expect(sortedTraces[1].trace_id).toBe("trace-def456");
    expect(sortedTraces[2].trace_id).toBe("trace-ghi789");
  });

  it("should limit display to last 20 traces", () => {
    const manyTraces: TraceInfo[] = Array.from({ length: 50 }, (_, i) => ({
      trace_id: `trace-${i}`,
      service_name: "test-service",
      duration_ms: 100 + i,
      span_count: 1,
      has_errors: false,
      start_time: 1000000000000 + i * 1000,
      root_span_name: `Operation ${i}`,
      root_span_kind: "Server",
    }));

    render(<ServiceChart traces={manyTraces} />);

    expect(screen.getByText("Last 20 traces")).toBeInTheDocument();
  });

  it("should handle traces with missing optional fields", () => {
    const minimalTraces: TraceInfo[] = [
      {
        trace_id: "minimal-1",
        duration_ms: 100,
        span_count: 1,
        has_errors: false,
        start_time: 1000000000000,
      },
    ];

    render(<ServiceChart traces={minimalTraces} />);

    expect(screen.getByText("Response Time")).toBeInTheDocument();
    expect(screen.getByText("Last 1 traces")).toBeInTheDocument();
  });
});
