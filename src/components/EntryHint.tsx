"use client";

import { useEffect, useState } from "react";

interface EntryHintProps {
  isMobile: boolean;
  horizonRadius: number; // pixels — radius of hint ring
}

/**
 * EntryHint — PRD §4.3
 *
 * Idle pulse ring shown after 8s of inactivity. Disappears on any pointer
 * interaction. On mobile shows a small "↓ TAP" label below the ring.
 */
export function EntryHint({ isMobile, horizonRadius }: EntryHintProps) {
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    // Show hint after 8s of inactivity
    const timer = setTimeout(() => setVisible(true), 8000);

    const hideHint = () => {
      setVisible(false);
      clearTimeout(timer);
    };

    window.addEventListener("pointerdown", hideHint, { once: true });

    return () => {
      clearTimeout(timer);
      window.removeEventListener("pointerdown", hideHint);
    };
  }, []);

  if (!visible || horizonRadius <= 0) return null;

  const ringSize = horizonRadius * 2 + 8; // px, slight overshoot of the horizon

  return (
    <div
      style={{
        position: "absolute",
        top: "50%",
        left: "50%",
        transform: "translate(-50%, -50%)",
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        pointerEvents: "none",
        zIndex: 20,
      }}
    >
      {/* Pulse ring */}
      <div
        style={{
          width: ringSize,
          height: ringSize,
          borderRadius: "50%",
          border: "1.5px solid rgba(255, 200, 80, 0.25)",
          animation: "horizon-pulse 3s ease-in-out infinite",
          boxShadow: "0 0 12px rgba(255, 200, 80, 0.08)",
        }}
      />

      {/* Mobile TAP label */}
      {isMobile && (
        <span
          style={{
            marginTop: 10,
            fontFamily: "monospace",
            fontSize: 11,
            color: "rgba(255, 200, 80, 0.4)",
            letterSpacing: "0.15em",
            userSelect: "none",
          }}
        >
          ↓ TAP
        </span>
      )}

      <style>{`
        @keyframes horizon-pulse {
          0%   { opacity: 0.0; transform: scale(1.0); }
          50%  { opacity: 0.3; transform: scale(1.08); }
          100% { opacity: 0.0; transform: scale(1.0); }
        }
      `}</style>
    </div>
  );
}
