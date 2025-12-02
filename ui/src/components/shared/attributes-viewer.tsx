import type { Attributes, AttributeValue } from "@/types";
import { useState } from "react";

interface AttributesViewerProps {
  attributes: Attributes;
  title?: string;
}

function renderAttributeValue(value: AttributeValue): string {
  switch (value.type) {
    case "string":
      return value.value;
    case "int":
    case "double":
      return String(value.value);
    case "bool":
      return value.value ? "true" : "false";
    case "bytes":
      return `[${value.value.length} bytes]`;
    case "array":
      return `[${value.value.length} items]`;
    default:
      return "unknown";
  }
}

export function AttributesViewer({
  attributes,
  title = "Attributes",
}: AttributesViewerProps) {
  const [expandedKeys, setExpandedKeys] = useState<Set<string>>(new Set());

  const entries = Object.entries(attributes);

  if (entries.length === 0) {
    return <div className="text-xs text-foreground/30 py-2">No attributes</div>;
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
      <h4 className="text-xs font-mono text-foreground/50 mb-2">{title}</h4>
      <div className="border border-border">
        {entries.map(([key, value], index) => {
          const isExpanded = expandedKeys.has(key);
          const isArray = value.type === "array";

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
                    <span className="text-xs text-foreground/30">
                      {value.type}
                    </span>
                  </div>
                  <div className="text-xs font-mono text-foreground mt-1">
                    {isArray && !isExpanded ? (
                      <button
                        onClick={() => toggleExpand(key)}
                        className="text-foreground/50 hover:text-foreground"
                      >
                        [{value.value.length} items] (click to expand)
                      </button>
                    ) : isArray && isExpanded ? (
                      <div>
                        <button
                          onClick={() => toggleExpand(key)}
                          className="text-foreground/50 hover:text-foreground mb-1"
                        >
                          (click to collapse)
                        </button>
                        <div className="pl-4 space-y-1 border-l-2 border-foreground/10">
                          {value.value.map((item, i) => (
                            <div key={i} className="text-foreground/70">
                              [{i}]: {renderAttributeValue(item)}
                            </div>
                          ))}
                        </div>
                      </div>
                    ) : (
                      renderAttributeValue(value)
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
