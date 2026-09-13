# Harbor labor experiments — rolled back

**Status: none of these labor-policy experiments is enabled or retained in current
source.** The material-demand connection from acd49bf remains. The experiments
proved that construction can receive finite work and commission more ports, but
did not establish an acceptable economic improvement. Keep circulation recovery
open; do not describe port commissioning as evidence of improved food access.

## Mechanisms tested

| Arm | Change relative to acd49bf |
| --- | --- |
| Baseline | Existing annual leftover-work installation, with harbor material orders |
| A: protect leftovers | Annual grant capped by opening material-supported work and 20% of remaining uncontracted craft time after services/buildings |
| B: include staffing demand | A plus supported harbor installation in adaptive craft demand, under existing farm/nonfarm caps and gradual reassignment |
| C: bounded public work | Replace the leftover fraction with a 2%-of-workforce annual allowance inside the existing 20% total service ceiling, excluding earlier services and prepaid workshop attendance; include it in staffing's service floor |

Each arm preserved water operation, prior building work and prepaid workshop
attendance. Protected time was kept outside ordinary unused work so earlier road
construction could not spend it. Annual shipping still required actual materials
above working reserves. Requested, protected, used and expired time were recorded;
unusable grants expired without backdated production. Named construction was
excluded rather than creating anonymous participants alongside named builders.
No money, materials, crew work or smaller/free ports were added.

## Settings and evidence

Baseline acd49bf; founding archives at terrain/ecology 32/32; seeds 1024, 256 and
409; 600 history months; zero additional geological epochs. Same controls as
[harbor material demand](harbor-production-demand.md): delivery-paid exports,
service-order procurement (share 0.25), contract/demand workshop staffing,
household inheritance, named office service and abandoned-stock recovery enabled;
estate reclamation, commercial/service/council credit and shared issuance disabled.
Native development build, Quadro RTX 5000 with Max-Q Design.

All nine experimental runs completed and validated. Local raw results and frozen
binaries are ignored under `output/harbor-work-screen/` (A),
`output/harbor-staffing-screen/` (B), and `output/harbor-service-screen/` (C).
The final experimental diff is retained locally in the last directory, not as a
shipped feature. Baseline results are `output/harbor-demand-screen/`.
Compilation and some focused checks overlapped frozen-binary runs: these are
behavioral comparisons, not isolated timing benchmarks.

### Population and food access

Hunger is the ending household-need-weighted burden, not a lifetime famine count.

| Seed | Baseline population / hunger | A | B | C |
| --- | --- | --- | --- | --- |
| 1024 | 159.871 / 0.03695 | 158.766 / 0.04042 | 158.862 / 0.03752 | 158.351 / 0.03090 |
| 256 | 337.571 / 0.03323 | 337.312 / 0.04227 | 330.490 / 0.03458 | 316.189 / 0.05461 |
| 409 | 352.819 / 0.05919 | 349.507 / 0.06223 | 322.082 / 0.07071 | 334.309 / 0.06332 |

A and B did not increase commissioned port counts. C did, but populations were
lower in every seed and hunger was worse in two. This does not prove the policy
would always fail; it does not justify enabling it by default either.

### C: actual work and economic consequences

| Seed | Commissioned ports, baseline → C | Cumulative harbor work | Operator completed work | Operator operating margin | Cumulative vessel crew wages |
| --- | --- | --- | --- | --- | --- |
| 1024 | 2 → 3 | 12.911 → 27.786 | 11.219 → 11.390 | 47.422 → 48.907 | 5649.56 → 7618.10 |
| 256 | 0 → 2 | 11.366 → 31.944 | 4.500 → 4.923 | 18.502 → 19.244 | 0 → 2396.79 |
| 409 | 1 → 3 | 15.136 → 35.233 | 41.532 → 135.048 | 134.217 → 656.285 | 1658.49 → 6275.80 |

Operator margin is revenue minus wages and rent, excluding capital transfers and
financing. Crew wages include standby work; they are not proof of delivered cargo.
Maximum absolute relative monetary residual in C was 3.33e-7.

## What changed the next action

1. A labor grant cannot fix a missing task in staffing demand. A mostly preserved
   tiny leftovers; B corrected that connection but still did not establish useful
   port operations. Sector-demand and allocation policy are separate concerns.
2. C proved the construction pathway: real work increased and five additional ports
   commissioned across the three seeds. But finishing harbors incurred recurring
   crew expenses and did not automatically connect a useful food surplus to need.
3. Important ports still lacked generic tools. In seed 256, sites 0 and 4 ended
   with approximately 200 kg timber and 100 kg masonry installed but **zero tools**;
   seed 409 site 1 had the same pattern. Extra construction time cannot install an
   unavailable component. Existing copper/bronze tool substitution does not extend
   to the harbor's fixed generic-tool inventory.
4. A small population may also have its entire service ceiling consumed by earlier
   commitments. The experiments did not prove that moving work away from those
   services would improve welfare.

Further stock inspection prevents an overly simple substitution diagnosis. C's
blocked seed-256 sites 0/4 and seed-409 site 1 hold about 22–23 kg generic tools
and **zero copper/bronze tools**. The port working-reserve floor is 0.5 kg per
resident, approximately 56–80 kg in these towns; it prevents installing any of
that generic stock. Seed-256 site 0 also has zero remaining local ore reserve and
no metal stock. Surplus copper cannot be assumed into existence.

The next experiment should distinguish desired stock targets from indispensable
working equipment and test explicit sharing of existing tools between operations
and infrastructure. Compare actual production/tool wear and shipment benefits,
not just whether a port can take the last available tools. Then test material
substitution where real alternatives exist, and recovered/externally supplied
metal where deposits are exhausted. Preserve embodied material identities and
include route usefulness and crew costs. Do not tune only for more commissioned
ports, higher operator revenue or cash movement.

## Verification of the experimental implementation

- 192 ordinary library tests passed; 149 GPU tests remained ignored in that command.
- Extended GPU craft fixture retained all prepaid attendance while constraining
  the harbor grant to the actual remaining pool; named construction received no
  anonymous grant. A no-recipe case showed increased craft assignment from a
  supported harbor request with total workforce fixed at 50 and no invented goods.
- GPU construction fixture combined protected and ordinary work, checked physical
  stock plus assets, repeated-boundary idempotence, expiration with missing
  materials, degradation/recovery and serialized continuation.
- Automatic service procurement and checkpoint continuation passed; native builds
  and strict library Clippy checks passed.

These checks establish bounded implementation behavior, not successful balancing.
The experimental source changes were rolled back after comparison; existing source
and defaults remain at the baseline behavior above.
