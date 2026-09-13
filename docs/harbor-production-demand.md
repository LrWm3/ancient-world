# Harbor construction as production demand

Ports previously installed only whatever surplus timber, generic tools and bricks
happened to remain at the annual shipping update. Their missing materials did not
enter the ordinary production planner. In the three adult-payroll 50-year screens,
only 2/4, 0/5 and 1/5 ports had ever commissioned (seeds 1024, 256 and 409).
Several unfinished ports had no installed tools.

## Change and timing

Reserve-phase production planning now requests each existing surveyed harbor's
remaining materials, including the next annual wear allowance. It adds these to
ordinary demand while ensuring the working-stock floor used by construction is
also targeted. The floor is shared with the shipping module; it is not added a
second time if ordinary demand already covers it. No harbor requests are added
for abandoned settlements or when local trade policy is disabled.

Held goods, incoming cargo and funded procurement reduce duplicate recipe orders
through the existing planner. Targets enter ordinary production and trade;
workshop service procurement still requires its own finite customer funding.
Annual Respond-phase installation still consumes actual goods above the working
reserve and available leftover construction work. Requests are forecasts, not
escrowed materials or promised completion. Other consumers can still compete.

Commissioning still needs the full structure. A built harbor does not supply free
crew work, bypass closure/flood restrictions or create a reachable sea route.
The installed tool component still specifically requires generic tools; substituting
other tool materials requires preserving their actual embodied inventories.

## Verification

- CPU fixture: missing tools create recipe demand; expected material deliveries
  suppress duplicate recipes; prior working reserve is not requested twice;
  forecasting leaves physical harbor assets unchanged.
- GPU construction fixture: no work means no installation; bounded work installs
  bounded materials; stock plus structure balances; commissioning, degradation,
  recovery, repeated-boundary and serialized continuation checks pass.
- GPU automatic service-procurement continuation fixture passes.

## Matched 50-year screen

Baseline: commit 771cf26. Seeds 1024, 256 and 409, founding archives at terrain/ecology
32/32, zero additional geological epochs, 600 history months. Same controls as
[adult payroll](household-adult-payroll.md): delivery-paid exports, service-order
procurement (share 0.25), contract/demand workshop staffing, household inheritance,
named office service and abandoned-stock recovery enabled; estate reclamation,
commercial/service/council credit and shared issuance disabled. Native development
build, Quadro RTX 5000 with Max-Q Design. Local raw results are ignored under
`output/harbor-demand-screen/`; baseline is `output/adult-payroll-screen/`.

| Seed | Population before → after | Need-weighted ending hunger | Operator completed work | Cumulative operating margin |
| --- | --- | --- | --- | --- |
| 1024 | 160.170 → 159.871 | 0.03714 → 0.03695 | 11.060 → 11.219 | 48.342 → 47.422 |
| 256 | 335.503 → 337.571 | 0.03744 → 0.03323 | 3.809 → 4.500 | 15.860 → 18.502 |
| 409 | 347.072 → 352.819 | 0.07023 → 0.05919 | 49.216 → 41.532 | 190.581 → 134.217 |

All three runs complete and validate; maximum absolute relative monetary residual
is 1.29e-7. Operator margins remain positive but throughput is mixed. These small
screens are comparisons, not evidence of generally calibrated economic behavior.

Commissioned counts remain 2/4, 0/5 and 1/5. Seed 1024's first port commissions at
month 24 instead of 36. Ending annual leftover work is zero or float-rounding-sized
at all inspected seed 1024/256 ports. For example, seed 256 site 2 retains about
481 kg timber, 27.6 kg generic tools and 399 kg bricks, but its unfinished port has
only 55.5/0/17.8 kg installed and no available construction work. Cumulative harbor
work remains tiny and actually decreases slightly in each world.

The production connection is retained, but **does not solve shipping or circulation**.
The next concrete issue is reserving a bounded annual harbor construction/repair
allowance before other craft claims exhaust it, without stealing prepaid shifts or
basic water-service labor. Merely lowering commissioning thresholds would conceal
this missing work allocation. Generic-only installed equipment is another remaining
substitution limit.

The seven CPU production tests, both GPU fixtures above, native build and strict
library Clippy check pass. Compilation overlapped frozen-binary scenario runs;
these runs are not isolated performance benchmarks.
