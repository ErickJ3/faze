import { createFileRoute, Link } from "@tanstack/react-router";
import { TraceDetails } from "@/features/traces/components/trace-details";
import { useTraceDetails } from "@/features/traces/hooks/use-trace-details";
import { QueryBoundary } from "@/components/shared/query-boundary";
import { DetailPageSkeleton } from "@/components/shared/page-skeletons";

export const Route = createFileRoute("/traces/$traceId")({
  component: TraceDetailPage,
});

function TraceDetailPage() {
  const { traceId } = Route.useParams();
  const { trace, isLoading, error, refetch } = useTraceDetails(traceId);

  return (
    <QueryBoundary
      isLoading={isLoading}
      error={error}
      onRetry={() => refetch()}
      errorTitle="Couldn't load trace"
      isEmpty={!trace}
      emptyMessage="Trace not found"
      loadingFallback={<DetailPageSkeleton />}
    >
      <div>
        <div className="mb-6">
          <Link
            to="/traces"
            search={{ service: undefined }}
            className="text-xs text-muted-foreground hover:text-foreground transition-colors"
          >
            ← Back to traces
          </Link>
        </div>

        {trace && <TraceDetails trace={trace} />}
      </div>
    </QueryBoundary>
  );
}
