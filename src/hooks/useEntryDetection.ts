/**
 * useEntryDetection — PRD §4
 *
 * Unified pointer (mouse + touch) hit-detection against the event horizon
 * projected circle on screen. Triggers onEntry when user taps/clicks inside.
 */

"use client";

import { useEffect, RefObject } from "react";

interface EntryDetectionOptions {
  canvasRef: RefObject<HTMLCanvasElement | null>;
  horizonScreenRadius: number; // pixels
  onEntry: () => void;
  disabled: boolean; // true during oblivion sequence
}

export function useEntryDetection({
  canvasRef,
  horizonScreenRadius,
  onEntry,
  disabled,
}: EntryDetectionOptions): void {
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas || disabled || horizonScreenRadius <= 0) return;

    const handlePointerDown = (e: PointerEvent) => {
      if (disabled) return;
      e.preventDefault();

      const rect = canvas.getBoundingClientRect();
      const centerX = rect.left + rect.width / 2;
      const centerY = rect.top + rect.height / 2;

      const dx = e.clientX - centerX;
      const dy = e.clientY - centerY;
      const dist = Math.sqrt(dx * dx + dy * dy);

      // 10% generous margin per PRD
      if (dist <= horizonScreenRadius * 1.1) {
        onEntry();
      }
    };

    canvas.addEventListener("pointerdown", handlePointerDown, {
      passive: false,
    });

    return () => {
      canvas.removeEventListener("pointerdown", handlePointerDown);
    };
  }, [canvasRef, horizonScreenRadius, onEntry, disabled]);
}
