import { memo } from "react";
import { Bar, BarChart, XAxis, YAxis } from "recharts";
import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
  type ChartConfig,
} from "@/components/ui/chart";
import { formatNumber } from "@/lib/formatters";
import type { Distribution } from "@/types";

const chartConfig = {
  count: { label: "Count", color: "var(--chart-1)" },
} satisfies ChartConfig;

const CHART_MARGIN = { top: 4, right: 4, left: -24 };
const AXIS_TICK = { fontSize: 10 };

interface DistributionChartProps {
  distribution: Distribution;
}

interface BucketDatum {
  bucket: string;
  count: number;
}

function histogramBuckets(
  bucketCounts: number[],
  explicitBounds: number[],
): BucketDatum[] {
  return bucketCounts.map((count, i) => {
    let bucket: string;
    if (explicitBounds.length === 0) {
      bucket = "all";
    } else if (i === 0) {
      bucket = `<=${formatNumber(explicitBounds[0])}`;
    } else if (i >= explicitBounds.length) {
      bucket = `>${formatNumber(explicitBounds[explicitBounds.length - 1])}`;
    } else {
      bucket = `${formatNumber(explicitBounds[i - 1])}-${formatNumber(explicitBounds[i])}`;
    }
    return { bucket, count };
  });
}

function exponentialBuckets(
  scale: number,
  zeroCount: number,
  offset: number,
  bucketCounts: number[],
): BucketDatum[] {
  // Base-2 exponential scale: bucket i covers (base^(offset+i), base^(offset+i+1)].
  const base = 2 ** 2 ** -scale;
  const buckets: BucketDatum[] = bucketCounts.map((count, i) => ({
    bucket: `<=${formatNumber(base ** (offset + i + 1))}`,
    count,
  }));
  if (zeroCount > 0) {
    buckets.unshift({ bucket: "0", count: zeroCount });
  }
  return buckets;
}

function BucketBarChart({ data }: { data: BucketDatum[] }) {
  const total = data.reduce((sum, d) => sum + d.count, 0);
  return (
    <ChartContainer
      config={chartConfig}
      className="w-full h-32"
      role="img"
      aria-label={`Value distribution across ${data.length} buckets, ${total} total observations.`}
    >
      <BarChart data={data} margin={CHART_MARGIN}>
        <XAxis
          dataKey="bucket"
          tickLine={false}
          axisLine={false}
          interval="preserveStartEnd"
          tick={AXIS_TICK}
        />
        <YAxis
          tickLine={false}
          axisLine={false}
          allowDecimals={false}
          tick={AXIS_TICK}
        />
        <ChartTooltip content={<ChartTooltipContent />} />
        <Bar dataKey="count" fill="var(--color-count)" radius={0} />
      </BarChart>
    </ChartContainer>
  );
}

export const DistributionChart = memo(function DistributionChart({
  distribution,
}: DistributionChartProps) {
  switch (distribution.kind) {
    case "HISTOGRAM": {
      const data = histogramBuckets(
        distribution.bucket_counts,
        distribution.explicit_bounds,
      );
      if (data.length === 0) {
        return null;
      }
      return <BucketBarChart data={data} />;
    }
    case "EXPONENTIAL_HISTOGRAM": {
      const data = exponentialBuckets(
        distribution.scale,
        distribution.zero_count,
        distribution.positive_offset,
        distribution.positive_bucket_counts,
      );
      if (data.length === 0) {
        return null;
      }
      return <BucketBarChart data={data} />;
    }
    case "SUMMARY": {
      if (distribution.quantile_values.length === 0) {
        return null;
      }
      return (
        <div className="border border-border">
          {distribution.quantile_values.map((qv, i) => (
            <div
              key={qv.quantile}
              className={`flex items-center justify-between px-2 py-1 text-xs font-mono ${
                i > 0 ? "border-t border-border" : ""
              }`}
            >
              <span className="text-muted-foreground">
                p{Math.round(qv.quantile * 100)}
              </span>
              <span>{formatNumber(qv.value)}</span>
            </div>
          ))}
        </div>
      );
    }
  }
});
