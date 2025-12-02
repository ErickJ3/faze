import type { Span } from "@/types";
import { formatTimestamp, formatNanoDuration } from "@/lib/formatters";
import { SPAN_KIND_MAP } from "@/lib/constants";
import { StatusBadge } from "@/components/shared/status-badge";
import { AttributesViewer } from "@/components/shared/attributes-viewer";
import { CopyButton } from "@/components/shared/copy-button";
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";

interface SpanDetailSheetProps {
  span: Span | null;
  open: boolean;
  onClose: () => void;
}

export function SpanDetailSheet({ span, open, onClose }: SpanDetailSheetProps) {
  if (!span) return null;

  const duration = formatNanoDuration(
    span.start_time_unix_nano,
    span.end_time_unix_nano,
  );

  return (
    <Sheet open={open} onOpenChange={onClose}>
      <SheetContent className="w-[600px] sm:max-w-[600px] overflow-y-auto bg-background p-0">
        <SheetHeader className="border-b border-border">
          <SheetTitle className="font-mono text-base">Span Details</SheetTitle>
        </SheetHeader>

        <div className="space-y-4 p-6">
          <div>
            <h4 className="text-xs text-foreground/50 mb-1">Name</h4>
            <p className="text-sm font-mono">{span.name}</p>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <h4 className="text-xs text-foreground/50 mb-1">Kind</h4>
              <p className="text-sm font-mono">{SPAN_KIND_MAP[span.kind]}</p>
            </div>

            <div>
              <h4 className="text-xs text-foreground/50 mb-1">Status</h4>
              <StatusBadge status={span.status.code} />
            </div>
          </div>

          <div>
            <h4 className="text-xs text-foreground/50 mb-1">Duration</h4>
            <p className="text-sm font-mono">{duration}</p>
          </div>

          <div>
            <div className="flex items-center justify-between mb-1">
              <h4 className="text-xs text-foreground/50">Span ID</h4>
              <CopyButton text={span.span_id} label="Copy" />
            </div>
            <p className="text-xs font-mono text-foreground/70 break-all">
              {span.span_id}
            </p>
          </div>

          <div>
            <div className="flex items-center justify-between mb-1">
              <h4 className="text-xs text-foreground/50">Trace ID</h4>
              <CopyButton text={span.trace_id} label="Copy" />
            </div>
            <p className="text-xs font-mono text-foreground/70 break-all">
              {span.trace_id}
            </p>
          </div>

          {span.parent_span_id && (
            <div>
              <div className="flex items-center justify-between mb-1">
                <h4 className="text-xs text-foreground/50">Parent Span ID</h4>
                <CopyButton text={span.parent_span_id} label="Copy" />
              </div>
              <p className="text-xs font-mono text-foreground/70 break-all">
                {span.parent_span_id}
              </p>
            </div>
          )}

          {span.service_name && (
            <div>
              <h4 className="text-xs text-foreground/50 mb-1">Service</h4>
              <p className="text-sm font-mono">{span.service_name}</p>
            </div>
          )}

          <div className="grid grid-cols-2 gap-4">
            <div>
              <h4 className="text-xs text-foreground/50 mb-1">Start Time</h4>
              <p className="text-xs font-mono text-foreground/70">
                {formatTimestamp(span.start_time_unix_nano)}
              </p>
            </div>

            <div>
              <h4 className="text-xs text-foreground/50 mb-1">End Time</h4>
              <p className="text-xs font-mono text-foreground/70">
                {formatTimestamp(span.end_time_unix_nano)}
              </p>
            </div>
          </div>

          {span.status.message && (
            <div>
              <h4 className="text-xs text-foreground/50 mb-1">
                Status Message
              </h4>
              <p className="text-sm font-mono text-red-500">
                {span.status.message}
              </p>
            </div>
          )}

          <Tabs defaultValue="attributes" className="w-full">
            <TabsList>
              <TabsTrigger value="attributes">
                Attributes ({Object.keys(span.attributes).length})
              </TabsTrigger>
              <TabsTrigger value="json">JSON</TabsTrigger>
            </TabsList>

            <TabsContent value="attributes" className="mt-4">
              <AttributesViewer attributes={span.attributes} />
            </TabsContent>

            <TabsContent value="json" className="mt-4">
              <div className="relative">
                <div className="absolute top-2 right-2">
                  <CopyButton
                    text={JSON.stringify(span, null, 2)}
                    label="Copy JSON"
                  />
                </div>
                <pre className="text-xs font-mono bg-card border border-border p-4 overflow-x-auto">
                  {JSON.stringify(span, null, 2)}
                </pre>
              </div>
            </TabsContent>
          </Tabs>
        </div>
      </SheetContent>
    </Sheet>
  );
}
