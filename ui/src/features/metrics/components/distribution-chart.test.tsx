import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { DistributionChart } from "./distribution-chart";
import type { Distribution } from "@/types";

describe("DistributionChart", () => {
  it("renders a bar chart for histogram distributions", () => {
    const distribution: Distribution = {
      kind: "HISTOGRAM",
      count: 4,
      sum: 15,
      bucket_counts: [1, 2, 1],
      explicit_bounds: [2.5, 5],
    };

    const { container } = render(
      <DistributionChart distribution={distribution} />,
    );
    expect(container.querySelector(".recharts-responsive-container")).not.toBe(
      null,
    );
  });

  it("renders a bar chart for exponential histogram distributions", () => {
    const distribution: Distribution = {
      kind: "EXPONENTIAL_HISTOGRAM",
      count: 5,
      sum: 20,
      scale: 2,
      zero_count: 1,
      zero_threshold: 0,
      positive_offset: -1,
      positive_bucket_counts: [2, 2],
      negative_offset: 0,
      negative_bucket_counts: [],
    };

    const { container } = render(
      <DistributionChart distribution={distribution} />,
    );
    expect(container.querySelector(".recharts-responsive-container")).not.toBe(
      null,
    );
  });

  it("renders a quantile table for summary distributions", () => {
    const distribution: Distribution = {
      kind: "SUMMARY",
      count: 10,
      sum: 30,
      quantile_values: [
        { quantile: 0.5, value: 2 },
        { quantile: 0.99, value: 8 },
      ],
    };

    render(<DistributionChart distribution={distribution} />);
    expect(screen.getByText("p50")).toBeDefined();
    expect(screen.getByText("p99")).toBeDefined();
  });

  it("renders nothing for empty histograms", () => {
    const distribution: Distribution = {
      kind: "HISTOGRAM",
      count: 0,
      bucket_counts: [],
      explicit_bounds: [],
    };

    const { container } = render(
      <DistributionChart distribution={distribution} />,
    );
    expect(container.innerHTML).toBe("");
  });
});
