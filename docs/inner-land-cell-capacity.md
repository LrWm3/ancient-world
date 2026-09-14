# Inner-continent land-cell capacity

Counted seeds 17, 81 and 256 at the lowest and highest production terrain grids
(256 and 1024 cells per cube face), using default planet settings and the Quadro
RTX 5000 with Max-Q Design. Source base: `118bc9c` plus the counter added here.

This is the hypothetical one-settlement-per-cell geographic upper bound, **not**
the current settlement candidate count. Each run uses `Generator::new`, including
GPU initialization and coast cleanup, then counts terrain cells with `meta[0] == 2`.
No epochs, civilization history, suitability survey, spacing, resource constraints,
or 2,560-record limit are applied. Later inland lakes and flooding are not excluded.
The count describes the inner-continent geographic mask, not guaranteed dry sites.

| Seed | Face resolution | Planet cells | Inner-land cells | Inner-land area (km²) |
| --- | ---: | ---: | ---: | ---: |
| 17 | 256 | 393,216 | 3,677 | 7,206,959 |
| 81 | 256 | 393,216 | 3,475 | 7,224,955 |
| 256 | 256 | 393,216 | 3,647 | 7,131,037 |
| 17 | 1024 | 6,291,456 | 58,843 | 7,212,246 |
| 81 | 1024 | 6,291,456 | 55,589 | 7,221,726 |
| 256 | 1024 | 6,291,456 | 58,460 | 7,142,398 |

Area is summed with cube-sphere solid angles and the configured radius; it is not
estimated by assuming every cell has the same area. Within each seed, area differs
by less than 0.2% between these resolutions. Counts increase approximately sixteenfold
because each face edge is four times finer. Increasing resolution therefore creates
more hypothetical cell-sized sites without meaningfully increasing available land.
The actual simulation still enforces its settlement cap and founding requirements.

Reproduce:

```sh
cargo run --example inner_land_count -- --seeds 17,81,256 --resolutions 256,1024
```

All six runs completed. Counts were checked against `6 × resolution²` and were
positive and below the full grid size. Run time totaled approximately 9.78 seconds
excluding compilation; the first run includes additional initialization overhead.
Raw CSV and build output remain under ignored `output/`; only this summary and
source are committed.
