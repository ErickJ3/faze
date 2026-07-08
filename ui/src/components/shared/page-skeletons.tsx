import { Skeleton } from "./skeleton";

function PageHeaderSkeleton() {
  return (
    <div className="mb-6 space-y-2">
      <Skeleton className="h-6 w-32" />
      <Skeleton className="h-4 w-44" />
    </div>
  );
}

/** Header + filter row + table rows. For logs and traces. */
export function ListPageSkeleton({
  filters = 3,
  rows = 8,
}: {
  filters?: number;
  rows?: number;
}) {
  return (
    <div>
      <PageHeaderSkeleton />
      <div className="flex gap-3 mb-4 flex-wrap">
        {Array.from({ length: filters }, (_, i) => (
          <Skeleton key={i} className="h-9 w-40" />
        ))}
      </div>
      <div className="border border-border">
        {Array.from({ length: rows }, (_, i) => (
          <div key={i} className="border-b border-border last:border-0 p-3">
            <Skeleton className="h-4 w-full" />
          </div>
        ))}
      </div>
    </div>
  );
}

/** Header + optional filter row + card grid. For metrics and services. */
export function GridPageSkeleton({
  filters = 1,
  cards = 8,
}: {
  filters?: number;
  cards?: number;
}) {
  return (
    <div>
      <PageHeaderSkeleton />
      {filters > 0 && (
        <div className="flex gap-3 mb-4">
          {Array.from({ length: filters }, (_, i) => (
            <Skeleton key={i} className="h-9 w-48" />
          ))}
        </div>
      )}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {Array.from({ length: cards }, (_, i) => (
          <Skeleton key={i} className="h-32 border border-border" />
        ))}
      </div>
    </div>
  );
}

/** Back link + title + overview + content block. For trace and service detail. */
export function DetailPageSkeleton() {
  return (
    <div>
      <div className="mb-6">
        <Skeleton className="h-4 w-28" />
      </div>
      <div className="mb-6 space-y-2">
        <Skeleton className="h-7 w-64" />
        <Skeleton className="h-4 w-48" />
      </div>
      <Skeleton className="h-40 border border-border mb-6" />
      <Skeleton className="h-64 border border-border" />
    </div>
  );
}
