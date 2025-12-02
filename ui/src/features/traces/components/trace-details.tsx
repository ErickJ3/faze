import type { Trace } from "@/types";
import { SpanWaterfall } from "./span-waterfall";
import { CopyButton } from "@/components/shared/copy-button";

interface TraceDetailsProps {
  trace: Trace;
}

export function TraceDetails({ trace }: TraceDetailsProps) {
  return (
    <div>
      <div className="mb-6">
        <h2 className="text-lg font-mono mb-2">Trace Details</h2>
        <div className="text-sm text-foreground/50 space-y-2">
          <div className="flex items-center justify-between">
            <div>
              <span className="text-foreground/30">ID:</span>{" "}
              <span className="font-mono text-xs">{trace.trace_id}</span>
            </div>
            <CopyButton text={trace.trace_id} label="Copy ID" />
          </div>
          {trace.service_name && (
            <div>
              <span className="text-foreground/30">Service:</span>{" "}
              <span className="font-mono">{trace.service_name}</span>
            </div>
          )}
          <div>
            <span className="text-foreground/30">Spans:</span>{" "}
            <span className="font-mono">{trace.spans.length}</span>
          </div>
        </div>
      </div>

      <SpanWaterfall spans={trace.spans} />
    </div>
  );
}
