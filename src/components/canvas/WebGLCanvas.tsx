"use client";

import { useRef, useEffect, useState, useCallback, useMemo } from "react";
import { WebGLRenderer } from "@/rendering/webgl/renderer";
import { AlertCircle } from "lucide-react";
import type { SimulationParams, MouseState } from "@/types/simulation";
import type { PerformanceMetrics } from "@/performance/monitor";
import { useDeviceTier } from "@/hooks/useDeviceTier";
import { useOblivionSequence } from "@/hooks/useOblivionSequence";
import { OblivionOverlay } from "@/components/OblivionOverlay";
import { EntryHint } from "@/components/EntryHint";

interface CanvasError {
  type: "context" | "shader" | "program" | "memory";
  message: string;
  details?: string;
}

interface WebGLCanvasProps {
  params: SimulationParams;
  mouse: MouseState;
  onMouseDown: (e: React.MouseEvent) => void;
  onMouseMove: (e: React.MouseEvent) => void;
  onMouseUp: (e: React.MouseEvent) => void;
  onWheel: (e: React.WheelEvent | WheelEvent) => void;
  onTouchStart: (e: React.TouchEvent | TouchEvent) => void;
  onTouchMove: (e: React.TouchEvent | TouchEvent) => void;
  onTouchEnd: (e: React.TouchEvent | TouchEvent) => void;
  onMetricsUpdate?: (metrics: PerformanceMetrics) => void;
  onPhaseChange?: (phase: string) => void;
}

