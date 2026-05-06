import { PHYSICS_CONSTANTS } from "@/configs/physics.config";

export const DISK_CHUNK = `
  // Accretion Disk Physics & Rendering
  // Inputs:
  //   p: current ray position (vec3)
  //   ro: ray origin (vec3)
  //   r: current radius from BH center
  //   isco: innermost stable circular orbit
  //   M: black hole mass
  //   a: black hole spin parameter
  //   dt: integration step size (for density integration)
  //   accumulatedColor: (inout)
  //   accumulatedAlpha: (inout)

  void sample_accretion_disk(
      vec3 p, vec3 p_prev, vec3 ro, vec3 v, float r, float isco, float M, float a, float dt, float rs,
      inout vec3 accumulatedColor, inout float accumulatedAlpha
  ) {
      if (u_show_redshift < 0.5) {
          // Plane-Crossing Detection (eliminates holes from step-skipping)
          bool crossedEquator = (p_prev.y * p.y < 0.0);
          
          vec3 sampleP = p;
          if (crossedEquator) {
              float t = abs(p_prev.y) / max(0.0001, abs(p_prev.y) + abs(p.y));
              sampleP = mix(p_prev, p, t);
          }
          
          float sampleR = length(sampleP);
          
          float effectiveScaleHeight = min(u_disk_scale_height, ${PHYSICS_CONSTANTS.accretion.diskHeightMultiplier.toFixed(3)});
          // PRD §2.2: thickness h(r) = 0.08*r*exp(-0.3*(r-isco)) — thick inner, tapers outward
          float hProfile = 0.08 * sampleR * exp(-0.3 * max(0.0, sampleR - isco));
          float diskHeight = max(sampleR * effectiveScaleHeight, hProfile);

          float diskInner = isco;
          // PRD §2.2: outer edge = 4.5 * rs (Schwarzschild radius rs = 2*M)
          float diskOuter = max(M * u_disk_size, max(diskInner * 1.1, rs * 4.5));

          if((abs(sampleP.y) < diskHeight || crossedEquator) && sampleR > diskInner && sampleR < diskOuter) {

              float sqrt_M_phase = sqrt(M);
              float signSpinPhase = sign(u_spin + 1e-8);
              float OmegaPhase = (signSpinPhase * sqrt_M_phase) / (sampleR * sqrt(sampleR) + a * sqrt_M_phase);
              float rotAngle = OmegaPhase * u_time * ${PHYSICS_CONSTANTS.accretion.timeScale.toFixed(2)} * 10.0;
              mat2 rotPhase = mat2(cos(rotAngle), -sin(rotAngle), sin(rotAngle), cos(rotAngle));
              
              vec3 noiseP = sampleP;
              noiseP.xz *= rotPhase;
              noiseP *= ${PHYSICS_CONSTANTS.accretion.turbulenceScale.toFixed(2)};

              float turbulence = noise(noiseP) * 0.5 + noise(noiseP * ${PHYSICS_CONSTANTS.accretion.turbulenceDetail.toFixed(1)}) * 0.25;

              float samplesDiskHeight = sampleR * effectiveScaleHeight;
              float heightFalloff = exp(-abs(sampleP.y) / max(0.001, samplesDiskHeight * ${PHYSICS_CONSTANTS.accretion.densityFalloff.toFixed(2)}));
              float radialFalloff = smoothstep(diskOuter, diskInner, sampleR);
              // Fade to alpha=0 at outer edge
              float edgeFade = smoothstep(diskOuter, diskOuter * 0.88, sampleR);

              float baseDensity = turbulence * heightFalloff * radialFalloff * edgeFade;

              if (baseDensity > 0.001) {
                  float r2 = sampleR * sampleR;
                  float sqrt_M = sqrt(M);
                  float signSpin = sign(u_spin + 1e-8);
                  float Omega = (signSpin * sqrt_M) / (sampleR * sqrt(sampleR) + a * sqrt_M);

                  float g_tt     = -(1.0 - 2.0 * M / sampleR);
                  float g_tphi   = -2.0 * M * a / sampleR;
                  float g_phiphi = r2 + a*a + 2.0 * M * a*a / sampleR;

                  float u_t_sq = -(g_tt + 2.0 * Omega * g_tphi + Omega * Omega * g_phiphi);
                  float u_t = 1.0 / sqrt(max(1e-6, u_t_sq));

                  float L_photon = p.z * v.x - p.x * v.z;
                  float delta = 1.0 / max(0.01, u_t * (1.0 - Omega * L_photon));

#ifdef ENABLE_DOPPLER
                  // PRD §2.2: Doppler beaming — approaching side (left) up, receding side dim
                  // delta encodes direction: >1 = approaching, <1 = receding
                  float beaming = max(0.01, pow(delta, 3.5));
#else
                  float beaming = 1.0;
#endif
                  float isco_r = clamp(isco / sampleR, 0.0, 1.0);
                  float nt_factor = max(0.0, 1.0 - sqrt(isco_r));
                  float radialTempGradient = pow(isco_r, 0.75) * pow(nt_factor, 0.25);
                  float temperature = u_disk_temp * radialTempGradient * delta;

                  // PRD §2.2: Gargantua temperature colour gradient
                  // inner → outer: #FFFFFF → #FFF5E0 → #FFD580 → #FF9500 → #FF6B1A → #8B1A00
                  float radialT = clamp((sampleR - diskInner) / max(1e-4, diskOuter - diskInner), 0.0, 1.0);
                  vec3 c0 = vec3(1.000, 0.961, 0.878); // white-hot
                  vec3 c1 = vec3(1.000, 0.835, 0.502); // amber
                  vec3 c2 = vec3(1.000, 0.584, 0.000); // gold-orange
                  vec3 c3 = vec3(1.000, 0.420, 0.102); // orange
                  vec3 c4 = vec3(0.545, 0.102, 0.000); // deep rust

                  vec3 gargantua;
                  if      (radialT < 0.25) gargantua = mix(c0, c1, radialT * 4.0);
                  else if (radialT < 0.50) gargantua = mix(c1, c2, (radialT - 0.25) * 4.0);
                  else if (radialT < 0.75) gargantua = mix(c2, c3, (radialT - 0.50) * 4.0);
                  else                     gargantua = mix(c3, c4, (radialT - 0.75) * 4.0);

                  // Blend physical blackbody with cinematic Gargantua palette
                  vec3 bbColor = blackbody(temperature);
                  vec3 diskColor = mix(bbColor, gargantua * max(0.001, length(bbColor)), 0.6) * beaming;

                  float density = baseDensity * u_disk_density * 0.12 * dt;

                  accumulatedColor += diskColor * density * (1.0 - accumulatedAlpha);
                  accumulatedAlpha += density;
              }
          }
      }
  }

  void sample_relativistic_jets(
      vec3 p, vec3 v, float r, float rh, float dt,
      inout vec3 accumulatedColor, inout float accumulatedAlpha
  ) {
      // Jets align with spin axis (Y-axis)
      float jetVerticalPos = abs(p.y);
      if (jetVerticalPos > rh * 1.8 && jetVerticalPos < MAX_DIST * 0.8) {
          float jetRadialDist = length(p.xz);
          float jetWidth = 1.0 + jetVerticalPos * 0.15;

          if (jetRadialDist < jetWidth * 2.0) {
              float radialFalloff = exp(-(jetRadialDist * jetRadialDist) / (jetWidth * 0.5));
              float lengthFalloff = exp(-jetVerticalPos * 0.05);

              float flowCombined = p.y * 2.0 - u_time * 8.0;
              vec3 uvJet = vec3(p.x, flowCombined, p.z);
              float noiseVal = noise(uvJet * 0.5) * 0.6 + noise(uvJet * 1.5) * 0.4;

              float jetDensity = radialFalloff * lengthFalloff * max(0.0, noiseVal - 0.2);

              if (jetDensity > 0.001) {
                  float jetVel = 0.92 * sign(p.y);
                  vec3 jetVelVec = vec3(0.0, jetVel, 0.0);

                  float cosThetaJet = dot(normalize(jetVelVec), -v);
                  float betaJet = abs(jetVel);
                  float gammaJet = 1.0 / sqrt(1.0 - betaJet * betaJet);
                  float deltaJet = 1.0 / (gammaJet * (1.0 - betaJet * cosThetaJet));
                  float beamingJet = pow(deltaJet, 3.5);

                  vec3 baseJetColor = vec3(0.4, 0.7, 1.0);
                  vec3 jetEmission = baseJetColor * jetDensity * 0.05 * beamingJet * dt;

                  accumulatedColor += jetEmission * (1.0 - accumulatedAlpha);
                  accumulatedAlpha += jetDensity * 0.05 * dt;
              }
          }
      }
  }
`;
