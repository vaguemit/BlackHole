import { describe, it, expect } from "vitest";
import * as fc from "fast-check";
import {
  OFFSETS,
  BLOCK_FLOATS,
  TELEMETRY_SLOTS,
  SHADOW_CURVE_MAX_POINTS,
} from "@/engine/sab-schema";

describe("sab-schema offset arithmetic invariants", () => {
  it("blocks are monotonically ordered", () => {
    expect(OFFSETS.CONTROL).toBeLessThan(OFFSETS.CAMERA);
    expect(OFFSETS.CAMERA).toBeLessThan(OFFSETS.PHYSICS);
    expect(OFFSETS.PHYSICS).toBeLessThan(OFFSETS.TELEMETRY);
    expect(OFFSETS.TELEMETRY).toBeLessThan(OFFSETS.LUTS);
  });

  it("all f32 offsets are non-negative integers", () => {
    for (const v of Object.values(OFFSETS)) {
      expect(Number.isInteger(v)).toBe(true);
      expect(v).toBeGreaterThanOrEqual(0);
    }
  });

  it("block floats sum equals LUTS offset", () => {
    const sum =
      BLOCK_FLOATS.CONTROL +
      BLOCK_FLOATS.CAMERA +
      BLOCK_FLOATS.PHYSICS +
      BLOCK_FLOATS.TELEMETRY;
    expect(sum).toBe(OFFSETS.LUTS);
  });

  it("TELEMETRY slots stay inside the TELEMETRY block", () => {
    for (const slot of Object.values(TELEMETRY_SLOTS)) {
      expect(slot).toBeGreaterThanOrEqual(0);
      expect(slot).toBeLessThan(BLOCK_FLOATS.TELEMETRY);
    }
  });

  it("shadow curve cap fits inside PHYSICS minus the 16-slot scalar header", () => {
    const SCALAR_HEADER_FLOATS = 16;
    const curveFloats = SHADOW_CURVE_MAX_POINTS * 2;
    expect(SCALAR_HEADER_FLOATS + curveFloats).toBeLessThanOrEqual(
      BLOCK_FLOATS.PHYSICS,
    );
  });

  it("byte offsets round-trip through f32 indices", () => {
    fc.assert(
      fc.property(
        fc.constantFrom(...Object.values(OFFSETS)),
        (floatIndex: number) => {
          const byteOffset = floatIndex * 4;
          expect(byteOffset % 4).toBe(0);
          expect(byteOffset / 4).toBe(floatIndex);
        },
      ),
    );
  });
});
