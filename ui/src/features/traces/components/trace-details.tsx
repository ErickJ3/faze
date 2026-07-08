import type { Trace } from "@/types";
import { SpanWaterfall } from "./span-waterfall";
import { TraceLogs } from "./trace-logs";
import { CopyButton } from "@/components/shared/copy-button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useLogs } from "@/hooks/api";
import {
  formatDurationCompact,
  formatDateTime,
} from "@/lib/formatters";

interface TraceDetailsProps {
  trace: Trace;
}

export function TraceDetails({ trace }: TraceDetailsProps) {
  const {
    data: logs,
    isLoading: logsLoading,
    isError: logsError,
    refetch: refetchLogs,
  } = useLogs({
    trace_id: trace.trace_id,
  });
  const startTimes = trace.spans
    .map((s) => s.start_time_unix_nano)
    .filter((t) => t > 0);
  const endTimes = trace.spans
    .map((s) => s.end_time_unix_nano)
    .filter((t) => t > 0);
  const startTime = startTimes.length > 0 ? Math.min(...startTimes) : null;
  const durationMs =
    startTime !== null && endTimes.length > 0
      ? (Math.max(...endTimes) - startTime) / 1_000_000
      : null;

  return (
    <div>
      <div className="mb-6">
        <h1 className="text-lg font-mono mb-2">Trace Details</h1>
        <div className="text-sm text-muted-foreground space-y-2">
          <div className="flex items-center justify-between">
            <div>
              <span className="text-muted-foreground">ID:</span>{" "}
              <span className="font-mono text-xs">{trace.trace_id}</span>
            </div>
            <CopyButton text={trace.trace_id} label="Copy ID" />
          </div>
          {trace.service_name && (
            <div>
              <span className="text-muted-foreground">Service:</span>{" "}
              <span className="font-mono">{trace.service_name}</span>
            </div>
          )}
          <div className="flex items-center gap-6">
            <div>
              <span className="text-muted-foreground">Spans:</span>{" "}
              <span className="font-mono">{trace.spans.length}</span>
            </div>
            {durationMs !== null && (
              <div>
                <span className="text-muted-foreground">Duration:</span>{" "}
                <span className="font-mono">
                  {formatDurationCompact(durationMs)}
                </span>
              </div>
            )}
            {startTime !== null && (
              <div>
                <span className="text-muted-foreground">Started:</span>{" "}
                <span className="font-mono">{formatDateTime(startTime)}</span>
              </div>
            )}
          </div>
        </div>
      </div>

      <Tabs defaultValue="waterfall">
        <TabsList>
          <TabsTrigger value="waterfall">Waterfall</TabsTrigger>
          <TabsTrigger value="logs">Logs ({logs?.length ?? 0})</TabsTrigger>
        </TabsList>

        <TabsContent value="waterfall" className="mt-4">
          {trace.spans.length === 0 ? (
            <div className="flex items-center justify-center h-32 border border-border">
              <p className="text-sm text-muted-foreground">
                No spans recorded for this trace
              </p>
            </div>
          ) : (
            <SpanWaterfall key={trace.trace_id} spans={trace.spans} />
          )}
        </TabsContent>

        <TabsContent value="logs" className="mt-4">
          <TraceLogs
            logs={logs ?? []}
            isLoading={logsLoading}
            isError={logsError}
            onRetry={() => refetchLogs()}
          />
        </TabsContent>
      </Tabs>
    </div>
  );
}
