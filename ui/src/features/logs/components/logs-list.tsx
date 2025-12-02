import type { Log } from "@/types";
import { LogEntry } from "./log-entry";

interface LogsListProps {
  logs: Log[];
}

export function LogsList({ logs }: LogsListProps) {
  if (logs.length === 0) {
    return (
      <div className="flex items-center justify-center h-64 border border-border">
        <div className="text-center">
          <p className="text-foreground/50 text-sm">No logs found</p>
          <p className="text-foreground/30 text-xs mt-1">
            Adjust filters or start sending logs
          </p>
        </div>
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
