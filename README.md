# BlackHole

A real-time black hole simulation running in your browser. Built with WebGL 2.0 and ray-marched GLSL shaders.

**Tap or click the black center to fall in.**

---

## What it does

Renders a rotating Kerr black hole using a custom ray-marching engine written in GLSL ES 3.0. Every pixel is a ray traced through curved spacetime.

- **Gravitational lensing** — light bends around the event horizon
- **Accretion disk** — physically-based temperature gradient with relativistic Doppler beaming
- **Photon ring** — light that orbits the black hole multiple times
- **Oblivion entry** — tap the center, mass grows and observer distance shrinks, the disk wraps around you, singularity flash, then void

---

## Run locally

**Requires:** Node.js 18+

```bash
git clone https://github.com/vaguemit/BlackHole.git
cd BlackHole
npm install
npm run dev:next
```

Open [http://localhost:3000](http://localhost:3000)

---

## Controls

| Input              | Action                   |
| ------------------ | ------------------------ |
| Drag / Swipe       | Orbit camera             |
| Scroll / Pinch     | Zoom                     |
| Click / Tap center | Fall into the black hole |

---

## Stack

- **Next.js 14** — framework
- **WebGL 2.0 + GLSL ES 3.0** — real-time rendering
- **TypeScript** — language
- **Bun** — runtime

---

## License

MIT
