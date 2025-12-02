import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

interface MetricFiltersProps {
  selectedService?: string;
  services: string[];
  onServiceChange: (service?: string) => void;
}

export function MetricFilters({
  selectedService,
  services,
  onServiceChange,
}: MetricFiltersProps) {
  return (
    <div className="flex gap-3 mb-4">
      <div className="w-48">
        <Select value={selectedService} onValueChange={onServiceChange}>
          <SelectTrigger>
            <SelectValue placeholder="All services" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All services</SelectItem>
            {services.map((service) => (
              <SelectItem key={service} value={service}>
                {service}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>
    </div>
  );
}
