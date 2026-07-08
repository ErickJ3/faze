interface StatCardProps {
  label: string;
  value: string | number;
  hint?: string;
}

export function StatCard({ label, value, hint }: StatCardProps) {
  return (
    <div className="border border-border p-4 bg-card">
      <div className="text-xs text-foreground/50 mb-2">{label}</div>
      <div className="text-2xl font-mono">{value}</div>
      {hint && <div className="text-xs text-foreground/40 mt-1">{hint}</div>}
    </div>
  );
}
