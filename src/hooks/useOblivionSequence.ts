/**
 * useOblivionSequence — GPU-driven fall into the black hole.
 *
 * On touch: simultaneously increases mass and decreases observer distance.
 * The shader re-renders each frame so the disk keeps spinning and
 * lensing evolves naturally.
 */

"use client";

import { useState, useRef, useCallback, useEffect } from "react";

export type OblivionPhase =
  | "IDLE"
  | "FALLING"
  | "SINGULARITY"
  | "VOID"
  | "RESET";

export interface OblivionOverrides {
  zoom?: number;
  mass?: number;
}

// Gravitational freefall easing — starts slow, accelerates
function easeInCubic(t: number): number {
  return t * t * t;
}

const FALL_DURATION = 5000; // ms
const TARGET_ZOOM = 1.5; // deep close
const TARGET_MASS = 10.0; // max mass

export function useOblivionSequence(currentZoom: number, currentMass: number) {
  const [phase, setPhase] = useState<OblivionPhase>("IDLE");
  const phaseRef = useRef<OblivionPhase>("IDLE");
  const [overrides, setOverrides] = useState<OblivionOverrides>({});
  const rafRef = useRef<number>(0);
  const timersRef = useRef<ReturnType<typeof setTimeout>[]>([]);
  const startZoomRef = useRef(currentZoom);
  const startMassRef = useRef(currentMass);

  const go = useCallback((p: OblivionPhase) => {
    phaseRef.current = p;
    setPhase(p);
  }, []);

  const clearTimers = useCallback(() => {
    timersRef.current.forEach(clearTimeout);
    timersRef.current = [];
    cancelAnimationFrame(rafRef.current);
  }, []);

  const triggerEntry = useCallback(() => {
    if (phaseRef.current !== "IDLE") return;

    startZoomRef.current = currentZoom;
    startMassRef.current = currentMass;
    clearTimers();
    go("FALLING");

    const startTime = performance.now();

    const animate = (ts: number) => {
      const elapsed = ts - startTime;
      const t = Math.min(elapsed / FALL_DURATION, 1.0);
      const eased = easeInCubic(t);

      // Zoom decreases: current → 1.5
      const newZoom =
        startZoomRef.current + (TARGET_ZOOM - startZoomRef.current) * eased;
      // Mass increases: current → 10.0
      const newMass =
        startMassRef.current + (TARGET_MASS - startMassRef.current) * eased;

      setOverrides({ zoom: newZoom, mass: newMass });

      if (t < 1.0) {
        rafRef.current = requestAnimationFrame(animate);
      } else {
        // Singularity
        go("SINGULARITY");

        const t1 = setTimeout(() => go("VOID"), 1200);
        timersRef.current.push(t1);

        const t2 = setTimeout(() => {
          go("RESET");
          setOverrides({});
        }, 4700);
        timersRef.current.push(t2);

        const t3 = setTimeout(() => {
          go("IDLE");
          setOverrides({});
        }, 6200);
        timersRef.current.push(t3);
      }
    };

    rafRef.current = requestAnimationFrame(animate);
  }, [currentZoom, currentMass, go, clearTimers]);

  useEffect(() => () => clearTimers(), [clearTimers]);

  return { phase, overrides, triggerEntry };
}
