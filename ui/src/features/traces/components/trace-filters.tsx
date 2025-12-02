import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

interface TraceFiltersProps {
  selectedService?: string;
  services: string[];
  onServiceChange: (service?: string) => void;
  minDuration?: number;
  maxDuration?: number;
  onMinDurationChange: (duration?: number) => void;
  onMaxDurationChange: (duration?: number) => void;
  searchQuery?: string;
  onSearchChange: (query?: string) => void;
}

export function TraceFilters({
  selectedService,
  services,
  onServiceChange,
  minDuration,
  maxDuration,
  onMinDurationChange,
  onMaxDurationChange,
  searchQuery,
  onSearchChange,
}: TraceFiltersProps) {
  return (
    <div className="flex gap-3 mb-4 flex-wrap">
      <div className="flex-1 min-w-[300px]">
        <Input
          type="text"
          placeholder="Search by trace ID or span name..."
          value={searchQuery ?? ""}
          onChange={(e) => onSearchChange(e.target.value || undefined)}
        />
      </div>

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

      <div className="w-32">
        <Input
          type="number"
          placeholder="Min (ms)"
          value={minDuration ?? ""}
          onChange={(e) =>
            onMinDurationChange(
              e.target.value ? Number(e.target.value) : undefined,
            )
          }
        />
      </div>

      <div className="w-32">
        <Input
          type="number"
          placeholder="Max (ms)"
          value={maxDuration ?? ""}
          onChange={(e) =>
            onMaxDurationChange(
              e.target.value ? Number(e.target.value) : undefined,
            )
          }
        />
      </div>
    </div>
  );
}
