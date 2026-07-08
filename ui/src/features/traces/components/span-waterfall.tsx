import { useMemo, useState } from "react";
import { ChevronDown, ChevronRight } from "lucide-react";
import type { Span } from "@/types";
import { formatDurationCompact } from "@/lib/formatters";
import {
  buildSpanTree,
  collectParentIds,
  type SpanNode,
} from "../lib/span-tree";
import { SpanDetailSheet } from "./span-detail-sheet";

interface SpanWaterfallProps {
  spans: Span[];
}

/** Shared column template so every row lines up on one time gutter. */
const GRID_COLS = "grid grid-cols-[minmax(280px,40%)_1fr]";

/** Fractions of total duration where axis ticks and gridlines sit. */
const TICK_FRACTIONS = [0, 0.25, 0.5, 0.75, 1];

/** Cap on rendered event markers per span, guarding pathological spans. */
const MAX_EVENT_MARKERS = 50;

function EmptyWaterfall({ message }: { message: string }) {
  return (
    <div className="flex items-center justify-center h-32 border border-border">
      <p className="text-sm text-foreground/50">{message}</p>
    </div>
  );
}

interface SpanRowProps {
  node: SpanNode;
  minTime: number;
  totalDuration: number;
  showService: boolean;
  collapsed: Set<string>;
  onToggle: (spanId: string) => void;
  onSpanClick: (span: Span) => void;
}

function SpanRow({
  node,
  minTime,
  totalDuration,
  showService,
  collapsed,
  onToggle,
  onSpanClick,
}: SpanRowProps) {
  const duration =
    (node.end_time_unix_nano - node.start_time_unix_nano) / 1_000_000;
  const startOffset =
    ((node.start_time_unix_nano - minTime) / 1_000_000 / totalDuration) * 100;
  const width = (duration / totalDuration) * 100;
  const hasError = node.status.code === "ERROR";
  const hasChildren = node.children.length > 0;
  const isExpanded = !collapsed.has(node.span_id);

  return (
    <>
      <div
        className={`${GRID_COLS} border-b border-border hover:bg-card/30 transition-colors`}
      >
        <div
          className="flex items-center gap-2 py-2 pr-3 min-w-0"
          style={{ paddingLeft: `${node.depth * 20 + 12}px` }}
        >
          {hasChildren ? (
            <button
              onClick={() => onToggle(node.span_id)}
              aria-expanded={isExpanded}
              aria-label={`${isExpanded ? "Collapse" : "Expand"} ${node.name}`}
              className="text-foreground/50 hover:text-foreground w-4 h-4 flex items-center justify-center shrink-0"
            >
              {isExpanded ? (
                <ChevronDown size={12} />
              ) : (
                <ChevronRight size={12} />
              )}
            </button>
          ) : (
            <div className="w-4 shrink-0" />
          )}

          <button
            onClick={() => onSpanClick(node)}
            className="flex-1 text-left min-w-0"
          >
            <div className="flex items-center gap-2">
              <span className="text-sm font-mono truncate">{node.name}</span>
              {hasError && (
                <span className="text-xs px-1 bg-destructive/10 text-destructive shrink-0">
                  ERROR
                </span>
              )}
              {showService && node.service_name && (
                <span className="text-xs px-1 border border-border text-foreground/50 truncate shrink-0 max-w-32">
                  {node.service_name}
                </span>
              )}
            </div>
          </button>

          <span className="text-xs font-mono text-foreground/50 whitespace-nowrap">
            {formatDurationCompact(duration)}
          </span>
        </div>

        <div className="relative h-9">
          <button
            onClick={() => onSpanClick(node)}
            aria-label={`${node.name}, ${formatDurationCompact(duration)}`}
            className={`absolute top-1/2 -translate-y-1/2 h-4 ${
              hasError ? "bg-destructive" : "bg-primary"
            } hover:opacity-80 transition-opacity`}
            style={{
              left: `${startOffset}%`,
              width: `${Math.max(width, 0.5)}%`,
            }}
          />
          {(node.events ?? []).slice(0, MAX_EVENT_MARKERS).map((event, i) => {
            const eventOffset =
              ((event.time_unix_nano - minTime) / 1_000_000 / totalDuration) *
              100;
            return (
              <div
                key={`${event.time_unix_nano}-${i.toString()}`}
                title={event.name}
                className="absolute top-1/2 -translate-y-1/2 -translate-x-1/2 w-1.5 h-1.5 rotate-45 bg-amber-500 pointer-events-auto"
                style={{
                  left: `${Math.min(Math.max(eventOffset, 0), 100)}%`,
                }}
              />
            );
          })}
        </div>
      </div>

      {hasChildren &&
        isExpanded &&
        node.children.map((child) => (
          <SpanRow
            key={child.span_id}
            node={child}
            minTime={minTime}
            totalDuration={totalDuration}
            showService={showService}
            collapsed={collapsed}
            onToggle={onToggle}
            onSpanClick={onSpanClick}
          />
        ))}
    </>
  );
}

