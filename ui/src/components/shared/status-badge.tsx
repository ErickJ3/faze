import type { StatusCode } from "@/types";
import { statusBgColors, statusColors } from "@/lib/colors";
import { STATUS_CODE_MAP } from "@/lib/constants";

interface StatusBadgeProps {
  status: StatusCode;
}

export function StatusBadge({ status }: StatusBadgeProps) {
  return (
    <span
      className={`px-2 py-0.5 text-xs font-mono ${statusColors[status]} ${statusBgColors[status]}`}
    >
      {STATUS_CODE_MAP[status]}
    </span>
  );
}
