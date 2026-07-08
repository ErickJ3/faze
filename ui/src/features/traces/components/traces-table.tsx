import { useNavigate } from "@tanstack/react-router";
import type { TraceInfo, SpanKind } from "@/types";
import { formatRelativeTime } from "@/lib/formatters";
import { DurationBadge } from "@/components/shared/duration-badge";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";

const SPAN_KIND_COLORS: Record<SpanKind, string> = {
  SERVER: "bg-chart-3/10 text-chart-3",
  CLIENT: "bg-chart-4/10 text-chart-4",
  PRODUCER: "bg-chart-1/10 text-chart-1",
  CONSUMER: "bg-chart-2/10 text-chart-2",
  INTERNAL: "bg-muted text-muted-foreground",
  UNSPECIFIED: "bg-muted text-muted-foreground",
};

interface TracesTableProps {
  traces: TraceInfo[];
}

export function TracesTable({ traces }: TracesTableProps) {
  const navigate = useNavigate();

  if (traces.length === 0) {
    return (
      <div className="flex items-center justify-center h-64 border border-border">
        <div className="text-center">
          <p className="text-muted-foreground text-sm">No traces found</p>
          <p className="text-muted-foreground text-xs mt-1">
            Adjust filters or start sending traces
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="border border-border">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Operation</TableHead>
            <TableHead>Service</TableHead>
            <TableHead className="w-[100px] text-right">Duration</TableHead>
            <TableHead className="w-[80px] text-right">Spans</TableHead>
            <TableHead className="w-[80px] text-center">Status</TableHead>
            <TableHead className="w-[120px]">Time</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {traces.map((trace) => {
            const openTrace = () =>
              navigate({
                to: "/traces/$traceId",
                params: { traceId: trace.trace_id },
              });
            return (
            <TableRow
              key={trace.trace_id}
              role="link"
              tabIndex={0}
              aria-label={`View trace ${trace.root_span_name || "unknown"}`}
              className="cursor-pointer hover:bg-card/50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-inset"
              onClick={openTrace}
              onKeyDown={(e) => {
                if (e.key === "Enter" || e.key === " ") {
                  e.preventDefault();
                  openTrace();
                }
              }}
            >
              <TableCell>
                <div className="flex flex-col gap-1">
                  <div className="flex items-center gap-2">
                    {trace.root_span_kind && (
                      <span
                        className={`text-xs px-1.5 py-0.5 ${SPAN_KIND_COLORS[trace.root_span_kind]}`}
                      >
                        {trace.root_span_kind.toLowerCase()}
                      </span>
                    )}
                    <span className="font-mono text-sm">
                      {trace.root_span_name || "unknown"}
                    </span>
                  </div>
                  <span className="font-mono text-xs text-muted-foreground">
                    {trace.trace_id.substring(0, 16)}...
                  </span>
                </div>
              </TableCell>
              <TableCell className="font-mono text-sm">
                {trace.service_name || "unknown"}
              </TableCell>
              <TableCell className="text-right">
                <DurationBadge durationMs={trace.duration_ms} />
              </TableCell>
              <TableCell className="text-right font-mono text-sm text-muted-foreground">
                {trace.span_count}
              </TableCell>
              <TableCell className="text-center">
                {trace.has_errors && (
                  <span className="text-xs px-2 py-0.5 bg-status-error/10 text-status-error">
                    ERROR
                  </span>
                )}
                {!trace.has_errors && (
                  <span className="text-xs px-2 py-0.5 bg-status-ok/10 text-status-ok">
                    OK
                  </span>
                )}
              </TableCell>
              <TableCell className="font-mono text-xs text-muted-foreground">
                {trace.start_time ? formatRelativeTime(trace.start_time) : "-"}
              </TableCell>
            </TableRow>
            );
          })}
        </TableBody>
      </Table>
    </div>
  );
}
