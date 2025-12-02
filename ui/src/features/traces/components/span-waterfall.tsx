import { useState, useEffect, useMemo } from "react";
import type { Span } from "@/types";
import { formatDurationCompact } from "@/lib/formatters";
import { SpanDetailSheet } from "./span-detail-sheet";

interface SpanWaterfallProps {
  spans: Span[];
}

interface SpanNode extends Span {
  children: SpanNode[];
  depth: number;
}

function buildSpanTree(spans: Span[]): {
  roots: SpanNode[];
  minTime: number;
  maxTime: number;
} {
  const spanMap = new Map<string, SpanNode>();
  const roots: SpanNode[] = [];

  spans.forEach((span) => {
    spanMap.set(span.span_id, { ...span, children: [], depth: 0 });
  });

  const sorted = [...spans].sort(
    (a, b) => a.start_time_unix_nano - b.start_time_unix_nano,
  );

  sorted.forEach((span) => {
    const node = spanMap.get(span.span_id)!;
    if (span.parent_span_id && span.parent_span_id !== "") {
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

  const startTimes = spans
    .map((s) => s.start_time_unix_nano)
    .filter((t) => t > 0);
  const endTimes = spans.map((s) => s.end_time_unix_nano).filter((t) => t > 0);

  const minTime = Math.min(...startTimes);
  const maxTime = Math.max(...endTimes);

  return { roots, minTime, maxTime };
}

interface SpanRowProps {
  node: SpanNode;
  minTime: number;
  totalDuration: number;
  onSpanClick: (span: Span) => void;
  expanded: Set<string>;
  onToggle: (spanId: string) => void;
}

function SpanRow({
  node,
  minTime,
  totalDuration,
  onSpanClick,
  expanded,
  onToggle,
}: SpanRowProps) {
  const duration =
    (node.end_time_unix_nano - node.start_time_unix_nano) / 1_000_000;
  const startOffset =
    ((node.start_time_unix_nano - minTime) / 1_000_000 / totalDuration) * 100;
  const width = (duration / totalDuration) * 100;
  const hasError = node.status.code === "Error";
  const hasChildren = node.children.length > 0;
  const isExpanded = expanded.has(node.span_id);

  return (
    <>
      <div className="flex items-center border-b border-border hover:bg-card/30 transition-colors">
        <div
          className="flex-1 flex items-center gap-2 py-2 px-3 min-w-0"
          style={{ paddingLeft: `${node.depth * 20 + 12}px` }}
        >
          {hasChildren && (
            <button
              onClick={(e) => {
                e.stopPropagation();
                onToggle(node.span_id);
              }}
              className="text-foreground/50 hover:text-foreground w-4 h-4 flex items-center justify-center"
            >
              {isExpanded ? "▼" : "▶"}
            </button>
          )}
          {!hasChildren && <div className="w-4" />}

          <button
            onClick={() => onSpanClick(node)}
            className="flex-1 text-left min-w-0"
          >
            <div className="flex items-center gap-2">
              <span className="text-sm font-mono truncate">{node.name}</span>
              {hasError && (
                <span className="text-xs px-1 bg-red-500/10 text-red-500">
                  ERROR
                </span>
              )}
            </div>
          </button>

          <span className="text-xs font-mono text-foreground/50 whitespace-nowrap ml-2">
            {formatDurationCompact(duration)}
          </span>
        </div>

        <div className="w-1/2 px-3 py-2 relative h-8">
          <div className="absolute inset-y-0 left-3 right-3 flex items-center">
            <div
              className={`h-4 ${hasError ? "bg-red-500" : "bg-primary"} hover:opacity-80 transition-opacity cursor-pointer`}
              style={{
                marginLeft: `${startOffset}%`,
                width: `${Math.max(width, 0.5)}%`,
              }}
              onClick={() => onSpanClick(node)}
            />
          </div>
        </div>
      </div>

      {hasChildren && isExpanded && (
        <>
          {node.children.map((child) => (
            <SpanRow
              key={child.span_id}
              node={child}
              minTime={minTime}
              totalDuration={totalDuration}
              onSpanClick={onSpanClick}
              expanded={expanded}
              onToggle={onToggle}
            />
          ))}
        </>
      )}
    </>
  );
}

export function SpanWaterfall({ spans }: SpanWaterfallProps) {
  const [selectedSpan, setSelectedSpan] = useState<Span | null>(null);
  const [expanded, setExpanded] = useState<Set<string>>(new Set());

  const spanTree = useMemo(() => {
    if (!spans || spans.length === 0) return null;
    try {
      return buildSpanTree(spans);
    } catch {
      return null;
    }
  }, [spans]);

  useEffect(() => {
    if (!spanTree) return;

    const allSpanIds = new Set<string>();
    const traverse = (node: SpanNode) => {
      if (node.children.length > 0) {
        allSpanIds.add(node.span_id);
        node.children.forEach(traverse);
      }
    };
    spanTree.roots.forEach(traverse);
    setExpanded(allSpanIds);
  }, [spanTree]);

  if (!spans || spans.length === 0) {
    return (
      <div className="flex items-center justify-center h-32 border border-border">
        <p className="text-sm text-foreground/50">No spans to display</p>
      </div>
    );
  }

  if (!spanTree) {
    return (
      <div className="flex items-center justify-center h-32 border border-border">
        <p className="text-sm text-foreground/50">Error displaying waterfall</p>
      </div>
    );
  }

  try {
    const { roots, minTime, maxTime } = spanTree;
    const totalDuration = (maxTime - minTime) / 1_000_000;

    if (!isFinite(totalDuration) || totalDuration <= 0) {
      return (
        <div className="flex items-center justify-center h-32 border border-border">
          <p className="text-sm text-foreground/50">Invalid span timing data</p>
        </div>
      );
    }

    const toggleExpand = (spanId: string) => {
      const newExpanded = new Set(expanded);
      if (newExpanded.has(spanId)) {
        newExpanded.delete(spanId);
      } else {
        newExpanded.add(spanId);
      }
      setExpanded(newExpanded);
    };

    const expandAll = () => {
      const allSpanIds = new Set<string>();
      const traverse = (node: SpanNode) => {
        if (node.children.length > 0) {
          allSpanIds.add(node.span_id);
          node.children.forEach(traverse);
        }
      };
      roots.forEach(traverse);
      setExpanded(allSpanIds);
    };

    const collapseAll = () => {
      setExpanded(new Set());
    };

    return (
      <>
        <div className="border border-border">
          <div className="flex items-center justify-between border-b border-border px-3 py-2 bg-card/20">
            <div className="flex items-center gap-3">
              <span className="text-xs font-mono text-foreground/70">
                Total: {formatDurationCompact(totalDuration)}
              </span>
              <span className="text-xs text-foreground/50">
                {spans.length} {spans.length === 1 ? "span" : "spans"}
              </span>
            </div>

            <div className="flex items-center gap-2">
              <button
                onClick={expandAll}
                className="text-xs text-foreground/50 hover:text-foreground"
              >
                Expand All
              </button>
              <span className="text-foreground/30">|</span>
              <button
                onClick={collapseAll}
                className="text-xs text-foreground/50 hover:text-foreground"
              >
                Collapse All
              </button>
            </div>
          </div>

          <div className="flex border-b border-border bg-card/10">
            <div className="flex-1 px-3 py-2">
              <span className="text-xs font-mono text-foreground/50">SPAN</span>
            </div>
            <div className="w-1/2 px-3 py-2">
              <span className="text-xs font-mono text-foreground/50">
                TIMELINE
              </span>
            </div>
          </div>

          <div>
            {roots.map((node) => (
              <SpanRow
                key={node.span_id}
                node={node}
                minTime={minTime}
                totalDuration={totalDuration}
                onSpanClick={setSelectedSpan}
                expanded={expanded}
                onToggle={toggleExpand}
              />
            ))}
          </div>
        </div>

        <SpanDetailSheet
          span={selectedSpan}
          open={!!selectedSpan}
          onClose={() => setSelectedSpan(null)}
        />
      </>
    );
  } catch (error) {
    console.error("Error rendering waterfall:", error);
    return (
      <div className="flex items-center justify-center h-32 border border-border">
        <p className="text-sm text-foreground/50">Error displaying waterfall</p>
      </div>
    );
  }
}
