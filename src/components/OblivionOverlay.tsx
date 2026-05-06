"use client";

/**
 * OblivionOverlay — minimal overlays for the fall sequence.
 *
 * The actual "falling" visual is handled by the GPU shader (zoom uniform
 * is animated by useOblivionSequence). This component only handles:
 *   - SINGULARITY: white flash
 *   - VOID: pure black
 *   - RESET: fade back out
 *
 * NO fake rings. NO CSS scale. The shader does all the visual work.
 */

import { useEffect, useRef } from "react";
import type { OblivionPhase } from "@/hooks/useOblivionSequence";

interface Props {
  phase: OblivionPhase;
}

export function OblivionOverlay({ phase }: Props) {
  const flashRef = useRef<HTMLDivElement>(null);
  const voidRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const flash = flashRef.current;
    const voidEl = voidRef.current;
    if (!flash || !voidEl) return;

    switch (phase) {
      case "IDLE": {
        flash.style.transition = "opacity 0.5s ease-out";
        flash.style.opacity = "0";
        voidEl.style.transition = "opacity 1s ease-out";
        voidEl.style.opacity = "0";
        break;
      }

      case "FALLING": {
        // No overlay during fall — the shader handles everything
        flash.style.opacity = "0";
        voidEl.style.opacity = "0";
        break;
      }

      case "SINGULARITY": {
        // White flash — peaks then fades to black
        flash.style.transition = "none";
        flash.style.opacity = "0";
        requestAnimationFrame(() => {
          requestAnimationFrame(() => {
            if (!flash) return;
            flash.style.transition = "opacity 0.2s ease-in";
            flash.style.opacity = "1";
            // Fade flash out after 400ms
            setTimeout(() => {
              flash.style.transition = "opacity 0.6s ease-out";
              flash.style.opacity = "0";
            }, 400);
          });
        });
        // Black void starts fading in behind the flash
        voidEl.style.transition = "opacity 0.8s ease-in 0.3s"; // 0.3s delay
        voidEl.style.opacity = "1";
        break;
      }

      case "VOID": {
        // Pure black
        flash.style.opacity = "0";
        voidEl.style.transition = "none";
        voidEl.style.opacity = "1";
        break;
      }

      case "RESET": {
        // Fade out
        flash.style.opacity = "0";
        voidEl.style.transition = "opacity 1.5s ease-out";
        voidEl.style.opacity = "0";
        break;
      }
    }
  }, [phase]);

  if (phase === "IDLE") return null;

  return (
    <>
      {/* White flash at the singularity */}
      <div
        ref={flashRef}
        style={{
          position: "absolute",
          inset: 0,
          pointerEvents: "none",
          zIndex: 20,
          background: "white",
          opacity: 0,
        }}
      />

      {/* Black void */}
      <div
        ref={voidRef}
        style={{
          position: "absolute",
          inset: 0,
          pointerEvents: "none",
          zIndex: 25,
          background: "black",
          opacity: 0,
        }}
      />
    </>
  );
}
