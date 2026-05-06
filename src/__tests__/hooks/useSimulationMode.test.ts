import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { act, renderHook } from "@testing-library/react";
import { useSimulationMode } from "@/hooks/useSimulationMode";

describe("useSimulationMode mutex state machine", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("starts in interactive mode", () => {
    const { result } = renderHook(() => useSimulationMode());
    expect(result.current.mode).toBe("interactive");
    expect(result.current.cinematicVariant).toBeNull();
    expect(result.current.isInteractive).toBe(true);
  });

  it("interactive -> transitioning -> cinematic on enterCinematic", () => {
    const { result } = renderHook(() => useSimulationMode());

    act(() => {
      result.current.enterCinematic("orbit");
    });
    expect(result.current.mode).toBe("transitioning");
    expect(result.current.cinematicVariant).toBe("orbit");

    act(() => {
      vi.advanceTimersByTime(500);
    });
    expect(result.current.mode).toBe("cinematic");
    expect(result.current.cinematicVariant).toBe("orbit");
  });

  it("ignores enterCinematic while transitioning", () => {
    const { result } = renderHook(() => useSimulationMode());

    act(() => {
      result.current.enterCinematic("orbit");
    });
    const variantBefore = result.current.cinematicVariant;

    act(() => {
      result.current.enterCinematic("dive");
    });
    expect(result.current.mode).toBe("transitioning");
    expect(result.current.cinematicVariant).toBe(variantBefore);
  });

  it("cinematic -> transitioning -> interactive on enterInteractive", () => {
    const { result } = renderHook(() => useSimulationMode());

    act(() => {
      result.current.enterCinematic("dive");
      vi.advanceTimersByTime(500);
    });
    expect(result.current.mode).toBe("cinematic");

    act(() => {
      result.current.enterInteractive();
    });
    expect(result.current.mode).toBe("transitioning");

    act(() => {
      vi.advanceTimersByTime(500);
    });
    expect(result.current.mode).toBe("interactive");
    expect(result.current.cinematicVariant).toBeNull();
  });

  it("enterCinematic is idempotent when already cinematic", () => {
    const { result } = renderHook(() => useSimulationMode());

    act(() => {
      result.current.enterCinematic("orbit");
      vi.advanceTimersByTime(500);
    });
    expect(result.current.mode).toBe("cinematic");
    expect(result.current.cinematicVariant).toBe("orbit");

    act(() => {
      result.current.enterCinematic("dive");
    });
    expect(result.current.mode).toBe("cinematic");
    expect(result.current.cinematicVariant).toBe("orbit");
  });

  it("enterInteractive is idempotent when already interactive", () => {
    const { result } = renderHook(() => useSimulationMode());

    act(() => {
      result.current.enterInteractive();
    });
    expect(result.current.mode).toBe("interactive");
  });
});
