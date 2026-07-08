import { useLogs } from "@/hooks/api";
import { useServices } from "@/hooks/api";
import { SEVERITY_LEVEL_MAP } from "@/lib/constants";
import { useQueryStates, parseAsString, parseAsInteger } from "nuqs";
import { useMemo } from "react";

export function useLogsData() {
  const [filters, setFilters] = useQueryStates({
    service: parseAsString,
    level: parseAsString,
    search: parseAsString,
    page: parseAsInteger.withDefault(1),
    page_size: parseAsInteger.withDefault(25),
  });

  const { data: servicesData } = useServices();
  const { data, isLoading, error, refetch } = useLogs({
    limit: 100,
    service: filters.service ?? undefined,
  });

  const services = servicesData ?? [];
  const allLogs = data ?? [];

  const filteredLogs = useMemo(() => {
    let result = allLogs;

    if (filters.level) {
      const level = filters.level;
      result = result.filter(
        (log) =>
          log.severity_level &&
          SEVERITY_LEVEL_MAP[log.severity_level] === level,
      );
    }

    if (filters.search) {
      const query = filters.search.toLowerCase();
      result = result.filter((log) => {
        return (
          log.body.toLowerCase().includes(query) ||
          log.service_name?.toLowerCase().includes(query)
        );
      });
    }

    return result;
  }, [allLogs, filters.level, filters.search]);

  const totalItems = filteredLogs.length;
  const totalPages = Math.ceil(totalItems / filters.page_size);
  const currentPage = Math.min(filters.page, totalPages || 1);

  const logs = useMemo(() => {
    const startIndex = (currentPage - 1) * filters.page_size;
    const endIndex = startIndex + filters.page_size;
    return filteredLogs.slice(startIndex, endIndex);
  }, [filteredLogs, currentPage, filters.page_size]);

  const updateFilter = (key: string, value: string | undefined) => {
    if (value === "all" || value === undefined || value === "") {
      setFilters({ [key]: null });
    } else {
      setFilters({ [key]: value });
    }
  };

  return {
    logs,
    services,
    filters: {
      service: filters.service ?? undefined,
      level: filters.level ?? undefined,
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
