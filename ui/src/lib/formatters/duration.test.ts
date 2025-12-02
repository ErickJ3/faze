import { describe, it, expect } from "vitest";
import {
  formatDuration,
  formatDurationCompact,
  formatNanoDuration,
} from "./duration";

describe("formatDuration", () => {
  it("formats microseconds correctly", () => {
    expect(formatDuration(0.5)).toBe("500µs");
    expect(formatDuration(0.001)).toBe("1µs");
    expect(formatDuration(0.999)).toBe("999µs");
  });

  it("formats milliseconds correctly", () => {
    expect(formatDuration(1)).toBe("1.0ms");
    expect(formatDuration(50)).toBe("50.0ms");
    expect(formatDuration(500)).toBe("500.0ms");
    expect(formatDuration(999.9)).toBe("999.9ms");
  });

  it("formats seconds correctly", () => {
    expect(formatDuration(1000)).toBe("1.00s");
    expect(formatDuration(5000)).toBe("5.00s");
    expect(formatDuration(30000)).toBe("30.00s");
    expect(formatDuration(59999)).toBe("60.00s");
  });

  it("formats minutes correctly", () => {
    expect(formatDuration(60000)).toBe("1m 0s");
    expect(formatDuration(90000)).toBe("1m 30s");
    expect(formatDuration(125000)).toBe("2m 5s");
    expect(formatDuration(3600000)).toBe("60m 0s");
  });
});

describe("formatDurationCompact", () => {
  it("formats microseconds compactly", () => {
    expect(formatDurationCompact(0.5)).toBe("500µs");
    expect(formatDurationCompact(0.001)).toBe("1µs");
  });

  it("formats milliseconds compactly", () => {
    expect(formatDurationCompact(1)).toBe("1ms");
    expect(formatDurationCompact(50)).toBe("50ms");
    expect(formatDurationCompact(999)).toBe("999ms");
  });

  it("formats seconds compactly", () => {
    expect(formatDurationCompact(1000)).toBe("1.0s");
    expect(formatDurationCompact(5000)).toBe("5.0s");
    expect(formatDurationCompact(30000)).toBe("30.0s");
  });

  it("formats minutes compactly", () => {
    expect(formatDurationCompact(60000)).toBe("1.0m");
    expect(formatDurationCompact(90000)).toBe("1.5m");
    expect(formatDurationCompact(3600000)).toBe("60.0m");
  });
});

describe("formatNanoDuration", () => {
  it("calculates and formats nanosecond duration", () => {
    const startNano = 1000000000000;
    const endNano = 1000001000000;
    expect(formatNanoDuration(startNano, endNano)).toBe("1.0ms");
  });

  it("handles large nanosecond differences", () => {
    const startNano = 1000000000000;
    const endNano = 1001000000000;
    expect(formatNanoDuration(startNano, endNano)).toBe("1.00s");
  });

  it("handles small nanosecond differences", () => {
    const startNano = 1000000000000;
    const endNano = 1000000500000;
    expect(formatNanoDuration(startNano, endNano)).toBe("500µs");
  });

  it("handles microsecond precision", () => {
    const startNano = 1000000000000;
    const endNano = 1000000001000;
    expect(formatNanoDuration(startNano, endNano)).toBe("1µs");
  });
});
