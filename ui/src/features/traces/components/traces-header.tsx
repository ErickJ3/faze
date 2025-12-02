interface TracesHeaderProps {
  total: number;
}

export function TracesHeader({ total }: TracesHeaderProps) {
  return (
    <div className="mb-6">
      <h1 className="text-xl font-mono mb-1">Traces</h1>
      <p className="text-sm text-foreground/50">
        {total} {total === 1 ? "trace" : "traces"} found
      </p>
    </div>
  );
}
