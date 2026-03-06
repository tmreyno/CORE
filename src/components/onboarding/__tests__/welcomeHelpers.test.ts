// =============================================================================
// welcomeHelpers — formatRelativeTime tests
// =============================================================================

import { describe, it, expect, vi, afterEach } from "vitest";
import { formatRelativeTime } from "../welcomeHelpers";

// Fix "now" for deterministic tests
const NOW = new Date("2026-06-15T12:00:00Z");

describe("formatRelativeTime", () => {
  afterEach(() => {
    vi.useRealTimers();
  });

  function withFakeNow(fn: () => void) {
    vi.useFakeTimers();
    vi.setSystemTime(NOW);
    fn();
    vi.useRealTimers();
  }

  it('returns "Just now" for < 1 minute ago', () => {
    withFakeNow(() => {
      const recent = new Date(NOW.getTime() - 30_000).toISOString(); // 30s ago
      expect(formatRelativeTime(recent)).toBe("Just now");
    });
  });

  it("returns minutes for < 60 minutes", () => {
    withFakeNow(() => {
      const fiveMin = new Date(NOW.getTime() - 5 * 60_000).toISOString();
      expect(formatRelativeTime(fiveMin)).toBe("5m ago");
    });
  });

  it("returns hours for < 24 hours", () => {
    withFakeNow(() => {
      const threeHrs = new Date(NOW.getTime() - 3 * 3_600_000).toISOString();
      expect(formatRelativeTime(threeHrs)).toBe("3h ago");
    });
  });

  it("returns days for < 7 days", () => {
    withFakeNow(() => {
      const twoDays = new Date(NOW.getTime() - 2 * 86_400_000).toISOString();
      expect(formatRelativeTime(twoDays)).toBe("2d ago");
    });
  });

  it("returns locale date string for >= 7 days", () => {
    withFakeNow(() => {
      const twoWeeks = new Date(NOW.getTime() - 14 * 86_400_000).toISOString();
      const result = formatRelativeTime(twoWeeks);
      // Should be a locale date, not a relative "Xd ago" string
      expect(result).not.toContain("ago");
      expect(result).not.toBe("");
    });
  });

  it('returns toLocaleDateString result for invalid date string', () => {
    // new Date("not-a-date") doesn't throw — it creates Invalid Date
    // whose getTime() is NaN, producing negative diffMs/diffMins/etc.
    // The function falls through to toLocaleDateString()
    const result = formatRelativeTime("not-a-date");
    expect(result).toBe("Invalid Date");
  });

  it("handles 1 minute boundary", () => {
    withFakeNow(() => {
      const oneMin = new Date(NOW.getTime() - 60_000).toISOString();
      expect(formatRelativeTime(oneMin)).toBe("1m ago");
    });
  });

  it("handles 1 hour boundary", () => {
    withFakeNow(() => {
      const oneHour = new Date(NOW.getTime() - 3_600_000).toISOString();
      expect(formatRelativeTime(oneHour)).toBe("1h ago");
    });
  });

  it("handles 1 day boundary", () => {
    withFakeNow(() => {
      const oneDay = new Date(NOW.getTime() - 86_400_000).toISOString();
      expect(formatRelativeTime(oneDay)).toBe("1d ago");
    });
  });
});
