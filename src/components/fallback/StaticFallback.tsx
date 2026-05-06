// Last-resort renderer fallback. Surfaces when both WebGPU and WebGL 2.0
// are unavailable on the user's device. The render-path negotiator selects
// this component instead of a blank canvas so visitors land on something
// readable rather than a runtime error.

export function StaticFallback() {
  return (
    <div className="fixed inset-0 flex flex-col items-center justify-center bg-black text-white p-8">
      <div className="max-w-prose text-center">
        <h1 className="text-2xl font-light tracking-[0.2em] uppercase mb-4">
          Simulation requires hardware-accelerated graphics
        </h1>
        <p className="text-sm text-zinc-400 leading-relaxed mb-6">
          This simulation traces relativistic light rays in real time and needs
          either WebGPU or WebGL 2.0. Your browser does not currently expose
          either to this page.
        </p>
        <p className="text-sm text-zinc-500 leading-relaxed">
          Try the latest desktop{" "}
          <a
            className="underline hover:text-white"
            href="https://www.google.com/chrome/"
            rel="noopener noreferrer"
            target="_blank"
          >
            Chrome
          </a>{" "}
          or{" "}
          <a
            className="underline hover:text-white"
            href="https://www.mozilla.org/firefox/"
            rel="noopener noreferrer"
            target="_blank"
          >
            Firefox
          </a>
          . On macOS, Safari 17+ exposes WebGPU when enabled in Develop {">"}{" "}
          Feature Flags.
        </p>
      </div>
    </div>
  );
}
