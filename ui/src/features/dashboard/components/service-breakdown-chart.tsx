import { Bar, BarChart, LabelList, XAxis, YAxis } from "recharts";
import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
  type ChartConfig,
} from "@/components/ui/chart";
import type { ServiceStat } from "@/types";

const chartConfig = {
  ok: { label: "OK", color: "var(--chart-1)" },
  errors: { label: "Errors", color: "var(--destructive)" },
} satisfies ChartConfig;

const MAX_SERVICES = 8;
const ROW_HEIGHT = 36;

interface ServiceBreakdownChartProps {
  services: ServiceStat[];
}

export function ServiceBreakdownChart({
  services,
}: ServiceBreakdownChartProps) {
  const top = services.slice(0, MAX_SERVICES);
  const rest = services.slice(MAX_SERVICES);

  const data = top.map((service) => ({
    name: service.name,
    ok: service.trace_count - service.error_count,
    errors: service.error_count,
    total: service.trace_count,
  }));

  if (rest.length > 0) {
    const traceCount = rest.reduce((acc, s) => acc + s.trace_count, 0);
    const errorCount = rest.reduce((acc, s) => acc + s.error_count, 0);
    data.push({
      name: `other (${rest.length})`,
      ok: traceCount - errorCount,
      errors: errorCount,
      total: traceCount,
    });
  }

  return (
    <div className="border border-border">
      <div className="px-4 py-3 border-b border-border">
        <h3 className="text-sm font-mono">Traces by Service</h3>
      </div>

      {data.length === 0 ? (
        <div className="flex items-center justify-center h-48">
          <p className="text-sm text-foreground/50">No services yet</p>
        </div>
      ) : (
        <ChartContainer
          config={chartConfig}
          className="w-full aspect-auto p-4"
          style={{ height: data.length * ROW_HEIGHT + 24 }}
        >
          <BarChart
            data={data}
            layout="vertical"
            margin={{ top: 4, right: 40, left: 8, bottom: 4 }}
          >
            <XAxis type="number" hide />
            <YAxis
              type="category"
              dataKey="name"
              tickLine={false}
              axisLine={false}
              width={130}
              tickFormatter={(name: string) =>
                name.length > 18 ? `${name.slice(0, 17)}…` : name
              }
            />
            <ChartTooltip content={<ChartTooltipContent />} />
            <Bar dataKey="ok" stackId="a" fill="var(--color-ok)" radius={0} />
            <Bar
              dataKey="errors"
              stackId="a"
              fill="var(--color-errors)"
              radius={0}
            >
              <LabelList
                dataKey="total"
                position="right"
                className="fill-foreground font-mono"
                fontSize={11}
              />
            </Bar>
          </BarChart>
        </ChartContainer>
      )}
    </div>
  );
}
