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
  Server: "bg-blue-500/10 text-blue-500",
  Client: "bg-purple-500/10 text-purple-500",
  Producer: "bg-green-500/10 text-green-500",
  Consumer: "bg-yellow-500/10 text-yellow-500",
  Internal: "bg-gray-500/10 text-gray-500",
  Unspecified: "bg-gray-500/10 text-gray-500",
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
          <p className="text-foreground/50 text-sm">No traces found</p>
          <p className="text-foreground/30 text-xs mt-1">
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
          {traces.map((trace) => (
            <TableRow
              key={trace.trace_id}
              className="cursor-pointer hover:bg-card/50"
              onClick={() =>
                navigate({
                  to: "/traces/$traceId",
                  params: { traceId: trace.trace_id },
                })
              }
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
                  <span className="font-mono text-xs text-foreground/50">
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
              <TableCell className="text-right font-mono text-sm text-foreground/70">
                {trace.span_count}
              </TableCell>
              <TableCell className="text-center">
                {trace.has_errors && (
                  <span className="text-xs px-2 py-0.5 bg-red-500/10 text-red-500">
                    ERROR
                  </span>
                )}
                {!trace.has_errors && (
                  <span className="text-xs px-2 py-0.5 bg-green-500/10 text-green-500">
                    OK
                  </span>
                )}
              </TableCell>
              <TableCell className="font-mono text-xs text-foreground/50">
                {trace.start_time ? formatRelativeTime(trace.start_time) : "-"}
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  );
}
