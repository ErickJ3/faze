import { createFileRoute } from "@tanstack/react-router";
import { ServicesHeader } from "@/features/services/components/services-header";
import { ServicesList } from "@/features/services/components/services-list";
import { useServicesData } from "@/features/services/hooks/use-services-data";
import { LoadingState } from "@/components/shared/loading-state";
import { ErrorState } from "@/components/shared/error-state";

export const Route = createFileRoute("/services/")({
  component: ServicesPage,
});

function ServicesPage() {
  const { services, count, isLoading, error, refetch } = useServicesData();

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
      <ServicesHeader count={count} />
      <ServicesList services={services} />
    </div>
  );
}
