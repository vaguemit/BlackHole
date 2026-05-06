/**
 * useDeviceTier — PRD §3
 *
 * Detects device capability tier on mount. Used to select render quality,
 * resolution scale, and ray march step count.
 *
 * Tier precedence: any single LOW vote → LOW, else any MEDIUM → MEDIUM, else HIGH.
 */

"use client";

import { useState, useEffect } from "react";

export type DeviceTier = "HIGH" | "MEDIUM" | "LOW";

// Extend NavigatorNetworkInformation for TypeScript
interface NetworkInformation {
  saveData?: boolean;
  effectiveType?: "slow-2g" | "2g" | "3g" | "4g";
}

interface NavigatorWithNetwork extends Navigator {
  deviceMemory?: number;
  connection?: NetworkInformation;
}

export function useDeviceTier(): DeviceTier {
  const [tier, setTier] = useState<DeviceTier>("HIGH");

  useEffect(() => {
    const nav = navigator as NavigatorWithNetwork;
    const votes: DeviceTier[] = [];

    // 1. WebGPU check — only candidate for HIGH if available
    const hasWebGPU = "gpu" in navigator;
    if (!hasWebGPU) votes.push("MEDIUM");

    // 2. Hardware concurrency
    const cores = navigator.hardwareConcurrency ?? 4;
    if (cores >= 8) votes.push("HIGH");
    else if (cores >= 4) votes.push("MEDIUM");
    else votes.push("LOW");

    // 3. Device memory (GB)
    const memory = nav.deviceMemory ?? 4;
    if (memory >= 4) votes.push("HIGH");
    else if (memory >= 2) votes.push("MEDIUM");
    else votes.push("LOW");

    // 4. Mobile UA — cap at MEDIUM
    const isMobile =
      /Android|iPhone|iPad|iPod|Opera Mini|IEMobile|WPDesktop/i.test(
        navigator.userAgent,
      );
    if (isMobile) votes.push("MEDIUM");

    // 5. Network — saveData or 2G → LOW
    const connection = nav.connection;
    if (
      connection &&
      (connection.saveData ||
        connection.effectiveType === "2g" ||
        connection.effectiveType === "slow-2g")
    ) {
      votes.push("LOW");
    }

    // Resolve: take the lowest (worst) tier from all votes
    if (votes.includes("LOW")) {
      setTier("LOW");
    } else if (votes.includes("MEDIUM")) {
      setTier("MEDIUM");
    } else {
      setTier("HIGH");
    }
  }, []);

  return tier;
}
