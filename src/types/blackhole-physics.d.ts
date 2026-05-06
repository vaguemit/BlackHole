// Type stub for the WASM physics module.
// The actual module is built from Rust/wasm-pack and loaded at runtime.
// In CI/Vercel builds without Rust, this stub satisfies TypeScript.
declare module "blackhole-physics" {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const init: (...args: any[]) => Promise<any>;
  export default init;

  export class PhysicsEngine {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    constructor(mass: number, spin: number): any;
    tick_sab(dt: number): void;
    update_params(mass: number, spin: number): void;
    set_camera_state(
      px: number,
      py: number,
      pz: number,
      lx: number,
      ly: number,
      lz: number,
    ): void;
    set_auto_spin(enabled: boolean): void;
    get_sab_ptr(): number;
    compute_horizon(): number;
    compute_isco(): number;
    compute_photon_sphere(): number;
    compute_dilation(r: number): number;
    compute_shadow_radius(): number;
    compute_shadow_curve(thetaObs: number, nPoints: number): Float32Array;
    compute_disk_flux(r: number): number;
    compute_g_factor(r: number, lambda: number): number;
    generate_disk_lut(): Float32Array;
    generate_spectrum_lut(
      width: number,
      height: number,
      maxTemp: number,
    ): Float32Array;
    generate_embedding_mesh(
      rMin: number,
      rMax: number,
      nRadial: number,
      nAngular: number,
    ): Float32Array;
    generate_ergosphere_mesh(nPolar: number, nAzimuthal: number): Float32Array;
    readonly memory: WebAssembly.Memory;
  }

  export function init_hooks(): void;
  export const memory: WebAssembly.Memory;
}
