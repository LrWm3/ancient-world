# Ecological sunlight

Ecological production previously used a latitude attenuation multiplied by the
same January-peaking cosine everywhere. This did not reverse seasons across
hemispheres and did not account for day length or axial tilt.

`shaders/sunlight.wgsl` now supplies the daily mean positive solar zenith cosine.
For latitude φ, declination δ and sunset hour angle H:

- sin δ = sin(tilt) sin(2π(month − 2)/12)
- A = sin φ sin δ; B = cos φ cos δ
- H = acos(−A/B) where the sun rises and sets
- ecological light = A H + B sin H

Polar night returns zero. Continuous daylight returns π max(A, 0).
Explicit branches handle the poles and 90° obliquity without division by zero.
This is the daily mean normalized by equatorial equinox irradiance (1/π).
`solar_scale` multiplies that dimensionless input; it is **not W/m²** or an
independently calibrated carbon flux. Polar summer values can exceed one.

The physical mechanism follows daily-mean solar geometry described in the
[climlab insolation documentation](https://climlab.readthedocs.io/en/stable/api/climlab.solar.insolation.html).
Our implementation uses a circular orbit and twelve equal months, with
representative March/September equinox and June/December solstice days. It does
not integrate over each month or simulate eccentricity, clouds, atmospheric
radiative transfer or terrain shading. Existing producer growth efficiencies,
vertical light attenuation and resource limits remain game-model parameters.

Both terrestrial photosynthesis and aquatic production use this input. Normal
ecology and living-history updates use their existing month counters (January
is index zero). Monthly weather uses the same declination phase, scaled relative
to Earth's default tilt to retain its existing 12-degree temperature coefficient.
The geological climate solver remains a separate approximate seasonal model.
No saved field layouts change; existing worlds receive the corrected forcing on
continuation, so continuation across this code change is not bitwise compatible
with the old sunlight model.

## Verification (2026-09-10)

On the available Quadro RTX 5000 Vulkan backend:

- 7,236 GPU samples across latitude, all months and tilts 0°, 23.44°, 90°
  matched independent numerical integration of a rotating surface normal.
  Maximum absolute error: 0.000000548; acceptance tolerance: 0.00002.
- Checks cover opposite-hemisphere half-year symmetry, January/July reversal,
  zero-tilt invariance, polar night/day, equatorial equinox maxima and spherical
  mean irradiance π/4 (normalized units).
- Controlled aquatic production with identical habitat and inventories reverses
  January/July growth across hemispheres, stops photosynthesis under zero light
  and polar night, and agrees between ecological and living-history clocks.
  The existing darkness/phosphorus-limit integration test also passed.
- Existing coarse-grid conservation, scenarios and checkpoint continuation test
  passed. This verifies continuation within the new implementation.

Reproduce with `mise exec rust@1.89.0 -- cargo test --test sunlight -- --ignored
--nocapture` and `cargo test --test ecology
budgets_coarse_grid_scenarios_and_continuation -- --ignored` through the same
Rust toolchain. The production fixture is `cargo test --lib
seasonal_light_reverses_actual_aquatic_production -- --ignored`. These are explicit hardware tests, not part of default CPU CI.

Annual and regional productivity will change, particularly at high latitudes.
No long-history ecosystem recalibration is claimed by this correction.
