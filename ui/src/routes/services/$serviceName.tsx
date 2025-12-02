import { createFileRoute, Link } from "@tanstack/react-router";
import { useServiceDetails } from "@/features/services/hooks/use-service-details";
import { LoadingState } from "@/components/shared/loading-state";
import { ErrorState } from "@/components/shared/error-state";
import { ServiceOverview } from "@/features/services/components/service-overview";
import { ServiceChart } from "@/features/services/components/service-chart";
import { ServiceTraces } from "@/features/services/components/service-traces";
import { ServiceLogs } from "@/features/services/components/service-logs";
import { ServiceMetrics } from "@/features/services/components/service-metrics";

export const Route = createFileRoute("/services/$serviceName")({
  component: ServiceDetailPage,
});

function ServiceDetailPage() {
  const { serviceName } = Route.useParams();
  const decodedServiceName = decodeURIComponent(serviceName);

  const {
    stats,
    recentTraces,
    allTraces,
    recentLogs,
    metrics,
    isLoading,
    error,
    refetch,
  } = useServiceDetails(decodedServiceName);

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

  return (
    <div>
      <div className="mb-6">
        <Link
          to="/services"
          className="text-xs text-foreground/50 hover:text-foreground transition-colors"
        >
          ← Back to services
        </Link>
      </div>

      <div className="mb-6">
        <h1 className="text-2xl font-mono">{decodedServiceName}</h1>
        <p className="text-sm text-foreground/50 mt-1">
          Service overview and metrics
        </p>
      </div>

      <ServiceOverview stats={stats} serviceName={decodedServiceName} />

      <div className="mt-6">
        <ServiceChart traces={allTraces} />
      </div>

      <div className="grid grid-cols-1 gap-6 mt-6">
        <ServiceTraces traces={recentTraces} serviceName={decodedServiceName} />
        <ServiceLogs logs={recentLogs} serviceName={decodedServiceName} />
        <ServiceMetrics metrics={metrics} serviceName={decodedServiceName} />
      </div>
    </div>
  );
}
