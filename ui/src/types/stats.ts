export interface TraceStatsSummary {
  total: number;
  errors: number;
  avg_duration_ms: number;
}

export interface ServiceStat {
  name: string;
  trace_count: number;
  error_count: number;
}

export interface ActivityBucket {
  bucket_start_unix_nano: number;
  total: number;
  errors: number;
}

export interface StatsResponse {
  spans: number;
  logs: number;
  metrics: number;
  traces: TraceStatsSummary;
  services: ServiceStat[];
  activity: ActivityBucket[];
}
