import { createFileRoute } from "@tanstack/react-router";
import { ServicesHeader } from "@/features/services/components/services-header";
import { ServicesList } from "@/features/services/components/services-list";
import { useServicesData } from "@/features/services/hooks/use-services-data";
import { QueryBoundary } from "@/components/shared/query-boundary";
import { GridPageSkeleton } from "@/components/shared/page-skeletons";

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
      errorTitle="Couldn't load services"
      loadingFallback={<GridPageSkeleton filters={0} />}
    >
      <div>
        <ServicesHeader count={count} />
        <ServicesList services={services} />
      </div>
    </QueryBoundary>
  );
}
