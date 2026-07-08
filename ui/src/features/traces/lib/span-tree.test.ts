import { describe, it, expect } from "vitest";
import type { Span } from "@/types";
import { buildSpanTree, collectParentIds } from "./span-tree";

function makeSpan(overrides: Partial<Span> & { span_id: string }): Span {
  return {
    trace_id: "trace1",
    name: `op-${overrides.span_id}`,
    kind: "SERVER",
    start_time_unix_nano: 1_000,
    end_time_unix_nano: 2_000,
    attributes: {},
    status: { code: "OK" },
    ...overrides,
  };
}

describe("buildSpanTree", () => {
  it("nests children under their parents with depths", () => {
    const spans = [
      makeSpan({ span_id: "root", start_time_unix_nano: 1_000 }),
      makeSpan({
        span_id: "child",
        parent_span_id: "root",
        start_time_unix_nano: 1_100,
      }),
      makeSpan({
        span_id: "grandchild",
        parent_span_id: "child",
        start_time_unix_nano: 1_200,
      }),
    ];

    const { roots } = buildSpanTree(spans);

    expect(roots).toHaveLength(1);
    expect(roots[0].span_id).toBe("root");
    expect(roots[0].depth).toBe(0);
    expect(roots[0].children[0].span_id).toBe("child");
    expect(roots[0].children[0].depth).toBe(1);
    expect(roots[0].children[0].children[0].span_id).toBe("grandchild");
    expect(roots[0].children[0].children[0].depth).toBe(2);
  });

  it("promotes spans with a missing parent to roots", () => {
    const spans = [
      makeSpan({ span_id: "a" }),
      makeSpan({ span_id: "orphan", parent_span_id: "not-collected" }),
    ];

    const { roots } = buildSpanTree(spans);

    expect(roots.map((r) => r.span_id).sort()).toEqual(["a", "orphan"]);
  });

  it("recovers spans trapped in a parent cycle", () => {
    const spans = [
      makeSpan({ span_id: "a", parent_span_id: "b" }),
      makeSpan({ span_id: "b", parent_span_id: "a" }),
    ];

    const { roots } = buildSpanTree(spans);

    expect(roots).toHaveLength(1);
    const rendered: string[] = [];
    const stack = [...roots];
    while (stack.length > 0) {
      const node = stack.pop()!;
      rendered.push(node.span_id);
      stack.push(...node.children);
    }
    expect(rendered.sort()).toEqual(["a", "b"]);
  });

  it("computes min and max times ignoring zeroed timestamps", () => {
    const spans = [
      makeSpan({
        span_id: "a",
        start_time_unix_nano: 5_000,
        end_time_unix_nano: 9_000,
      }),
      makeSpan({
        span_id: "b",
        start_time_unix_nano: 3_000,
        end_time_unix_nano: 4_000,
      }),
      makeSpan({
        span_id: "c",
        start_time_unix_nano: 0,
        end_time_unix_nano: 0,
      }),
    ];

    const { minTime, maxTime } = buildSpanTree(spans);

    expect(minTime).toBe(3_000);
    expect(maxTime).toBe(9_000);
  });
});

describe("collectParentIds", () => {
  it("returns only ids of nodes with children", () => {
    const spans = [
      makeSpan({ span_id: "root" }),
      makeSpan({ span_id: "child", parent_span_id: "root" }),
      makeSpan({ span_id: "leaf", parent_span_id: "child" }),
      makeSpan({ span_id: "lone" }),
    ];

    const { roots } = buildSpanTree(spans);
    const ids = collectParentIds(roots);

    expect([...ids].sort()).toEqual(["child", "root"]);
  });
});
