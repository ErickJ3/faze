import { useState } from "react";
import type { Span } from "@/types";
import { formatNanoDuration } from "@/lib/formatters";
import { SPAN_KIND_MAP } from "@/lib/constants";
import { StatusBadge } from "@/components/shared/status-badge";
import { SpanDetailSheet } from "./span-detail-sheet";

interface SpanTreeProps {
  spans: Span[];
}

interface SpanNode extends Span {
  children: SpanNode[];
  depth: number;
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

function SpanTreeNode({
  node,
  onSpanClick,
}: {
  node: SpanNode;
  onSpanClick: (span: Span) => void;
}) {
  const duration = formatNanoDuration(
    node.start_time_unix_nano,
    node.end_time_unix_nano,
  );

  return (
    <>
      <button
        onClick={() => onSpanClick(node)}
        className="w-full flex items-center gap-3 py-2 px-3 border-b border-border hover:bg-card/50 transition-colors text-left cursor-pointer"
        style={{ paddingLeft: `${node.depth * 24 + 12}px` }}
      >
        <div className="flex-1 min-w-0">
          <div className="font-mono text-sm truncate">{node.name}</div>
          <div className="text-xs text-foreground/50">
            {SPAN_KIND_MAP[node.kind]}
          </div>
        </div>

        <div className="flex items-center gap-3">
          <span className="text-xs font-mono text-foreground/70">
            {duration}
          </span>
          <StatusBadge status={node.status.code} />
        </div>
      </button>

      {node.children.map((child) => (
        <SpanTreeNode
          key={child.span_id}
          node={child}
          onSpanClick={onSpanClick}
        />
      ))}
    </>
  );
}

export function SpanTree({ spans }: SpanTreeProps) {
  const [selectedSpan, setSelectedSpan] = useState<Span | null>(null);
  const tree = buildSpanTree(spans);

  return (
    <>
      <div className="border border-border">
        {tree.map((node) => (
          <SpanTreeNode
            key={node.span_id}
            node={node}
            onSpanClick={setSelectedSpan}
          />
        ))}
      </div>

      <SpanDetailSheet
        span={selectedSpan}
        open={!!selectedSpan}
        onClose={() => setSelectedSpan(null)}
      />
    </>
  );
}
