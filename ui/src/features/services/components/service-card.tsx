import { Link } from "@tanstack/react-router";

interface ServiceCardProps {
  name: string;
  onClick?: () => void;
}

export function ServiceCard({ name }: ServiceCardProps) {
  return (
    <Link
      to="/services/$serviceName"
      params={{ serviceName: encodeURIComponent(name) }}
      className="
        block w-full p-4 text-left
        border border-border
        bg-card hover:bg-card/80
        transition-colors
      "
    >
      <div className="flex items-center justify-between">
        <h2 className="font-mono text-sm font-normal">{name}</h2>
        <div className="flex items-center gap-2">
          <div className="w-2 h-2 bg-primary rounded-full" />
          <span className="text-xs text-muted-foreground">active</span>
        </div>
      </div>
    </Link>
  );
}
