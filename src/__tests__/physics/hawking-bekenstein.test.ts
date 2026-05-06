import { describe, it, expect } from "vitest";
import {
  hawkingTemperatureKelvin,
  wienPeakFrequency,
  hawkingSpectrumPlanck,
  sampleHawkingSpectrum,
} from "@/lib/physics/hawking";
import {
  horizonAreaGeometric,
  horizonAreaSi,
  bekensteinHawkingEntropyPerKb,
  schwarzschildMassLossRate,
  schwarzschildEvaporationTimeSeconds,
  formatScientific,
} from "@/lib/physics/bekenstein";

const SI_SOLAR_MASS = 1.98847e30;

describe("Hawking primitives", () => {
  it("solar-mass Schwarzschild T_H is in the picokelvin range", () => {
    const t = hawkingTemperatureKelvin(SI_SOLAR_MASS, 0);
    expect(t).toBeGreaterThan(1e-9);
    expect(t).toBeLessThan(1e-6);
  });

  it("primordial-mass Schwarzschild T_H matches Hawking 1975 order of magnitude", () => {
    const t = hawkingTemperatureKelvin(1e12, 0);
    expect(t).toBeGreaterThan(1e10);
    expect(t).toBeLessThan(1e12);
  });

  it("extremal Kerr T_H collapses to zero", () => {
    const t = hawkingTemperatureKelvin(SI_SOLAR_MASS, 1);
    expect(t).toBeCloseTo(0, 30);
  });

  it("clamps spin outside [-1, 1] to the boundary", () => {
    const t_pos = hawkingTemperatureKelvin(SI_SOLAR_MASS, 1.5);
    const t_neg = hawkingTemperatureKelvin(SI_SOLAR_MASS, -1.5);
    expect(t_pos).toBeCloseTo(0, 30);
    expect(t_neg).toBeCloseTo(0, 30);
  });

  it("Wien peak frequency scales linearly with T", () => {
    expect(wienPeakFrequency(200) / wienPeakFrequency(100)).toBeCloseTo(2, 12);
  });

  it("Wien peak is zero at zero or negative temperature", () => {
    expect(wienPeakFrequency(0)).toBe(0);
    expect(wienPeakFrequency(-100)).toBe(0);
  });

  it("Planck spectrum returns zero on degenerate inputs", () => {
    expect(hawkingSpectrumPlanck(0, 100)).toBe(0);
    expect(hawkingSpectrumPlanck(1e10, 0)).toBe(0);
    expect(hawkingSpectrumPlanck(-1, 100)).toBe(0);
  });

  it("Planck spectrum peaks near the Wien frequency", () => {
    const t = 100;
    const peak = wienPeakFrequency(t);
    const bPeak = hawkingSpectrumPlanck(peak, t);
    const bLow = hawkingSpectrumPlanck(peak * 0.1, t);
    const bHigh = hawkingSpectrumPlanck(peak * 10, t);
    expect(bPeak).toBeGreaterThan(bLow);
    expect(bPeak).toBeGreaterThan(bHigh);
  });

  it("Planck overflow guard returns zero at h ν / k T > 700", () => {
    const b = hawkingSpectrumPlanck(1e16, 1);
    expect(b).toBe(0);
    expect(Number.isFinite(b)).toBe(true);
  });

  it("sampleHawkingSpectrum returns the requested grid size", () => {
    const grid = sampleHawkingSpectrum(100, 3, 32);
    expect(grid.length).toBe(32);
    for (const point of grid) {
      expect(point.freqHz).toBeGreaterThan(0);
      expect(point.radiance).toBeGreaterThanOrEqual(0);
    }
  });

  it("sampleHawkingSpectrum returns empty array on degenerate temperature", () => {
    expect(sampleHawkingSpectrum(0)).toEqual([]);
    expect(sampleHawkingSpectrum(-1)).toEqual([]);
  });

  it("sampleHawkingSpectrum centres the grid on the Wien peak", () => {
    // With 3 samples spanning ±0 decades, all three sit at the peak.
    // With 3 samples spanning ±1 decade, the middle sample (index 1)
    // is at the peak frequency.
    const t = 100;
    const grid = sampleHawkingSpectrum(t, 1, 3);
    expect(grid).toHaveLength(3);
    const peak = wienPeakFrequency(t);
    const middle = grid[1];
    expect(middle).toBeDefined();
    expect(middle?.freqHz ?? 0).toBeCloseTo(peak, 0);
  });
});

describe("Bekenstein-Hawking primitives", () => {
  it("Schwarzschild horizon area is 16π M² in geometric units", () => {
    expect(horizonAreaGeometric(0)).toBeCloseTo(16 * Math.PI, 12);
  });

  it("Extremal Kerr horizon area is 8π M²", () => {
    expect(horizonAreaGeometric(1)).toBeCloseTo(8 * Math.PI, 12);
    expect(horizonAreaGeometric(-1)).toBeCloseTo(8 * Math.PI, 12);
  });

  it("Horizon area decreases as |a*| grows from 0 to 1", () => {
    const a0 = horizonAreaGeometric(0);
    const aHigh = horizonAreaGeometric(0.998);
    expect(aHigh).toBeLessThan(a0);
  });

  it("SI horizon area scales with M²", () => {
    const a1 = horizonAreaSi(1e30, 0);
    const a2 = horizonAreaSi(2e30, 0);
    expect(a2 / a1).toBeCloseTo(4, 9);
  });

  it("Solar-mass Schwarzschild S/k_B is on the order of 1e77", () => {
    const s = bekensteinHawkingEntropyPerKb(SI_SOLAR_MASS, 0);
    expect(s).toBeGreaterThan(1e76);
    expect(s).toBeLessThan(1e78);
  });

  it("Mass-loss rate is negative and scales as 1/M²", () => {
    const dm1 = schwarzschildMassLossRate(1e10);
    const dm2 = schwarzschildMassLossRate(2e10);
    expect(dm1).toBeLessThan(0);
    expect(dm2 / dm1).toBeCloseTo(0.25, 9);
  });

  it("Mass-loss rate is zero on non-positive mass", () => {
    expect(schwarzschildMassLossRate(0)).toBe(0);
    expect(schwarzschildMassLossRate(-1)).toBe(0);
  });

  it("Evaporation time scales as M³", () => {
    const t1 = schwarzschildEvaporationTimeSeconds(1e10);
    const t2 = schwarzschildEvaporationTimeSeconds(2e10);
    expect(t2 / t1).toBeCloseTo(8, 9);
  });

  it("Evaporation time is infinite for non-positive mass", () => {
    expect(schwarzschildEvaporationTimeSeconds(0)).toBe(Infinity);
    expect(schwarzschildEvaporationTimeSeconds(-1)).toBe(Infinity);
  });

  it("Solar-mass evaporation is many orders past the universe age", () => {
    const t = schwarzschildEvaporationTimeSeconds(SI_SOLAR_MASS);
    const ageUniverseSec = 1.4e10 * 365.25 * 24 * 3600;
    const ratio = t / ageUniverseSec;
    expect(ratio).toBeGreaterThan(1e50);
    expect(ratio).toBeLessThan(1e65);
  });

  it("formatScientific switches between fixed and scientific notation", () => {
    expect(formatScientific(0)).toBe("0");
    expect(formatScientific(1.234)).toBe("1.234");
    expect(formatScientific(1.234e10)).toMatch(/^1\.23e\+10$/);
    expect(formatScientific(NaN)).toBe("—");
    expect(formatScientific(Infinity)).toBe("—");
  });
});
