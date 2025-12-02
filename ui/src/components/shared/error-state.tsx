interface ErrorStateProps {
  message?: string;
  onRetry?: () => void;
}

export function ErrorState({ message, onRetry }: ErrorStateProps) {
  return (
    <div className="flex items-center justify-center h-64">
      <div className="text-center">
        <p className="text-red-500 text-sm mb-2">Error loading data</p>
        {message && (
          <p className="text-foreground/30 text-xs mb-3">{message}</p>
        )}
        {onRetry && (
          <button
            onClick={onRetry}
            className="text-xs border border-border px-3 py-1 hover:bg-card transition-colors"
          >
            Retry
          </button>
        )}
      </div>
    </div>
  );
}
