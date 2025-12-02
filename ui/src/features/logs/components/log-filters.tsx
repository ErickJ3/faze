import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

interface LogFiltersProps {
  selectedService?: string;
  services: string[];
  onServiceChange: (service?: string) => void;
  selectedLevel?: string;
  onLevelChange: (level?: string) => void;
  searchQuery?: string;
  onSearchChange: (query?: string) => void;
}

const LOG_LEVELS = ["TRACE", "DEBUG", "INFO", "WARN", "ERROR", "FATAL"];

export function LogFilters({
  selectedService,
  services,
  onServiceChange,
  selectedLevel,
  onLevelChange,
  searchQuery,
  onSearchChange,
}: LogFiltersProps) {
  return (
    <div className="flex gap-3 mb-4 flex-wrap">
      <div className="flex-1 min-w-[300px]">
        <Input
          type="text"
          placeholder="Search log body..."
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
        <Select value={selectedLevel} onValueChange={onLevelChange}>
          <SelectTrigger>
            <SelectValue placeholder="All levels" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All levels</SelectItem>
            {LOG_LEVELS.map((level) => (
              <SelectItem key={level} value={level}>
                {level}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>
    </div>
  );
}
