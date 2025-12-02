export function formatDuration(ms: number): string {
  if (ms < 1) {
    return `${(ms * 1000).toFixed(0)}µs`;
  }

  if (ms < 1000) {
    return `${ms.toFixed(1)}ms`;
  }

  const seconds = ms / 1000;
  if (seconds < 60) {
    return `${seconds.toFixed(2)}s`;
  }

  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = (seconds % 60).toFixed(0);
  return `${minutes}m ${remainingSeconds}s`;
}

export function formatDurationCompact(ms: number): string {
  if (ms < 1) {
    return `${(ms * 1000).toFixed(0)}µs`;
  }

  if (ms < 1000) {
    return `${ms.toFixed(0)}ms`;
  }

  const seconds = ms / 1000;
  if (seconds < 60) {
    return `${seconds.toFixed(1)}s`;
  }

  const minutes = seconds / 60;
  return `${minutes.toFixed(1)}m`;
}

export function formatNanoDuration(startNano: number, endNano: number): string {
  const durationNano = endNano - startNano;
  const ms = durationNano / 1_000_000;
  return formatDuration(ms);
}
