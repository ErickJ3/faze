import { ServiceCard } from "./service-card";

interface ServicesListProps {
  services: string[];
}

export function ServicesList({ services }: ServicesListProps) {
  if (services.length === 0) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-center">
          <p className="text-foreground/50 text-sm">No services found</p>
          <p className="text-foreground/30 text-xs mt-1">
            Start sending telemetry data to see services
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
      {services.map((service) => (
        <ServiceCard key={service} name={service} />
      ))}
    </div>
  );
}
