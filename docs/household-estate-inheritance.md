# Local household estate inheritance

Local policy with balance screens below; broad circulation remains unresolved. This addresses a
specific continuity gap: a living descendant can already head another household,
while the former household retains its wallet and ownership indefinitely.

`HouseholdEconomy.inheritance.enabled` is an opt-in local succession policy. At the
Reserve retail boundary it considers households vacant for at least twelve months,
with deceased heads and no known living members. It excludes traveling/lost
households, abandoned sites and estates with live debt or unrecovered defaults.
It traverses recorded descent, stopping at living descendants. All resulting heirs
must be resident in eligible non-vacant households at the same settlement. If any
heir is away or cannot be matched, the whole estate is deferred. There is no distant
cash teleportation.

Eligible descendants divide cash above a protected food reserve and the estate's
town ownership share equally. Multiple heirs can receive into the same household
wallet; the last heir in stable person order receives the rounding remainder. Identities, historical heads and genealogy remain intact. Zero-property
households become valid; town ownership still sums to one. `inheritance_paid` and
`inheritance_received` distinguish these transfers from wages, profits and relief.
Dated receipts and events retain the source, beneficiary, heir, cash and share.
The policy runs once per monthly retail boundary. Future receipts reaching an old
estate can pass through a later eligible succession boundary; repeated invocation
within a month cannot spend them again.

The equal-recorded-descendant rule is an explicit toy inheritance convention, not
proof of exhaustive genealogical knowledge. In aggregate mode anonymous population
is not removed or reassigned. This policy changes economic ownership only. It does
not seize every vacant wallet, merge household rosters, grant a political office,
or resolve spouses' and distant relatives' competing rights. Unclaimed estates,
remote beneficiary settlement and economic calibration remain outstanding.
Living children exclude their own descendants from this division. The returned
heirs share equally regardless of generation; this is not division by family branch.
Each heir has a separate dated receipt; receipt counts are not estate counts.

The screens below predate shared division unless explicitly identified otherwise;
they evaluated the original sole-descendant policy.

## Controls and verification

Use `--household-estate-inheritance[=true|false]`; omission preserves the archived
policy. The monetary runner accepts `--household-estate-inheritance` and applies it
to all arms, explicitly disabling it otherwise. Existing histories default to off.

The pure descent fixture passes (living branches, nearest living ancestor and a
cycle guard). The GPU-backed transaction fixture passes: conserved cash and local
shares, protection of a living owner, no duplicate transfer on repeated invocation,
serialization/next-month equality, and rejection of a corrupted receipt. This is
not yet a full multi-month natural-history checkpoint comparison. All three cash
audit tests and twelve monetary report tests pass. The CLI override test, strict all-target Clippy and native build pass. The first matched
seed-1024 four-arm 50-year screen completed; outputs stay under ignored
`output/household-inheritance-screen/on/`. See the correction and results below.

## First screen and subsistence correction

The first four-arm screen completed before the food-reserve safeguard below.
All arms exited successfully. Without issuance, the policy transferred 1,539.70
in 146 receipts; with issuance, 2,093.87 in 78 receipts. The no-issuance endpoint
had population 159.435 and hunger 0.05832, versus 156.688 and 0.06939 without
inheritance. Operator operating margin worsened slightly (-275.98 versus -269.35).
With issuance, population was 157.960 and hunger 0.06391. Those endpoint differences
do not establish the mechanism for each downstream change.

Inspecting individual receipts revealed repeated small transfers from estates that
still had aggregate food need. In one case, 126 later transfers amounted to 18.82.
Absence of named members does not establish absence of anonymous dependents. The
revised policy therefore retains three months of the last completed household food
need, valued at the current local food quote, before transferring cash. It protects
full food cost rather than assuming common entitlement will be funded next month.
Ownership share can still pass to the heir. Receipts record protected cash separately
from transferred cash. This is an explicit conservative game reserve, not a solved
household population model; changing need and local prices affect future transfers.

The first screen is superseded for balance claims by this correction. The reserve-specific GPU fixture passed (0.94 seconds), as did strict all-target
Clippy. The corrected native build and revised four-arm screen passed. Before the correction the full CPU
library suite passed: 184 passed, 143 hardware tests ignored. No portability or
performance conclusion is drawn from these runs.

### Corrected screen

The same seed-1024 founding checkpoint was advanced for 50 years with demand-aware
staffing and corrected inheritance, all other settings matching the prior screen.
All four arms exited successfully. Local outputs are under
`output/household-inheritance-reserve-screen/on/`.

