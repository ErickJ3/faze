import { useState } from "react";
import type { Span } from "@/types";
import { formatDurationCompact } from "@/lib/formatters";
import { SpanDetailDialog } from "./span-detail-dialog";

interface SpanTimelineProps {
  spans: Span[];
}

interface SpanNode extends Span {
  children: SpanNode[];
  depth: number;
}

interface TimelineSpan {
  span: Span;
  depth: number;
  startOffset: number;
  width: number;
  children: TimelineSpan[];
}

function buildSpanTree(spans: Span[]): SpanNode[] {
  const spanMap = new Map<string, SpanNode>();
  const roots: SpanNode[] = [];

  spans.forEach((span) => {
    spanMap.set(span.span_id, { ...span, children: [], depth: 0 });
  });

  spans.forEach((span) => {
    const node = spanMap.get(span.span_id)!;
    if (span.parent_span_id) {
      const parent = spanMap.get(span.parent_span_id);
      if (parent) {
        parent.children.push(node);
        node.depth = parent.depth + 1;
      } else {
        roots.push(node);
      }
    } else {
      roots.push(node);
    }
  });

  return roots;
}

function buildTimeline(spans: Span[]): {
  items: TimelineSpan[];
  totalDuration: number;
  minTime: number;
} {
  if (spans.length === 0) {
    return { items: [], totalDuration: 0, minTime: 0 };
  }

  const tree = buildSpanTree(spans);
  const minTime = Math.min(...spans.map((s) => s.start_time_unix_nano));
  const maxTime = Math.max(...spans.map((s) => s.end_time_unix_nano));
  const totalDuration = (maxTime - minTime) / 1_000_000;

  function processNode(node: SpanNode): TimelineSpan {
    const startOffset =
      ((node.start_time_unix_nano - minTime) / 1_000_000 / totalDuration) * 100;
    const duration =
      (node.end_time_unix_nano - node.start_time_unix_nano) / 1_000_000;
    const width = (duration / totalDuration) * 100;

    return {
      span: node,
      depth: node.depth,
      startOffset,
      width: Math.max(width, 0.5),
      children: node.children.map(processNode),
    };
  }

  const items = tree.map(processNode);

  return { items, totalDuration, minTime };
}

function flattenTimeline(items: TimelineSpan[]): TimelineSpan[] {
  const result: TimelineSpan[] = [];

  function traverse(item: TimelineSpan) {
    result.push(item);
    item.children.forEach(traverse);
  }

  items.forEach(traverse);
  return result;
}

function TimelineBar({
  item,
  onClick,
}: {
  item: TimelineSpan;
  onClick: (span: Span) => void;
}) {
  const hasError = item.span.status.code === "Error";
  const duration =
    (item.span.end_time_unix_nano - item.span.start_time_unix_nano) / 1_000_000;

  const barColor = hasError
    ? "bg-red-500/80 hover:bg-red-500"
    : "bg-green-500/60 hover:bg-green-500";

  return (
    <button
      onClick={() => onClick(item.span)}
      className={`absolute h-6 ${barColor} transition-colors cursor-pointer border border-foreground/20 group`}
      style={{
        left: `${item.startOffset}%`,
        width: `${item.width}%`,
        top: `${item.depth * 32}px`,
      }}
      title={`${item.span.name} - ${formatDurationCompact(duration)}`}
    >
      <div className="flex items-center h-full px-1 overflow-hidden">
        <span className="text-xs font-mono text-background truncate whitespace-nowrap">
          {item.span.name}
        </span>
      </div>
    </button>
  );
}

export function SpanTimeline({ spans }: SpanTimelineProps) {
  const [selectedSpan, setSelectedSpan] = useState<Span | null>(null);
  const { items, totalDuration } = buildTimeline(spans);
  const flatItems = flattenTimeline(items);

  if (spans.length === 0) {
    return (
      <div className="flex items-center justify-center h-32 border border-border">
        <p className="text-sm text-foreground/50">No spans to display</p>
      </div>
    );
  }

  const maxDepth = Math.max(...flatItems.map((item) => item.depth));
  const height = (maxDepth + 1) * 32 + 40;

  const timeMarkers = [0, 25, 50, 75, 100];

  return (
    <>
      <div className="border border-border bg-card/20">
        <div className="border-b border-border px-4 py-2 flex items-center justify-between">
          <span className="text-xs font-mono text-foreground/70">
            Total Duration: {formatDurationCompact(totalDuration)}
          </span>
          <span className="text-xs text-foreground/50">
            {spans.length} {spans.length === 1 ? "span" : "spans"}
          </span>
        </div>

        <div className="relative" style={{ height: `${height}px` }}>
          <div className="absolute inset-0 px-4 pt-6">
            {timeMarkers.map((marker) => (
              <div
                key={marker}
                className="absolute top-0 bottom-0 border-l border-foreground/10"
                style={{ left: `calc(${marker}% + 1rem)` }}
              >
                <span className="absolute -top-5 -translate-x-1/2 text-xs text-foreground/30">
                  {marker}%
                </span>
              </div>
            ))}

            {flatItems.map((item, index) => (
              <TimelineBar key={index} item={item} onClick={setSelectedSpan} />
            ))}
          </div>
        </div>

        <div className="border-t border-border px-4 py-2 flex gap-4 text-xs">
          <div className="flex items-center gap-2">
            <div className="w-3 h-3 bg-green-500/60" />
            <span className="text-foreground/50">Normal</span>
          </div>
          <div className="flex items-center gap-2">
            <div className="w-3 h-3 bg-red-500/80" />
            <span className="text-foreground/50">Error</span>
          </div>
        </div>
      </div>

      <SpanDetailDialog
        span={selectedSpan}
        open={!!selectedSpan}
        onClose={() => setSelectedSpan(null)}
      />
    </>
  );
}