export const WebGLCanvas = ({
  params,
  mouse,
  onMouseDown,
  onMouseMove,
  onMouseUp,
  onWheel,
  onTouchStart,
  onTouchMove,
  onTouchEnd,
  onMetricsUpdate,
  onPhaseChange,
}: WebGLCanvasProps) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const rendererRef = useRef<WebGLRenderer | null>(null);
  const mouseRef = useRef(mouse);
  const rafRef = useRef<number>(0);
  const [error, setError] = useState<CanvasError | null>(null);

  const tier = useDeviceTier();

  // Oblivion: pass current zoom so it knows where to start falling from
  const { phase, overrides, triggerEntry } = useOblivionSequence(
    params.zoom,
    params.mass,
  );

  // Merge overrides into params — zoom + mass animate during the fall
  const effectiveParams = useMemo(() => {
    if (!overrides.zoom && !overrides.mass) return params;
    return {
      ...params,
      ...(overrides.zoom != null && { zoom: overrides.zoom }),
      ...(overrides.mass != null && { mass: overrides.mass }),
    };
  }, [params, overrides]);

  const paramsRef = useRef(effectiveParams);
  useEffect(() => {
    paramsRef.current = effectiveParams;
  }, [effectiveParams]);
  useEffect(() => {
    mouseRef.current = mouse;
  }, [mouse]);

  // Notify parent (page.tsx hides UI during sequence)
  useEffect(() => {
    onPhaseChange?.(phase);
  }, [phase, onPhaseChange]);

  // ── Canvas sizing — DPR clamped per device tier ──
  const updateSize = useCallback(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const maxDpr = tier === "LOW" ? 1 : tier === "MEDIUM" ? 1.5 : 2;
    const dpr = Math.min(window.devicePixelRatio || 1, maxDpr);
    const w = Math.floor(window.innerWidth * dpr);
    const h = Math.floor(window.innerHeight * dpr);
    if (canvas.width !== w || canvas.height !== h) {
      canvas.width = w;
      canvas.height = h;
      canvas.style.width = "100%";
      canvas.style.height = "100%";
      rendererRef.current?.resize(w, h);
    }
  }, [tier]);

  // ── Renderer init ──
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas || rendererRef.current) return;
    updateSize();
    const renderer = new WebGLRenderer();
    renderer.onMetricsUpdate = onMetricsUpdate;
    if (renderer.init(canvas)) {
      rendererRef.current = renderer;
    } else {
      setError(
        renderer.error || {
          type: "context",
          message: "WebGL failed to initialize.",
        },
      );
    }
    return () => {
      cancelAnimationFrame(rafRef.current);
      rendererRef.current?.cleanup();
      rendererRef.current = null;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // ── Render loop — runs continuously, pauses only during VOID ──
  useEffect(() => {
    cancelAnimationFrame(rafRef.current);
    if (phase === "VOID") return; // black screen, no GPU work needed

    const minInterval = tier === "LOW" ? 1000 / 30 : 0;
    let last = 0;
    const loop = (ts: number) => {
      if (ts - last >= minInterval) {
        last = ts;
        try {
          rendererRef.current?.render(paramsRef.current, mouseRef.current);
        } catch (e) {
          const err = e as Error;
          setError({
            type: "shader",
            message: err.message,
            details: err.stack,
          });
          return;
        }
      }
      rafRef.current = requestAnimationFrame(loop);
    };
    rafRef.current = requestAnimationFrame(loop);
    return () => cancelAnimationFrame(rafRef.current);
  }, [phase, tier]);

  // ── Resize + orientation ──
  useEffect(() => {
    let debounce: ReturnType<typeof setTimeout>;
    const onResize = () => updateSize();
    const onOrient = () => {
      clearTimeout(debounce);
      debounce = setTimeout(updateSize, 150);
    };
    window.addEventListener("resize", onResize);
    window.addEventListener("orientationchange", onOrient);
    updateSize();
    return () => {
      window.removeEventListener("resize", onResize);
      window.removeEventListener("orientationchange", onOrient);
      clearTimeout(debounce);
    };
  }, [updateSize]);

  // ── Native touch/wheel ──
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const opts: AddEventListenerOptions = { passive: false };
    const wh = (e: WheelEvent) => {
      if (phase === "IDLE") onWheel(e);
    };
    const ts = (e: TouchEvent) => {
      if (phase !== "IDLE") {
        e.preventDefault();
        return;
      }
      onTouchStart(e);
    };
    const tm = (e: TouchEvent) => {
      if (phase !== "IDLE") {
        e.preventDefault();
        return;
      }
      onTouchMove(e);
    };
    const te = (e: TouchEvent) => {
      if (phase === "IDLE") onTouchEnd(e);
    };
    canvas.addEventListener("wheel", wh, opts);
    canvas.addEventListener("touchstart", ts, opts);
    canvas.addEventListener("touchmove", tm, opts);
    canvas.addEventListener("touchend", te, opts);
    return () => {
      canvas.removeEventListener("wheel", wh);
      canvas.removeEventListener("touchstart", ts);
      canvas.removeEventListener("touchmove", tm);
      canvas.removeEventListener("touchend", te);
    };
  }, [onWheel, onTouchStart, onTouchMove, onTouchEnd, phase]);

  // ── Click/tap the black center to trigger the fall ──
  const handlePointerDown = useCallback(
    (e: React.PointerEvent) => {
      if (phase !== "IDLE") return;
      const canvas = canvasRef.current;
      if (!canvas) return;

      const rect = canvas.getBoundingClientRect();
      const cx = rect.left + rect.width / 2;
      const cy = rect.top + rect.height / 2;
      const dx = e.clientX - cx;
      const dy = e.clientY - cy;
      const dist = Math.sqrt(dx * dx + dy * dy);
      const hitRadius = Math.min(rect.width, rect.height) * 0.08;

      if (dist <= hitRadius) {
        e.preventDefault();
        triggerEntry();
      } else {
        onMouseDown(e as unknown as React.MouseEvent);
      }
    },
    [phase, triggerEntry, onMouseDown],
  );

  const horizonRadius = (() => {
    const canvas = canvasRef.current;
    if (!canvas) return 40;
    const r = canvas.getBoundingClientRect();
    return Math.min(r.width, r.height) * 0.08;
  })();

  return (
    <div
      style={{
        position: "relative",
        width: "100vw",
        height: "100dvh",
        overflow: "hidden",
        background: "black",
      }}
    >
      {/* Canvas — no CSS transform, GPU handles everything */}
      <canvas
        ref={canvasRef}
        style={{
          position: "absolute",
          inset: 0,
          display: "block",
          width: "100%",
          height: "100%",
          touchAction: "none",
          cursor: phase === "IDLE" ? "crosshair" : "none",
        }}
        onMouseMove={phase === "IDLE" ? onMouseMove : undefined}
        onMouseUp={phase === "IDLE" ? onMouseUp : undefined}
      />

      {/* Hit capture layer */}
      <div
        style={{
          position: "absolute",
          inset: 0,
          zIndex: 5,
          cursor: phase === "IDLE" ? "crosshair" : "none",
        }}
        onPointerDown={handlePointerDown}
        onPointerMove={
          phase === "IDLE"
            ? (e) => onMouseMove(e as unknown as React.MouseEvent)
            : undefined
        }
        onPointerUp={
          phase === "IDLE"
            ? (e) => onMouseUp(e as unknown as React.MouseEvent)
            : undefined
        }
      />

      {/* Minimal overlays — only flash + void */}
      <OblivionOverlay phase={phase} />

      {/* Hint ring for idle users */}
      {phase === "IDLE" && (
        <EntryHint isMobile={tier !== "HIGH"} horizonRadius={horizonRadius} />
      )}

      {/* Error */}
      {error && (
        <div
          style={{
            position: "absolute",
            inset: 0,
            zIndex: 50,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            background: "rgba(0,0,0,0.92)",
          }}
        >
          <div
            style={{
              maxWidth: 540,
              margin: 16,
              padding: 24,
              background: "rgba(100,0,0,0.4)",
              border: "1px solid rgba(255,80,80,0.3)",
              borderRadius: 12,
            }}
          >
            <div style={{ display: "flex", gap: 12 }}>
              <AlertCircle
                size={20}
                style={{ color: "#f87171", flexShrink: 0, marginTop: 2 }}
              />
              <div>
                <p
                  style={{ color: "#fca5a5", fontWeight: 700, marginBottom: 8 }}
                >
                  {error.type === "context"
                    ? "WebGL Not Available"
                    : "Render Error"}
                </p>
                <p style={{ color: "#d1d5db", fontSize: 13 }}>
                  {error.message}
                </p>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
