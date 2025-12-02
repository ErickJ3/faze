interface StatCardProps {
  label: string;
  value: string | number;
  trend?: {
    value: number;
    direction: "up" | "down";
  };
}

export function StatCard({ label, value, trend }: StatCardProps) {
  return (
    <div className="border border-border p-4 bg-card">
      <div className="text-xs text-foreground/50 mb-2">{label}</div>
      <div className="flex items-end justify-between">
        <div className="text-2xl font-mono">{value}</div>
        {trend && (
          <div
            className={`text-xs font-mono ${
              trend.direction === "up" ? "text-green-500" : "text-red-500"
            }`}
          >
            {trend.direction === "up" ? "↑" : "↓"} {trend.value}%
          </div>
        )}
      </div>
    </div>
  );
}
