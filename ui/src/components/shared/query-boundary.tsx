import type { ReactNode } from "react";
import { LoadingState } from "./loading-state";
import { ErrorState } from "./error-state";

interface QueryBoundaryProps {
  isLoading: boolean;
  error?: unknown;
  onRetry?: () => void;
  isEmpty?: boolean;
  emptyMessage?: string;
  loadingFallback?: ReactNode;
  children: ReactNode;
}

export function QueryBoundary({
  isLoading,
  error,
  onRetry,
  isEmpty,
  emptyMessage = "No data to display",
  loadingFallback,
  children,
}: QueryBoundaryProps) {
  if (isLoading) {
    return <>{loadingFallback ?? <LoadingState />}</>;
  }

  if (error) {
    return (
      <ErrorState
        message={error instanceof Error ? error.message : "Unknown error"}
        onRetry={onRetry}
      />
    );
  }

  if (isEmpty) {
    return (
      <div className="flex items-center justify-center h-64">
        <p className="text-foreground/50 text-sm">{emptyMessage}</p>
      </div>
    );
  }

  return <>{children}</>;
}
