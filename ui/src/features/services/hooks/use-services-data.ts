import { useServices } from "@/hooks/api";

export function useServicesData() {
  const { data, isLoading, error, refetch } = useServices();

  const services = data ?? [];

  return {
    services,
    count: services.length,
    isLoading,
    error,
    refetch,
  };
}
