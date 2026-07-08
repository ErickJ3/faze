import type { Log } from "@/types";
import { LogEntry } from "@/features/logs/components/log-entry";
import { LoadingState } from "@/components/shared/loading-state";
import { ErrorState } from "@/components/shared/error-state";

interface TraceLogsProps {
  logs: Log[];
  isLoading: boolean;
  isError?: boolean;
  onRetry?: () => void;
}

export function TraceLogs({ logs, isLoading, isError, onRetry }: TraceLogsProps) {
  if (isLoading) {
    return <LoadingState />;
  }

  if (isError) {
    return (
      <ErrorState title="Couldn't load correlated logs" onRetry={onRetry} />
    );
  }

  if (logs.length === 0) {
    return (
      <div className="flex items-center justify-center h-32 border border-border">
        <p className="text-sm text-muted-foreground">
          No logs correlated with this trace
        </p>
      </div>
    );
  }

  return (
    <div className="border border-border">
      {logs.map((log, index) => (
        <LogEntry key={`${log.time_unix_nano}-${index}`} log={log} />
      ))}
    </div>
  );
}
