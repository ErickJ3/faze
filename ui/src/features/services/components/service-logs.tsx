import { Link } from "@tanstack/react-router";
import type { Log } from "@/types";
import { formatRelativeTime } from "@/lib/formatters";
import { getSeverityColor } from "@/lib/constants";

interface ServiceLogsProps {
  logs: Log[];
  serviceName: string;
}

export function ServiceLogs({ logs, serviceName }: ServiceLogsProps) {
  if (logs.length === 0) {
    return (
      <div className="border border-border p-6">
        <h2 className="text-lg font-mono mb-4">Recent Logs</h2>
        <p className="text-sm text-foreground/50">
          No logs found for this service
        </p>
      </div>
    );
  }

  return (
    <div className="border border-border">
      <div className="flex items-center justify-between p-4 border-b border-border">
        <h2 className="text-lg font-mono">Recent Logs</h2>
        <Link
          to="/logs"
          search={{ service: serviceName }}
          className="text-xs text-foreground/50 hover:text-foreground transition-colors"
        >
          View all →
        </Link>
      </div>

      <div className="divide-y divide-border">
        {logs.map((log, index) => (
          <div
            key={`${log.timestamp}-${index}`}
            className="p-4 hover:bg-card/50 transition-colors"
          >
            <div className="flex items-start justify-between gap-4">
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2 mb-1">
                  {log.severity_level && (
                    <span
                      className={`text-xs px-2 py-0.5 ${getSeverityColor(log.severity_level)}`}
                    >
                      {log.severity_level}
                    </span>
                  )}
                  <span className="text-xs text-foreground/50">
                    {formatRelativeTime(log.timestamp)}
                  </span>
                </div>
                <div className="text-sm font-mono truncate">{log.body}</div>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
