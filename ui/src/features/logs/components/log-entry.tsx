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
        <span className="text-xs text-foreground/40 min-w-[100px]">
          {formatTimestamp(log.time_unix_nano)}
        </span>
        <SeverityBadge level={log.severity_level} />
        {log.service_name && (
          <span className="text-xs text-foreground/50">{log.service_name}</span>
        )}
      </div>

      <div className="text-sm text-foreground pl-[100px]">{log.body}</div>

      {log.trace_id && (
        <div className="text-xs text-foreground/30 pl-[100px] mt-1">
          trace: {log.trace_id.substring(0, 16)}...
        </div>
      )}
    </div>
  );
}
