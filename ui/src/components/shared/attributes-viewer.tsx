import { useState } from "react";
import type { Attributes, AttributeValue } from "@/types";

interface AttributesViewerProps {
  attributes: Attributes;
  title?: string;
}

function isMapValue(
  value: AttributeValue,
): value is { [key: string]: AttributeValue } {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function renderAttributeValue(value: AttributeValue): string {
  if (
    typeof value === "string" ||
    typeof value === "number" ||
    typeof value === "boolean"
  ) {
    return String(value);
  }

  if (Array.isArray(value)) {
    return `[${value.length} items]`;
  }

  if (isMapValue(value)) {
    return `{${Object.keys(value).length} entries}`;
  }

  return String(value);
}

function getValueType(value: AttributeValue): string {
  if (typeof value === "string") return "string";
  if (typeof value === "number")
    return Number.isInteger(value) ? "int" : "double";
  if (typeof value === "boolean") return "bool";
  if (Array.isArray(value)) return "array";
  if (isMapValue(value)) return "map";
  return "unknown";
}

function isExpandable(value: AttributeValue): boolean {
  return Array.isArray(value) || isMapValue(value);
}

function getChildEntries(value: AttributeValue): [string, AttributeValue][] {
  if (Array.isArray(value)) {
    return value.map((item, i) => [`[${i}]`, item]);
  }
  if (isMapValue(value)) {
    return Object.entries(value);
  }
  return [];
}

export function AttributesViewer({
  attributes,
  title = "Attributes",
}: AttributesViewerProps) {
  const [expandedKeys, setExpandedKeys] = useState<Set<string>>(new Set());
  const entries = Object.entries(attributes);

  if (entries.length === 0) {
    return <div className="text-xs text-muted-foreground py-2">No attributes</div>;
  }

  const toggleExpand = (key: string) => {
    const newExpanded = new Set(expandedKeys);
    if (newExpanded.has(key)) {
      newExpanded.delete(key);
    } else {
      newExpanded.add(key);
    }
    setExpandedKeys(newExpanded);
  };

  return (
    <div>
      <h4 className="text-xs font-mono text-muted-foreground mb-2">{title}</h4>
      <div className="border border-border">
        {entries.map(([key, value], index) => {
          const isExpanded = expandedKeys.has(key);
          const expandable = isExpandable(value);
          const childEntries = expandable ? getChildEntries(value) : [];
          const valueType = getValueType(value);
          const collapsedLabel = renderAttributeValue(value);

          return (
            <div
              key={key}
              className={`${index > 0 ? "border-t border-border" : ""}`}
            >
              <div className="flex items-start gap-2 p-2 hover:bg-card/30 transition-colors">
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="text-xs font-mono text-foreground/70 break-all">
                      {key}
                    </span>
                    <span className="text-xs text-muted-foreground">
                      {valueType}
                    </span>
                  </div>
                  <div className="text-xs font-mono text-foreground mt-1">
                    {expandable && !isExpanded ? (
                      <button
                        type="button"
                        aria-expanded={false}
                        onClick={() => toggleExpand(key)}
                        className="text-muted-foreground hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                      >
                        {collapsedLabel} (click to expand)
                      </button>
                    ) : expandable && isExpanded ? (
                      <div>
                        <button
                          type="button"
                          aria-expanded={true}
                          onClick={() => toggleExpand(key)}
                          className="text-muted-foreground hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring mb-1"
                        >
                          (click to collapse)
                        </button>
                        <div className="pl-4 space-y-1 border-l-2 border-foreground/10">
                          {childEntries.map(([childKey, item]) => (
                            <div key={childKey} className="text-foreground/70">
                              {childKey}: {renderAttributeValue(item)}
                            </div>
                          ))}
                        </div>
                      </div>
                    ) : (
                      collapsedLabel
                    )}
                  </div>
                </div>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
