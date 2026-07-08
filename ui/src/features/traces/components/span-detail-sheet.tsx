import { Link } from "@tanstack/react-router";
import type { Span } from "@/types";
import { formatTimestamp, formatNanoDuration } from "@/lib/formatters";
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

  const events = span.events ?? [];
  const links = span.links ?? [];
  const resourceAttributes = span.resource_attributes ?? {};
  const hasResourceInfo =
    Object.keys(resourceAttributes).length > 0 || span.scope != null;
  const droppedCounts = [
    { label: "attributes", count: span.dropped_attributes_count ?? 0 },
    { label: "events", count: span.dropped_events_count ?? 0 },
    { label: "links", count: span.dropped_links_count ?? 0 },
  ].filter((entry) => entry.count > 0);

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
              <p className="text-sm font-mono">{span.kind}</p>
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

          {span.trace_state && (
            <div>
              <h4 className="text-xs text-foreground/50 mb-1">Trace State</h4>
              <p className="text-xs font-mono text-foreground/70 break-all">
                {span.trace_state}
              </p>
            </div>
          )}

          {droppedCounts.length > 0 && (
            <div>
              <h4 className="text-xs text-foreground/50 mb-1">Dropped</h4>
              <p className="text-xs font-mono text-amber-500">
                {droppedCounts
                  .map((entry) => `${entry.count} ${entry.label}`)
                  .join(", ")}
              </p>
            </div>
          )}

          <Tabs defaultValue="attributes" className="w-full">
            <TabsList>
              <TabsTrigger value="attributes">
                Attributes ({Object.keys(span.attributes).length})
              </TabsTrigger>
              {events.length > 0 && (
                <TabsTrigger value="events">Events ({events.length})</TabsTrigger>
              )}
              {links.length > 0 && (
                <TabsTrigger value="links">Links ({links.length})</TabsTrigger>
              )}
              {hasResourceInfo && (
                <TabsTrigger value="resource">Resource</TabsTrigger>
              )}
              <TabsTrigger value="json">JSON</TabsTrigger>
            </TabsList>

            <TabsContent value="attributes" className="mt-4">
              <AttributesViewer attributes={span.attributes} />
            </TabsContent>

            {events.length > 0 && (
              <TabsContent value="events" className="mt-4 space-y-3">
                {events.map((event, i) => (
                  <div
                    key={`${event.time_unix_nano}-${i.toString()}`}
                    className="border border-border p-3 space-y-2"
                  >
                    <div className="flex items-center justify-between gap-2">
                      <span className="text-sm font-mono break-all">
                        {event.name}
                      </span>
                      <span className="text-xs font-mono text-foreground/50 shrink-0">
                        +
                        {formatNanoDuration(
                          span.start_time_unix_nano,
                          event.time_unix_nano,
                        )}
                      </span>
                    </div>
                    <p className="text-xs font-mono text-foreground/50">
                      {formatTimestamp(event.time_unix_nano)}
                    </p>
                    {event.attributes &&
                      Object.keys(event.attributes).length > 0 && (
                        <AttributesViewer
                          attributes={event.attributes}
                          title="Event Attributes"
                        />
                      )}
                    {(event.dropped_attributes_count ?? 0) > 0 && (
                      <p className="text-xs font-mono text-amber-500">
                        {event.dropped_attributes_count} attributes dropped
                      </p>
                    )}
                  </div>
                ))}
              </TabsContent>
            )}

            {links.length > 0 && (
              <TabsContent value="links" className="mt-4 space-y-3">
                {links.map((link, i) => (
                  <div
                    key={`${link.trace_id}-${link.span_id}-${i.toString()}`}
                    className="border border-border p-3 space-y-2"
                  >
                    <div>
                      <h4 className="text-xs text-foreground/50 mb-1">
                        Linked Trace
                      </h4>
                      <Link
                        to="/traces/$traceId"
                        params={{ traceId: link.trace_id }}
                        className="text-xs font-mono text-primary hover:underline break-all"
                      >
                        {link.trace_id}
                      </Link>
                    </div>
                    <div>
                      <h4 className="text-xs text-foreground/50 mb-1">
                        Linked Span
                      </h4>
                      <p className="text-xs font-mono text-foreground/70 break-all">
                        {link.span_id}
                      </p>
                    </div>
                    {link.trace_state && (
                      <p className="text-xs font-mono text-foreground/50 break-all">
                        trace_state: {link.trace_state}
                      </p>
                    )}
                    {link.attributes &&
                      Object.keys(link.attributes).length > 0 && (
                        <AttributesViewer
                          attributes={link.attributes}
                          title="Link Attributes"
                        />
                      )}
                  </div>
                ))}
              </TabsContent>
            )}

            {hasResourceInfo && (
              <TabsContent value="resource" className="mt-4 space-y-4">
                {span.scope && (
                  <div>
                    <h4 className="text-xs text-foreground/50 mb-1">
                      Instrumentation Scope
                    </h4>
                    <p className="text-sm font-mono">
                      {span.scope.name}
                      {span.scope.version && (
                        <span className="text-foreground/50">
                          {" "}
                          v{span.scope.version}
                        </span>
                      )}
                    </p>
                  </div>
                )}
                <AttributesViewer
                  attributes={resourceAttributes}
                  title="Resource Attributes"
                />
              </TabsContent>
            )}

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
