import { Skeleton } from "@/components/shared/skeleton";

export function DashboardSkeleton() {
  return (
    <div>
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-6">
        {Array.from({ length: 4 }, (_, i) => (
          <Skeleton key={i} className="h-24 border border-border" />
        ))}
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4 mb-6">
        <Skeleton className="h-64 border border-border" />
        <Skeleton className="h-64 border border-border" />
      </div>

      <Skeleton className="h-72 border border-border" />
    </div>
  );
}
