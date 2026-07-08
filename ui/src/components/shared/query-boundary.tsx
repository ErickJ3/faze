import type { ReactNode } from "react";
import { LoadingState } from "./loading-state";
import { ErrorState } from "./error-state";
import { getErrorMessage } from "@/lib/errors";

interface QueryBoundaryProps {
  isLoading: boolean;
  error?: unknown;
  onRetry?: () => void;
  errorTitle?: string;
  isEmpty?: boolean;
  emptyMessage?: string;
  loadingFallback?: ReactNode;
  children: ReactNode;
}

export function QueryBoundary({
  isLoading,
  error,
  onRetry,
  errorTitle,
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
        title={errorTitle}
        message={getErrorMessage(error)}
        onRetry={onRetry}
      />
    );
  }

  if (isEmpty) {
    return (
      <div className="flex items-center justify-center h-64">
        <p className="text-muted-foreground text-sm">{emptyMessage}</p>
      </div>
    );
  }

  return <>{children}</>;
}