export function SpanWaterfall({ spans }: SpanWaterfallProps) {
  const [selectedSpan, setSelectedSpan] = useState<Span | null>(null);
  const [collapsed, setCollapsed] = useState<Set<string>>(new Set());

  const spanTree = useMemo(
    () => (spans && spans.length > 0 ? buildSpanTree(spans) : null),
    [spans],
  );

  const showService = useMemo(
    () => new Set(spans?.map((s) => s.service_name).filter(Boolean)).size > 1,
    [spans],
  );

  if (!spanTree) {
    return <EmptyWaterfall message="No spans to display" />;
  }

  const { roots, minTime, maxTime } = spanTree;
  const totalDuration = (maxTime - minTime) / 1_000_000;

  if (!isFinite(totalDuration) || totalDuration <= 0) {
    return <EmptyWaterfall message="Invalid span timing data" />;
  }

  const toggleExpand = (spanId: string) => {
    setCollapsed((prev) => {
      const next = new Set(prev);
      if (next.has(spanId)) {
        next.delete(spanId);
      } else {
        next.add(spanId);
      }
      return next;
    });
  };

  const expandAll = () => setCollapsed(new Set());
  const collapseAll = () => setCollapsed(collectParentIds(roots));

  return (
    <>
      <div className="border border-border">
        <div className="sticky top-0 z-10 bg-background">
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

          <div className={`${GRID_COLS} border-b border-border bg-card/10`}>
            <div className="px-3 py-2">
              <span className="text-xs font-mono text-foreground/50">
                SPAN
              </span>
            </div>
            <div className="relative py-2 text-xs font-mono text-foreground/50">
              {TICK_FRACTIONS.map((fraction) => (
                <span
                  key={fraction}
                  className="absolute whitespace-nowrap"
                  style={{
                    left: `${fraction * 100}%`,
                    transform:
                      fraction === 0
                        ? "translateX(2px)"
                        : fraction === 1
                          ? "translateX(calc(-100% - 2px))"
                          : "translateX(-50%)",
                  }}
                >
                  {formatDurationCompact(totalDuration * fraction)}
                </span>
              ))}
            </div>
          </div>
        </div>

        <div className="relative">
          <div
            className={`${GRID_COLS} absolute inset-0 pointer-events-none`}
            aria-hidden="true"
          >
            <div />
            <div className="relative">
              {TICK_FRACTIONS.slice(1, -1).map((fraction) => (
                <div
                  key={fraction}
                  className="absolute inset-y-0 border-l border-border/40"
                  style={{ left: `${fraction * 100}%` }}
                />
              ))}
            </div>
          </div>

          {roots.map((node) => (
            <SpanRow
              key={node.span_id}
              node={node}
              minTime={minTime}
              totalDuration={totalDuration}
              showService={showService}
              collapsed={collapsed}
              onToggle={toggleExpand}
              onSpanClick={setSelectedSpan}
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
}
