interface StatCardProps {
  label: string;
  value: string | number;
  hint?: string;
}

export function StatCard({ label, value, hint }: StatCardProps) {
  return (
    <div className="border border-border p-4 bg-card">
      <h2 className="text-xs text-muted-foreground mb-2 font-normal">{label}</h2>
      <div className="text-2xl font-mono">{value}</div>
      {hint && <div className="text-xs text-muted-foreground mt-1">{hint}</div>}
    </div>
  );
}
