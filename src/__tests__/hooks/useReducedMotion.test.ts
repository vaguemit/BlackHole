import { describe, it, expect, beforeEach, vi } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useReducedMotion } from "@/hooks/useReducedMotion";

interface MockMediaQueryList {
  matches: boolean;
  addEventListener: ReturnType<typeof vi.fn>;
  removeEventListener: ReturnType<typeof vi.fn>;
  dispatchChange: (matches: boolean) => void;
}

function installMatchMedia(initial: boolean): MockMediaQueryList {
  let listener: ((e: MediaQueryListEvent) => void) | null = null;
  const mql: MockMediaQueryList = {
    matches: initial,
    addEventListener: vi.fn((_event: string, handler: EventListener) => {
      listener = handler as (e: MediaQueryListEvent) => void;
    }),
    removeEventListener: vi.fn(() => {
      listener = null;
    }),
    dispatchChange(matches: boolean) {
      this.matches = matches;
      listener?.({ matches } as MediaQueryListEvent);
    },
  };
  vi.stubGlobal(
    "matchMedia",
    vi.fn(() => mql),
  );
  return mql;
}

describe("useReducedMotion", () => {
  beforeEach(() => {
    vi.unstubAllGlobals();
  });

  it("returns false when prefers-reduced-motion is not set", () => {
    installMatchMedia(false);
    const { result } = renderHook(() => useReducedMotion());
    expect(result.current).toBe(false);
  });

  it("returns true when prefers-reduced-motion is reduce on initial render", () => {
    installMatchMedia(true);
    const { result } = renderHook(() => useReducedMotion());
    expect(result.current).toBe(true);
  });

  it("flips reactively when the media query change event fires", () => {
    const mql = installMatchMedia(false);
    const { result } = renderHook(() => useReducedMotion());
    expect(result.current).toBe(false);

    act(() => {
      mql.dispatchChange(true);
    });
    expect(result.current).toBe(true);

    act(() => {
      mql.dispatchChange(false);
    });
    expect(result.current).toBe(false);
  });

  it("removes its event listener on unmount", () => {
    const mql = installMatchMedia(false);
    const { unmount } = renderHook(() => useReducedMotion());
    expect(mql.addEventListener).toHaveBeenCalledTimes(1);
    unmount();
    expect(mql.removeEventListener).toHaveBeenCalledTimes(1);
  });
});