| Measure | No issuance | With issuance |
| --- | ---: | ---: |
| Inherited cash | 1,507.48 | 2,087.22 |
| Transfer receipts | 14 | 7 |
| Retained-wallet cash (overlapping vacancy/abandonment flags, union) | 15,032.08 | 17,807.16 |
| Population | 159.434 | 157.960 |
| Terminal need-weighted hunger | 0.058317 | 0.063906 |
| Operator revenue less wages and rent | -275.98 | -135.10 |

No loans were issued. Histories within each issuance pair are identical after
removing only credit records. Maximum absolute endpoint relative money residual
is 1.841e-7, not a bound over every month. Receipt counts fall from 146/78 in the
unprotected screen to 14/7 while preserving nearly all of its inherited value.
This validates the reserve's immediate purpose; it does not establish that the
reserve length is optimally calibrated. One seed cannot establish robustness.

Inheritance reduces one retained-wealth mechanism. It does not repair workshops:
operator margins remain negative in aggregate. The remaining retained wealth still
requires claimant-sensitive escheat or recovery, and abandoned town recovery needs
physical access and costs. Those parts of the circulation objective are unfinished.


## Shared descendants and local review screen (September 2026)

The shared-descendant change was compared with `d4f8b75` using identical 32/32
founding archives, seeds 1024, 256 and 409, fifty years, plus a paired seed-256
century. Delivery-paid exports, service procurement (share 0.25), contract/demand
workshop staffing, named office service, inheritance and abandoned-stock recovery
were enabled. Commercial/service/council credit and issuance were disabled.
Reclamation was initially disabled. Raw outputs remain ignored under
`output/shared-inheritance-screen/`; this is a balance screen, not a timing benchmark.

Seed 1024's fifty-year export is entirely identical. Seed 256's fifty-year export
is identical after normalizing the intended inheritance event wording; its century
population (210.820099), ending hunger (0.0418784554), inherited cash (3,090.56) and
operator margin (19.66) are unchanged. The two-heir estate worth roughly 1,618
observed in an older selective-harbor century does not recur in this current run.
Seed 409 now transfers one estate at month 566 to two descendants in the same
receiving household, rather than waiting until month 572 for one surviving heir.
Its cash is only 0.02695, alongside the finite ownership share. Ending population
changes from 352.5453864 to 352.5396424. This fixes a continuity rule but does not
materially release the large inactive balances in these worlds.

A second screen enables the existing local estate reclamation policy, keeping
shared inheritance and all other settings fixed. Review requires delivered local
administration and protects known kin and the dependent-food reserve. Its money
enters the **town operating account**, not the council treasury.

| Seed | Reclaimed cash | Population off → on | Ending need-weighted hunger off → on | Operator revenue minus wages/rent off → on |
| --- | ---: | ---: | ---: | ---: |
| 1024 | 3,407.23 | 154.263 → 153.638 | 0.12297 → 0.11533 | 50.79 → 50.67 |
| 256 | 1,319.52 | 345.715 → 345.715 | 0.03746 → 0.03746 | 19.66 → 19.66 |
| 409 | 537.00 | 352.540 → 352.524 | 0.01888 → 0.01891 | 952.25 → 952.25 |

Reclamation reduces endpoint vacant-account cash in all three worlds, but does not
establish population recovery or a stronger workshop economy. Keep it an explicit
experiment; do not enable it by default on this evidence. These results argue for
tracing town operating cash into feasible orders, inputs, equipment and labor,
rather than assuming a larger cash transfer fixes circulation. Abandoned sources
remain excluded from local estate review, and undeveloped deposits require actual
extraction rather than stored-stock collection.

Eight completed runs (three shared fifty-year, two seed-256 century arms, three
reclamation fifty-year) passed; maximum absolute endpoint audited relative cash
residual was 1.72e-7. The targeted GPU inheritance fixture checks two separate
beneficiary wallets, reserve retention, remote-heir deferral, conserved cash/shares,
same-month repetition and serialized next-month continuation. The ordinary library
suite passed 196 tests (152 hardware/long fixtures ignored), the targeted fixture
passed separately, and strict library Clippy/native build passed. Shared receipts
retain the old archive shape; their uniqueness includes the heir and their summed
ownership allocation is bounded per estate/month. No full long-run checkpoint or
cross-backend equivalence claim is made here.
