"use client";

import { useState, useEffect } from "react";
import { WebGLCanvas } from "@/components/canvas/WebGLCanvas";
import ErrorBoundary from "@/components/debug/ErrorBoundary";
import { useCamera } from "@/hooks/useCamera";
import { useMobileOptimization } from "@/hooks/useMobileOptimization";
import { type SimulationParams, DEFAULT_PARAMS } from "@/types/simulation";
import { DEFAULT_FEATURES, type PresetName } from "@/types/features";

const App = () => {
  const { isMobile, getMobileFeatures } = useMobileOptimization();

  const [params, setParams] = useState<SimulationParams>(() => {
    let initialFeatures = DEFAULT_FEATURES;
    let initialPreset: PresetName = "ultra-quality";
    if (isMobile) {
      initialFeatures = getMobileFeatures();
      initialPreset = "balanced";
    }
    return {
      ...DEFAULT_PARAMS,
      quality: initialFeatures.rayTracingQuality,
      features: initialFeatures,
      performancePreset: initialPreset,
      adaptiveResolution: false,
    };
  });

  const {
    mouse,
    handleMouseDown,
    handleMouseMove,
    handleMouseUp,
    handleWheel,
    handleTouchStart,
    handleTouchMove,
    handleTouchEnd,
  } = useCamera(params, setParams);

  useEffect(() => {
    // eslint-disable-next-line @typescript-eslint/no-var-requires
    const { physicsBridge } = require("@/engine/physics-bridge");
    // eslint-disable-next-line no-console
    physicsBridge.ensureInitialized().catch(console.error);
  }, []);

  return (
    <div
      style={{
        position: "fixed",
        inset: 0,
        background: "black",
        overflow: "hidden",
        touchAction: "none",
        userSelect: "none",
      }}
    >
      <ErrorBoundary>
        <WebGLCanvas
          params={params}
          mouse={mouse}
          onMouseDown={handleMouseDown}
          onMouseMove={handleMouseMove}
          onMouseUp={handleMouseUp}
          onWheel={handleWheel}
          onTouchStart={handleTouchStart}
          onTouchMove={handleTouchMove}
          onTouchEnd={handleTouchEnd}
        />
      </ErrorBoundary>
    </div>
  );
};

export default App;
