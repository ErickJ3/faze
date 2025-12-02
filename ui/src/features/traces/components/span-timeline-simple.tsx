import { useState } from "react";
import type { Span } from "@/types";
import { formatDurationCompact } from "@/lib/formatters";
import { SpanDetailSheet } from "./span-detail-sheet";

interface SpanTimelineSimpleProps {
  spans: Span[];
}

export function SpanTimelineSimple({ spans }: SpanTimelineSimpleProps) {
  const [selectedSpan, setSelectedSpan] = useState<Span | null>(null);

  if (spans.length === 0) {
    return (
      <div className="flex items-center justify-center h-32 border border-border">
        <p className="text-sm text-foreground/50">No spans to display</p>
      </div>
    );
  }

  const minTime = Math.min(...spans.map((s) => s.start_time_unix_nano));
  const maxTime = Math.max(...spans.map((s) => s.end_time_unix_nano));
  const totalDuration = (maxTime - minTime) / 1_000_000;

  const sortedSpans = [...spans].sort(
    (a, b) => a.start_time_unix_nano - b.start_time_unix_nano,
  );

  return (
    <>
      <div className="border border-border">
        <div className="border-b border-border px-4 py-2 bg-card/20">
          <span className="text-xs font-mono text-foreground/70">
            Total Duration: {formatDurationCompact(totalDuration)}
          </span>
        </div>

        <div className="divide-y divide-border">
          {sortedSpans.map((span) => {
            const duration =
              (span.end_time_unix_nano - span.start_time_unix_nano) / 1_000_000;
            const startOffset =
              ((span.start_time_unix_nano - minTime) /
                1_000_000 /
                totalDuration) *
              100;
            const width = (duration / totalDuration) * 100;
            const hasError = span.status.code === "Error";

            return (
              <button
                key={span.span_id}
                onClick={() => setSelectedSpan(span)}
                className="w-full px-4 py-3 hover:bg-card/50 transition-colors text-left"
              >
                <div className="flex items-center justify-between mb-2">
                  <span className="text-sm font-mono truncate">
                    {span.name}
                  </span>
                  <div className="flex items-center gap-2">
                    <span className="text-xs font-mono text-foreground/70">
                      {formatDurationCompact(duration)}
                    </span>
                    {hasError && (
                      <span className="text-xs text-red-500">ERROR</span>
                    )}
                  </div>
                </div>

                <div className="relative h-2 bg-foreground/5">
                  <div
                    className={`absolute h-full ${hasError ? "bg-red-500" : "bg-primary"}`}
                    style={{
                      left: `${startOffset}%`,
                      width: `${Math.max(width, 0.5)}%`,
                    }}
                  />
                </div>
              </button>
            );
          })}
        </div>
      </div>

      <SpanDetailSheet
        span={selectedSpan}
        open={!!selectedSpan}
        onClose={() => setSelectedSpan(null)}
      />
    </>
  );
}
