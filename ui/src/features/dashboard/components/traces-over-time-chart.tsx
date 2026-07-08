import { Bar, BarChart, CartesianGrid, XAxis, YAxis } from "recharts";
import {
  ChartContainer,
  ChartLegend,
  ChartLegendContent,
  ChartTooltip,
  ChartTooltipContent,
  type ChartConfig,
} from "@/components/ui/chart";
import { formatDateTime, formatTimestamp } from "@/lib/formatters";
import type { ActivityBucket } from "@/types";

const chartConfig = {
  ok: { label: "OK", color: "var(--chart-1)" },
  errors: { label: "Errors", color: "var(--destructive)" },
} satisfies ChartConfig;

const MINUTE_NANOS = 60 * 1_000_000_000;
const DAY_NANOS = 24 * 60 * MINUTE_NANOS;

interface TracesOverTimeChartProps {
  activity: ActivityBucket[];
}

export function TracesOverTimeChart({ activity }: TracesOverTimeChartProps) {
  const data = activity.map((bucket) => ({
    time: bucket.bucket_start_unix_nano,
    ok: bucket.total - bucket.errors,
    errors: bucket.errors,
  }));

  const range =
    data.length > 1 ? data[data.length - 1].time - data[0].time : 0;
  const formatTick = (nano: number) => {
    if (range > DAY_NANOS) return formatDateTime(nano).slice(0, 16);
    // Sub-minute ranges need millisecond precision to tell ticks apart.
    if (range < MINUTE_NANOS) return formatTimestamp(nano);
    return formatTimestamp(nano).slice(0, 8);
  };

  return (
    <div className="border border-border">
      <div className="px-4 py-3 border-b border-border">
        <h3 className="text-sm font-mono">Trace Activity</h3>
      </div>

      {data.length === 0 ? (
        <div className="flex items-center justify-center h-48">
          <p className="text-sm text-foreground/50">No trace activity yet</p>
        </div>
      ) : (
        <ChartContainer config={chartConfig} className="w-full p-4 pt-2">
          <BarChart data={data} margin={{ top: 12, right: 4, left: -16 }}>
            <CartesianGrid vertical={false} />
            <XAxis
              dataKey="time"
              tickLine={false}
              axisLine={false}
              minTickGap={48}
              tickFormatter={formatTick}
            />
            <YAxis tickLine={false} axisLine={false} allowDecimals={false} />
            <ChartTooltip
              content={
                <ChartTooltipContent
                  labelFormatter={(_, payload) =>
                    payload?.[0]
                      ? formatDateTime(payload[0].payload.time)
                      : ""
                  }
                />
              }
            />
            <ChartLegend content={<ChartLegendContent />} />
            <Bar dataKey="ok" stackId="a" fill="var(--color-ok)" radius={0} />
            <Bar
              dataKey="errors"
              stackId="a"
              fill="var(--color-errors)"
              radius={0}
            />
          </BarChart>
        </ChartContainer>
      )}
    </div>
  );
}
