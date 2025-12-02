interface MetricsHeaderProps {
  count: number;
}

export function MetricsHeader({ count }: MetricsHeaderProps) {
  return (
    <div className="mb-6">
      <h1 className="text-xl font-mono mb-1">Metrics</h1>
      <p className="text-sm text-foreground/50">
        {count} {count === 1 ? "metric" : "metrics"} found
      </p>
    </div>
  );
}
