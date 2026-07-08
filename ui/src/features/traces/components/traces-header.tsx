interface TracesHeaderProps {
  total: number;
}

export function TracesHeader({ total }: TracesHeaderProps) {
  return (
    <div className="mb-6">
      <h1 className="text-xl font-mono mb-1">Traces</h1>
      <p className="text-sm text-muted-foreground">
        {total >= 100
          ? "Showing the latest 100 traces"
          : `${total} ${total === 1 ? "trace" : "traces"} found`}
      </p>
    </div>
  );
}
