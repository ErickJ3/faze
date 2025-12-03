import { useState } from "react";
import type { Attributes, AttributeValue } from "@/types";

interface AttributesViewerProps {
  attributes: Attributes;
  title?: string;
}

function renderAttributeValue(value: AttributeValue): string {
  if (
    typeof value === "string" ||
    typeof value === "number" ||
    typeof value === "boolean"
  ) {
    return String(value);
  }

  if (value && typeof value === "object" && "type" in value) {
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

  return String(value);
}

function getValueType(value: AttributeValue): string {
  if (typeof value === "string") return "string";
  if (typeof value === "number")
    return Number.isInteger(value) ? "int" : "double";
  if (typeof value === "boolean") return "bool";
  if (value && typeof value === "object" && "type" in value) return value.type;
  if (Array.isArray(value)) return "array";
  return "unknown";
}

function isArrayValue(value: AttributeValue): boolean {
  if (Array.isArray(value)) return true;
  if (
    value &&
    typeof value === "object" &&
    "type" in value &&
    value.type === "array"
  )
    return true;
  return false;
}

function getArrayItems(value: AttributeValue): AttributeValue[] {
  if (Array.isArray(value)) return value;
  if (
    value &&
    typeof value === "object" &&
    "type" in value &&
    value.type === "array"
  ) {
    return value.value;
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
          const isArray = isArrayValue(value);
          const arrayItems = isArray ? getArrayItems(value) : [];
          const valueType = getValueType(value);

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
                      {valueType}
                    </span>
                  </div>
                  <div className="text-xs font-mono text-foreground mt-1">
                    {isArray && !isExpanded ? (
                      <button
                        onClick={() => toggleExpand(key)}
                        className="text-foreground/50 hover:text-foreground"
                      >
                        [{arrayItems.length} items] (click to expand)
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
                          {arrayItems.map((item, i) => (
                            <div
                              key={i.toString()}
                              className="text-foreground/70"
                            >
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
