# Individual travel and participation verification

This increment links expedition service to historical people and their ownership
households. It does not complete the resident census or demographic-authority
conversion. Population and ordinary production labor remain cohort-based.

## Changes exercised

- Existing named adults can leave on expeditions; current service commitments and
  travel prevent conflicting local participation.
- When necessary, unnamed adults already in the town's cohort receive identities at
  recruitment. Their initialization is explicitly recorded, with estimated ages and
  unknown parents. Recruitment still subtracts exactly eight adults from town stocks.
- Rescue preserves identities and moves their active duties; death updates historical
  people and succession without spending another town mortality credit.
- Existing voyage escrow pays living crew households. Return preserves competence and
  witnessed field locations; elderly returnees enter the elder cohort.
- Cultural eligibility includes known adult kin rather than only ownership heads.
  Both eligibility and the participation allowance use the adult workforce age range.
- Expedition outfitting checks installed harbor capacity. Merchant cargo continues to
  require paid vessel staffing.

## Verification method

Rust 1.89.0, Quadro RTX 5000 with Max-Q Design, Vulkan. Generated files are ignored
under `output/`; only this summary is committed.

The expedition fixture initially failed before launch. A detached checkout of
`b09e326` reproduced that failure without these changes. The harbor had adequate
installed infrastructure but no funded merchant crew capacity. Expeditions already
reserve their own crew, timber and equipment, so their launch now checks usable
harbor infrastructure instead. The checkpoint fixture explicitly exercises that case.

Focused checks cover original IDs at departure/return, unique rescued identities,
missing-duty rejection, absence after disabling optional participation, named casualty
events, finite remittances, intact escrow/refund accounting, and exact whole-history
checkpoint/batch continuation. The participation fixture covers real research output
under competing commitments and eligibility of a known adult who owns no household.
The culture suite checks existing patron, artifact, faith and transaction behavior.

The 100-year ensemble uses seeds 17, 81 and 256, terrain edge 32, ecology edge 16,
one geological epoch, 16 founding civilizations and default crop yield 0.5. Society,
politics, governance, offices, shipping, expeditions, discoveries and living history
are enabled. Every monthly update runs the existing validators; quarterly checks
compare personal contributions with cumulative cultural/research work. Ten-year
samples and endpoint economic residuals are retained locally.

This is a game-behavior and accounting check, not demographic calibration. A seed
without expeditions is a valid outcome; controlled fixtures supply the travel coverage.
Cultural candidate expansion intentionally changes historical trajectories. Timings
from runs overlapping compilation or other checks are not performance benchmarks.

## Results

- Ordinary library suite: 76 passed (69 hardware tests excluded by that command).
- Participation suite: four passed, including one hardware fixture; three overlap
  the ordinary library suite.
- Culture hardware suite: nine passed.
- Expedition hardware suite: six passed, including save/resume and monthly/batch
  whole-history equality.
- Clippy across all targets with warnings denied, formatting, and artifact policy passed.

This is **92 distinct tests**, not the complete repository hardware suite.

| Seed | Resident population at year 100 | Voyages | Distinct crew person IDs | Expedition deaths | Research worker-months | Largest absolute endpoint economic residual |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 17 | 2,680 | 3 | 16 | 8 | 0.00 | 6.99e-06 |
| 81 | 3,191 | 19 | 106 | 16 | 68.44 | 2.2e-05 |
| 256 | 2,602 | 0 | 0 | 0 | 0.00 | 5.83e-06 |

All three worlds completed 100 years with validators enabled. No crew remained away
at the endpoints. The residuals above are the existing normalized economic checks,
not errors in every planetary field. Seed 81 performed actual specimen research;
seeds 17 and 256 did not. The absence of voyages in seed 256 is not treated as failure.
None of the natural runs needed to identify additional cohort adults at recruitment;
the forced-commitment casualty fixture exercises that bridge instead.

Final cultural contributions were 1,868.825, 1,858.400 and 1,814.050 worker-months
for seeds 17, 81 and 256. Personal/subsystem contribution checks passed each quarter.
About 5.1–5.3% of funded cultural bundles were cancelled by the existing identity
and execution guards. Cancellation is reported, rather than counted as completed work.

An earlier diagnostic retaining the owner-only cultural candidate pool produced
3, 28 and 0 voyages. The final pool produced 3, 19 and 0; this is a changed-history
comparison, not an isolated estimate of one parameter's causal effect. The controlled
fixtures are the evidence for particular mechanisms. Broader population authority,
individual ordinary employment and biological aging at sea remain future work.

## Outstanding authority conversion

The sparse named roster still is not a census. Domestic family groups must remain
separate from the current ownership accounts. Complete initialization, individual
food needs and births/deaths, military and general relocation membership, and
production employment/payroll must be converted before disabling aggregate demographic
updates. Ownership-account relocation currently waits for expedition members; allowing
other domestic families to leave independently needs that domestic-group separation.
Ordinary household payroll still uses aggregate work weights. Legacy archived voyages
remain anonymous with respect to the person registry.

## Reproduction

```sh
CARGO_INCREMENTAL=0 mise exec rust@1.89.0 -- cargo test --lib
CARGO_INCREMENTAL=0 mise exec rust@1.89.0 -- cargo test --lib participation::tests -- --include-ignored --test-threads=1
CARGO_INCREMENTAL=0 mise exec rust@1.89.0 -- cargo test --test culture --test expeditions -- --ignored --test-threads=1
CARGO_INCREMENTAL=0 mise exec rust@1.89.0 -- cargo run --example cultural_work_calibrate -- --seeds 17,81,256 --years 100 --output output/individual-travel-seeds-final.json
CARGO_INCREMENTAL=0 mise exec rust@1.89.0 -- cargo clippy --all-targets -- -D warnings
python3 scripts/check_repository_artifacts.py
```
