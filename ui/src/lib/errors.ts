import { AxiosError } from "axios";

/**
 * Maps an unknown error thrown by a query into a short, human-friendly
 * message. Network and common HTTP failures get contextual copy; anything
 * else falls back to the raw message so detail is never lost.
 */
export function getErrorMessage(error: unknown): string {
  if (error instanceof AxiosError) {
    if (error.code === "ERR_NETWORK") {
      return "Can't reach the collector. Make sure it's running.";
    }
    if (error.code === "ECONNABORTED") {
      return "The request timed out. The collector may be busy.";
    }

    const status = error.response?.status;
    if (status === 404) return "The requested data wasn't found.";
    if (status === 429) return "Too many requests. Slow down and retry.";
    if (status != null && status >= 500) {
      return "The collector hit a server error. Try again shortly.";
    }
    if (status != null) {
      return `The collector returned an error (${status}).`;
    }
    return "The request failed. Check your connection and try again.";
  }

  if (error instanceof Error && error.message) {
    return error.message;
  }

  return "An unexpected error occurred.";
}
