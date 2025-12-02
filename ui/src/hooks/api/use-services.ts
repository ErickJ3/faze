import { useQuery } from "@tanstack/react-query";
import { servicesService } from "@/lib/api/services";

export function useServices() {
  return useQuery({
    queryKey: ["services"],
    queryFn: () => servicesService.getServices(),
  });
}
