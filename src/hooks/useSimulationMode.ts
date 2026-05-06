import { useCallback, useRef, useState } from "react";

export type SimulationMode = "interactive" | "cinematic" | "transitioning";

export type CinematicVariant = "orbit" | "dive";

interface UseSimulationModeApi {
  mode: SimulationMode;
  cinematicVariant: CinematicVariant | null;
  isInteractive: boolean;
  isCinematic: boolean;
  isTransitioning: boolean;
  enterCinematic: (variant: CinematicVariant) => void;
  enterInteractive: () => void;
}

const TRANSITION_MS = 500;

export function useSimulationMode(): UseSimulationModeApi {
  const [mode, setMode] = useState<SimulationMode>("interactive");
  const [cinematicVariant, setCinematicVariant] =
    useState<CinematicVariant | null>(null);
  const transitionTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  const clearPending = () => {
    if (transitionTimer.current) {
      clearTimeout(transitionTimer.current);
      transitionTimer.current = null;
    }
  };

  const enterCinematic = useCallback((variant: CinematicVariant) => {
    setMode((current) => {
      if (current === "cinematic" || current === "transitioning")
        return current;
      clearPending();
      setCinematicVariant(variant);
      transitionTimer.current = setTimeout(() => {
        setMode("cinematic");
        transitionTimer.current = null;
      }, TRANSITION_MS);
      return "transitioning";
    });
  }, []);

  const enterInteractive = useCallback(() => {
    setMode((current) => {
      if (current === "interactive" || current === "transitioning")
        return current;
      clearPending();
      transitionTimer.current = setTimeout(() => {
        setMode("interactive");
        setCinematicVariant(null);
        transitionTimer.current = null;
      }, TRANSITION_MS);
      return "transitioning";
    });
  }, []);

  return {
    mode,
    cinematicVariant,
    isInteractive: mode === "interactive",
    isCinematic: mode === "cinematic",
    isTransitioning: mode === "transitioning",
    enterCinematic,
    enterInteractive,
  };
}
