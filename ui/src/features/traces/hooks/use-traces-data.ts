import { useTraces } from "@/hooks/api";
import { useServices } from "@/hooks/api";
import {
  useQueryStates,
  parseAsString,
  parseAsFloat,
  parseAsInteger,
} from "nuqs";
import { useMemo } from "react";

export function useTracesData() {
  const [filters, setFilters] = useQueryStates({
    service: parseAsString,
    min_duration: parseAsFloat,
    max_duration: parseAsFloat,
    search: parseAsString,
    page: parseAsInteger.withDefault(1),
    page_size: parseAsInteger.withDefault(25),
  });

  const { data: servicesData } = useServices();
  const { data, isLoading, error, refetch } = useTraces({
    limit: 100,
    service: filters.service ?? undefined,
    min_duration: filters.min_duration ?? undefined,
    max_duration: filters.max_duration ?? undefined,
  });

  const services = servicesData ?? [];
  const allTraces = data?.traces ?? [];

  const filteredTraces = useMemo(() => {
    if (!filters.search) return allTraces;

    const query = filters.search.toLowerCase();
    return allTraces.filter((trace) => {
      return (
        trace.trace_id.toLowerCase().includes(query) ||
        trace.root_span_name?.toLowerCase().includes(query) ||
        trace.service_name?.toLowerCase().includes(query)
      );
    });
  }, [allTraces, filters.search]);

  const totalItems = filteredTraces.length;
  const totalPages = Math.ceil(totalItems / filters.page_size);
  const currentPage = Math.min(filters.page, totalPages || 1);

  const traces = useMemo(() => {
    const startIndex = (currentPage - 1) * filters.page_size;
    const endIndex = startIndex + filters.page_size;
    return filteredTraces.slice(startIndex, endIndex);
  }, [filteredTraces, currentPage, filters.page_size]);

  const updateFilter = (key: string, value: string | number | undefined) => {
    if (value === "all" || value === undefined || value === "") {
      setFilters({ [key]: null });
    } else {
      setFilters({ [key]: value });
    }
  };

  return {
    traces,
    total: totalItems,
    services,
    filters: {
      service: filters.service ?? undefined,
      min_duration: filters.min_duration ?? undefined,
      max_duration: filters.max_duration ?? undefined,
      search: filters.search ?? undefined,
    },
    pagination: {
      currentPage,
      totalPages,
      pageSize: filters.page_size,
      totalItems,
      onPageChange: (page: number) => setFilters({ page }),
      onPageSizeChange: (size: number) =>
        setFilters({ page_size: size, page: 1 }),
    },
    updateFilter,
    isLoading,
    error,
    refetch,
  };
}
