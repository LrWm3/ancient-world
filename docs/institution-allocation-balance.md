# Institutional allocation: controlled scarcity

This is game-balance evidence, not a historical or sociological calibration.
`Stable` remains the default. `Rotating` is an opt-in alternative within each
institutional action class; it does not change the cultural grant or election-first
priority. See [the implementation boundary](governance-duty-audit.md).

## Twelve-quarter fixture

Seed 42, terrain 32/ecology 16, five starting civilizations, Quadro RTX 5000.
Four test schools share two eligible local representatives and equal opening
readiness (0.5). Each receives a declared treasury transferred from town finance.
Their upkeep requests are 0.025 worker-months. Population, food and environmental
exposure remain fixed: this isolates service allocation, not whole-history balance.

| Quarterly allowance | Priority | Work after 12 quarters, per institution | Final readiness | Operational institutions |
|---:|---|---|---|---:|
| 0.025 | Stable | 0.300, 0, 0, 0 | 1, 0, 0, 0 | 1/4 |
| 0.025 | Rotating | 0.075 each | 0.14 each | 0/4 |
| 0.050 | Stable | 0.300, 0.300, 0, 0 | 1, 1, 0, 0 | 2/4 |
| 0.050 | Rotating | 0.150 each | 0.74 each | 4/4 |

Both policies consume identical total labor at a given allowance and preserve total
money exactly in this fixture. Every quarterly reserved plan also resumes identically
through serialization. A separate test reverses input request order and verifies
that each policy produces the same ordered requests; rotation gives each institution
first access three times in twelve quarters with a fixed eligible request set.

These results support keeping a choice of policies. Rotation reduces permanent
exclusion, but spreading insufficient service across all institutions can leave
none operational. It is not a universal improvement. Conversely, concentrated
priority can unnecessarily exclude viable institutions when enough total service
exists to sustain broader participation. Changing membership, finances, buildings,
request sizes or the eligible request set can change these outcomes.

## Whole-history diagnostics

The balance runner accepts `--rotating-institutions`. Its monthly observations
accumulate requested/granted/used institutional work separately for elections and
upkeep. It also counts site-quarters with multiple requests in each class, and those
with an aggregate grant shortfall. Shortfall counts include unavailable participants;
they must not be interpreted solely as a shortage of town labor.

These counts establish whether a policy comparison encountered competition.
Population differences alone cannot show which allocation mechanism caused them.
Raw run output belongs under ignored `output/`.

Verification also includes 115 regular library tests (105 hardware tests skipped),
four hardware institution fixtures, and full frozen-history monthly/batched/
checkpoint equivalence on seeds 17, 81 and 256. Seed 256 enables rotation; the
other arms retain their prior policy. This checks continuation, not balance.

## Matched living histories

Completed on implementation `2f81e43`: seeds 17, 81 and 256, 30 years each,
terrain 32/ecology 16, one geological epoch, sixteen founders, crop yield 0.5,
individual demography and workshop/agriculture/extraction/construction refinement,
with resolution comparisons. Both arms use a **0.65 common-food share intervention**;
this is not the default entitlement. Named office service is not enabled. The only
policy difference is `--rotating-institutions`.

| Seed | Population stable → rotating | Operational / active institutions, both arms | Upkeep used, stable → rotating (worker-months) | Multiple-upkeep-request site-quarters | Grant-shortfall site-quarters, both arms |
|---|---|---|---|---|---|
| 17 | 1,961 → 1,960 | 0 / 32 | 427.625 → 427.625 | 1,637 | 0 |
| 81 | 1,995 → 1,994 | 0 / 32 | 403.125 → 403.125 | 1,512 | 0 |
| 256 | 2,054 → 2,054 | 0 / 32 | 430.875 → 430.750 | 1,656 | 0 |

All sixteen sites remain active in each arm. All requested election work is
granted and used; all requested upkeep is granted, with small execution losses.
Population residuals are zero; maximum monthly food partition residual across both
arms is 1.65e-7. Each seed takes approximately 61–63 seconds on the Quadro RTX 5000.
The comparison script verified complete ensembles and matching settings, explicitly
allowing only the rotation setting to differ.

The histories are not identical: changing which eligible member is reserved first
can alter later personal availability even when institutional grant totals match.
These runs do not exercise sustained institutional grant scarcity and therefore
cannot establish which priority works better under that condition. Keep Stable as
the default and retain the controlled scarcity evidence above.

A separate concern is that **none of the active institutions is operational at year
30 in either arm**, despite nearly complete upkeep delivery. An active record is
not evidence of a functioning institution. The next diagnostic samples each
institution's treasury, readiness, mandate, living membership and building state,
so we can distinguish administrative funding, usable space and succession gates
from allocation shortfall before changing balance.

Reproduce each arm with `target/debug/examples/cultural_work_calibrate --seeds
17,81,256 --years 30 --common-share 0.65 --individual-demography
--workshop-refinement --agriculture-refinement --extraction-refinement
--construction-refinement --compare-resolution --output output/institution-stable-30.json`.
Add `--rotating-institutions` and use a separate output path for the treatment.
Compare with `python3 scripts/compare_food_access.py output/institution-stable-30.json
output/institution-rotating-30.json --allow-difference rotating_institutions`.

### Seed 17 operational-gate follow-up

A read-only diagnostic rerun adds institution records and living-member counts to
the decadal samples. It reproduces **all preexisting report fields exactly** after
excluding elapsed wall time and the added observations. The example builds and
passes Clippy with warnings denied.

| Year | Institutions | Readiness below 0.25 | Vacant mandates | Building condition below 0.25 | Treasury below quarterly 0.5 fee |
|---|---:|---:|---:|---:|---:|
| 10 | 26 | 26 | 0 | 0 | 26 |
| 20 | 32 | 31 | 4 | 0 | 24 |
| 30 | 32 | 32 | 4 | 22 | 32 |

These are overlapping gates, not mutually exclusive causes. At year 30 treasuries
range from 0.000011 to 0.000229 abstract currency units. Every institution has at
least two surviving members; survival counts do not establish local adult
availability. The readiness equation limits support by actual fee payment as well
as work, usable space and local staffing. The first four institutions received
14.875 worker-months each but paid only 11.70–17.00 cumulative administration fees
over their lives. Funding and maintenance are therefore concrete next mediators
to inspect, rather than increasing cultural labor or assuming rotation fixes them.

This does not yet prove that more donations alone restore service: usable-space
requirements, building repairs, local staffing, succession and the timing of
fundraising must be held or varied explicitly in the next controlled comparison.
The annual/decadal snapshots also do not measure every quarterly cash shortfall.
