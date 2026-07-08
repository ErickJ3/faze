import { createFileRoute } from "@tanstack/react-router";
import { ServicesHeader } from "@/features/services/components/services-header";
import { ServicesList } from "@/features/services/components/services-list";
import { useServicesData } from "@/features/services/hooks/use-services-data";
import { QueryBoundary } from "@/components/shared/query-boundary";

export const Route = createFileRoute("/services/")({
  component: ServicesPage,
});

function ServicesPage() {
  const { services, count, isLoading, error, refetch } = useServicesData();

  return (
    <QueryBoundary
      isLoading={isLoading}
      error={error}
      onRetry={() => refetch()}
    >
      <div>
        <ServicesHeader count={count} />
        <ServicesList services={services} />
      </div>
    </QueryBoundary>
  );
}
