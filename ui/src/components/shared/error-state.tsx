import { Button } from "@/components/ui/button";

interface ErrorStateProps {
  title?: string;
  message?: string;
  onRetry?: () => void;
}

export function ErrorState({
  title = "Something went wrong",
  message,
  onRetry,
}: ErrorStateProps) {
  return (
    <div role="alert" className="flex items-center justify-center h-64">
      <div className="text-center">
        <p className="text-sev-error text-sm mb-2">{title}</p>
        {message && (
          <p className="text-muted-foreground text-xs mb-3">{message}</p>
        )}
        {onRetry && (
          <Button variant="outline" size="sm" onClick={onRetry}>
            Retry
          </Button>
        )}
      </div>
    </div>
  );
}
