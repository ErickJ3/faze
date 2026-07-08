interface LogsHeaderProps {
  count: number;
}

export function LogsHeader({ count }: LogsHeaderProps) {
  return (
    <div className="mb-6">
      <h1 className="text-xl font-mono mb-1">Logs</h1>
      <p className="text-sm text-muted-foreground">
        {count >= 100
          ? "Showing the latest 100 logs"
          : `${count} ${count === 1 ? "log" : "logs"} found`}
      </p>
    </div>
  );
}
