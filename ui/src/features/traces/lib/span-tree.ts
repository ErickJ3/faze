import type { Span } from "@/types";

export interface SpanNode extends Span {
  children: SpanNode[];
  depth: number;
}

export interface SpanTree {
  roots: SpanNode[];
  minTime: number;
  maxTime: number;
}

/**
 * Builds the parent/child tree for a trace's spans and computes its time
 * bounds. Spans whose parent is missing become roots; nodes trapped in a
 * parent cycle are promoted to roots with the cycle edge pruned, so every
 * span is always rendered exactly once.
 */
export function buildSpanTree(spans: Span[]): SpanTree {
  const spanMap = new Map<string, SpanNode>();
  spans.forEach((span) => {
    spanMap.set(span.span_id, { ...span, children: [], depth: 0 });
  });

  const sorted = [...spans].sort(
    (a, b) => a.start_time_unix_nano - b.start_time_unix_nano,
  );

  const roots: SpanNode[] = [];
  sorted.forEach((span) => {
    const node = spanMap.get(span.span_id)!;
    const parent = span.parent_span_id
      ? spanMap.get(span.parent_span_id)
      : undefined;
    if (parent && parent !== node) {
      parent.children.push(node);
    } else {
      roots.push(node);
    }
  });

  const visited = new Set<string>();
  const walk = (root: SpanNode) => {
    const stack = [{ node: root, depth: root.depth }];
    while (stack.length > 0) {
      const { node, depth } = stack.pop()!;
      visited.add(node.span_id);
      node.depth = depth;
      node.children = node.children.filter(
        (child) => !visited.has(child.span_id),
      );
      node.children.forEach((child) => {
        stack.push({ node: child, depth: depth + 1 });
      });
    }
  };

  roots.forEach(walk);

  // Nodes never reached from a root are part of a parent cycle.
  sorted.forEach((span) => {
    const node = spanMap.get(span.span_id)!;
    if (!visited.has(node.span_id)) {
      roots.push(node);
      walk(node);
    }
  });

  let minTime = Infinity;
  let maxTime = -Infinity;
  for (const span of spans) {
    if (span.start_time_unix_nano > 0 && span.start_time_unix_nano < minTime) {
      minTime = span.start_time_unix_nano;
    }
    if (span.end_time_unix_nano > 0 && span.end_time_unix_nano > maxTime) {
      maxTime = span.end_time_unix_nano;
    }
  }

  return { roots, minTime, maxTime };
}

/** Span ids of every node that has children, across the whole tree. */
export function collectParentIds(roots: SpanNode[]): Set<string> {
  const ids = new Set<string>();
  const stack = [...roots];
  while (stack.length > 0) {
    const node = stack.pop()!;
    if (node.children.length > 0) {
      ids.add(node.span_id);
      stack.push(...node.children);
    }
  }
  return ids;
}
