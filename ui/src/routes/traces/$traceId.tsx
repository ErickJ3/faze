import { createFileRoute, Link } from "@tanstack/react-router";
import { TraceDetails } from "@/features/traces/components/trace-details";
import { useTraceDetails } from "@/features/traces/hooks/use-trace-details";
import { LoadingState } from "@/components/shared/loading-state";
import { ErrorState } from "@/components/shared/error-state";

export const Route = createFileRoute("/traces/$traceId")({
  component: TraceDetailPage,
});

function TraceDetailPage() {
  const { traceId } = Route.useParams();
  const { trace, isLoading, error, refetch } = useTraceDetails(traceId);

  if (isLoading) {
    return <LoadingState />;
  }

  if (error) {
    return (
      <ErrorState
        message={error instanceof Error ? error.message : "Unknown error"}
        onRetry={() => refetch()}
      />
    );
  }

  if (!trace) {
    return (
      <div className="flex items-center justify-center h-64">
        <p className="text-foreground/50 text-sm">Trace not found</p>
      </div>
    );
  }

  return (
    <div>
      <div className="mb-6">
        <Link
          to="/traces"
          search={{ service: undefined }}
          className="text-xs text-foreground/50 hover:text-foreground transition-colors"
        >
          ← Back to traces
        </Link>
      </div>

      <TraceDetails trace={trace} />
    </div>
  );
}
