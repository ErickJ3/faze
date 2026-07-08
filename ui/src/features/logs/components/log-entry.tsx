import type { Log } from "@/types";
import { formatTimestamp } from "@/lib/formatters";
import { SeverityBadge } from "@/components/shared/severity-badge";

interface LogEntryProps {
  log: Log;
}

export function LogEntry({ log }: LogEntryProps) {
  return (
    <div className="py-3 px-4 border-b border-border hover:bg-card/30 transition-colors font-mono">
      <div className="flex items-start gap-3 mb-1">
        <span className="text-xs text-muted-foreground min-w-[100px]">
          {formatTimestamp(log.time_unix_nano)}
        </span>
        <SeverityBadge level={log.severity_level} />
        {log.event_name && (
          <span className="text-xs px-1 border border-border text-foreground/60">
            {log.event_name}
          </span>
        )}
        {log.service_name && (
          <span className="text-xs text-muted-foreground">{log.service_name}</span>
        )}
      </div>

      <div className="text-sm text-foreground pl-[100px]">{log.body}</div>

      {(log.trace_id || log.observed_time_unix_nano) && (
        <div className="text-xs text-muted-foreground pl-[100px] mt-1 flex gap-3">
          {log.trace_id && <span>trace: {log.trace_id.substring(0, 16)}...</span>}
          {log.observed_time_unix_nano && (
            <span>observed: {formatTimestamp(log.observed_time_unix_nano)}</span>
          )}
        </div>
      )}
    </div>
  );
}
