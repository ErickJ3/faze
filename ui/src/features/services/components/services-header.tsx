interface ServicesHeaderProps {
  count: number;
}

export function ServicesHeader({ count }: ServicesHeaderProps) {
  return (
    <div className="mb-6">
      <h1 className="text-xl font-mono mb-1">Services</h1>
      <p className="text-sm text-foreground/50">
        {count} {count === 1 ? "service" : "services"} detected
      </p>
    </div>
  );
}
