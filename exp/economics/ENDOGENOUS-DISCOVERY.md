# Starting an economy without arranged deals

Implemented 2026-10-05. The opt-in `discovery` adapter extends the
[combined agency control](INTEGRATED-AGENCY.md) through the existing scheduler,
allocation policies, contract adapters and double-entry reporting. The opening
world contains **no citizenships, household, governance selections, land rights,
accepted agreements, loan or forward terms, counterparty quotes, dated work
instructions or organization program menus**.

This removes those scenario setup requirements for a bounded person–household–state
loop. It does not make every existing experiment self-starting.

## What remains a model input

Persons, an existing public agent, a venue, assets, endowments, needs, recipes and
laws still define the world. Static constitution/charter templates define permissible
leadership and allocation options, contribution fractions and formation requirements.
Objectives, review horizons, lease/loan term rules, market listings and unit price
limits express preferences and institutions. Agents choose counterparties, whether
to sign, operating policies and dates; they do not invent technologies or legal powers.

The control retains three persons, a public plot and wheat, finite private metal
and coins, seed and monthly labor. Two persons need food and have five hours each;
cultivation requires six hours. Household contributions are 20%, output pooling is
half, and constitutions allow elected two-month terms. The outside worker has no
recurring food need in this test. The crop remains a one-month recipe yielding
four wheat and two seed from one seed and six hours. This is a computation and
integration control, not historical agricultural calibration.

The land rule charges one wheat annually and offers 24-month access. This differs
explicitly from the older control's one-coin lease. State reserve objectives and
fixed valuation/interest assumptions remain supplied. Individual self-directed
policy changes remain deferred.

## Decisions and visibility

At **Open**, one staged discovery pass:

1. Posts obligation-free citizenship and offers idle publicly owned plots under
   the legal term template. An expired right permits a new dated offer.
2. Installs governance of the existing public agent after citizenship exists.
   The lowest eligible citizen is the initial governor; the first constitutional
   policy is the bootstrap default. Subsequent policy/work choices and ballots use
   the ordinary objective controller. This does not found a sovereign from nothing.
3. Derives market counterparties and holding targets from venue eligibility,
   needs, stocks and capacity. No actor IDs or submission dates are supplied as quotes.
   The [supply follow-up](DISCOVERED-SUPPLY.md) compares cumulative outgoing lots
   with declining, protects private/member needs and active work, and caps actual
   orders at the chosen quantity. An offer is not evidence of a fill.
4. Evaluates eligible pairs for household formation. Each person compares the
   proposal with remaining independent using their own ordered death/need outcomes.
   Both must be no worse, and one must improve. Permitted initial household policies
   are evaluated; normal formation validates law and terms. Stable ordering resolves
   ties, without a global coalition optimizer. Shared derived voter preferences
   let elections continue after the founding term.
5. Tests bilateral financial proposals against independent borrower/buyer and
   lender/seller objectives. A selected loan must repay within its projection;
   a selected prepaid delivery must deliver. Actual ordinary admission and later
   performance still enforce finite resources and can fail.

Controllers discover candidate commands from constitutional options and backwards
resource reachability through recipes. Each decision retains its catalog,
projections, authority and issued command. Prohibited or infeasible commands do
not become authority merely by being discovered. A selected instruction takes
effect the following month. Minting can begin idle, with no prearranged first attempt.

At **Acquire**, free citizenship uses a standing acceptance policy. Land admission
collects feasible applicants before granting plots. The existing `StablePriority`
allocator reserves joint prerequisites in a temporary budget: plot exclusivity,
seed and private plus constitutionally available household hours. Existing active
work and outgoing market transfers reduce availability. Incoming transfers do not
count as already-finalized admission stock. Receipts distinguish feasible requests,
grants and rejected joint reservations. These are admission bounds, not an escrow
or a guarantee that future household policy will direct those hours to this job.
Ordinary productive allocation still selects actual work.

The remaining production, consumption, pooling and settlement phases are unchanged.
Only completed work creates outputs; forecasts create no spendable stock. New land
commitments and their annual claims enter the same obligation machinery. Under
`NeedsThenCommitments`, collective work preparation now includes uncovered member
land/forward claims already eligible for household support, after protecting current
private consumption. That creates no assumption of a member's debt or right to
appropriate private goods.

Financial reporting accepts explicit resource valuations for newly discovered land
agreement IDs and freezes each admitted basis. Explicit per-contract bases retain
precedence. Simulation and journal publication remain atomic; no future contract IDs
need to be preallocated in the opening accounting configuration.

## Verification

`tests/discovery.rs` checks:

- Empty opening arrangements lead to citizenship, an elected household and state,
  leased cultivation, repeated minting and a fully repaid state loan. Both food-needing
  people have one unmet unit in month 1 and none in months 2–14. The lease accepted
  in month 2 pays its first annual wheat bill in month 14.
- CPU and reference execution, ledger, reports and separate books agree. Rebuilding
  the simulation at every phase preserves the result. Reversing input catalogs
  preserves decisions and accounting.
- A 40-month continuation reoffers the expired plot, accepts the replacement in
  month 26 and pays its annual bill in month 38, without later scenario instructions.
- A richer state and six-wheat harvest make prepaid deliveries mutually useful:
  at least two are independently proposed, accepted and delivered by month 8.
- No seed prevents cultivation and land acceptance; denied citizenship prevents
  governance, household formation and minting; denied household formation prevents
  an unfulfillable six-hour cultivation commitment. No opening coins means no lender
  advance and no minted output. Removing all needs produces no beneficial household.
- Two seeded applicants with two plots cannot both count the same household hours
  at admission. Altered allocation/signature/catalog records fail validation; a
  failed Open publishes neither arrangements nor economic effects.

`tests/agency_programs.rs` independently checks recipe/constitution-derived programs,
CPU/reference equality and rejection of prohibited minting. Existing supplied-menu
and signed-contract controls remain regression controls.

The CPU examples finish with these observed outcomes:

| Control | Actual outcome after 14 months |
| --- | --- |
| Baseline | Three mint batches issue 30 coins; state closes with 14 coins; its six-coin advance from the outside worker is repaid by month 6 |
| Baseline forwards | Rejected when buying wheat would worsen the state's higher-priority coin reserve; an available seller does not force a trade |
| Surplus variant | One-unit forwards accepted in months 2 and 5 deliver by months 4 and 7; state closes with 18 coins and nine wheat, with no minting or loan needed |
| Both variants | Three citizenships, one household, one land agreement; two initial unmet food units, no subsequent food deficit, annual wheat bill paid |

A proposal receipt's `accepted` flag records a passing comparison. Several
household initial-policy alternatives can pass; the separate `formed household`
receipt identifies actual formation. Published terms, accepted contract state and
settlement receipts remain the evidence of issuance and performance.

Verification for this change: **211 tests passed** across 25 selected targets,
including the 12 discovery and two program-discovery controls, existing agency,
state formation/governance, minting, financial/common offers, dues/reporting,
household funding/support, conditions and base economics. Both CPU examples,
strict all-target Clippy, formatting and repository artifact checks passed.
This selected regression run is not a new full-crate release certification.

## Boundaries still present

Discovery is opt-in and bounded to eight activity participants, at most 24 forecast
months and 16 organizational program candidates. Household proposals currently pair
people; they do not recruit into an existing household or negotiate constitutions.
Constitution templates and technology catalogs are static. Historical program
catalog validation deliberately rejects incompatible later catalog edits.

Financial discovery currently offers mint-input funding loans and reserve-driven,
one-unit prepaid deliveries. It is not underwriting for arbitrary projects,
mortgages, employment or recovery deals. Price limits and unit valuations are rules,
not ZIP discovery. Quote generation uses the existing mint venue's listings and
person counterparties; it is not universal asset, rights or membership trading.
The adapter excludes employment contracts and competing composition planners rather
than promising their claims the same capacity. Other driver combinations need their
own integration controls.

Private projections freeze future offer generation, formation and peer organization
policy decisions, while retaining published offers and ordinary acceptance/work.
Bilateral benefit is therefore conditional on a finite forecast, not proof of
long-run viability or consent under every shock. Stable admission can favor lower
IDs. Initial preferences are derived uniformly for each office; political disagreement,
automatic sovereign formation, multiple jurisdictions and self-modifying preferences
remain separate work. There is no mortality rule in this control.

## Reproduce

The `--circulation` variant connects discovered paid mint labor to later public
food purchases. It starts the worker with zero coins and three wheat, and the state
with six coins and sixteen wheat. Public surplus sales protect four wheat. The
supplier retains its six opening coins and finite metal, so its earlier food bids
compete with the worker's later income. Other persons have one labor hour each,
below the two-hour mint lot. Land, household and finance discovery are disabled
for this control; their original nutrition needs remain visible.

Over fourteen months, the worker earns four coins in month three, buys three wheat
for three coins in month four and consumes six wheat including its opening buffer.
CPU/reference state, receipts, reports and books agree, including reconstruction
at every phase between earning and buying. No loan, grant or supplied agreement
finances the worker. The [review record](REVIEW-ITERATIONS.md#5-discovered-wages-to-food-circulation)
reports all participants and the paired stock/reserve controls. Eight worker food
deficits remain: the public coin objective is satisfied, paid work stops and the
worker cannot afford another lot despite remaining public surplus. This verifies
adapter composition, not sustainable production or a general subsistence policy.

From `exp/economics`:

```sh
cargo +1.92.0 run --release --locked --example discovered_economy
cargo +1.92.0 run --release --locked --example discovered_economy -- --surplus
cargo +1.92.0 run --release --locked --example discovered_economy -- --circulation
cargo +1.92.0 test --release --locked --test discovery --test agency_programs
cargo +1.92.0 test --release --locked --test discovered_circulation --test public_sales
```

The example runs on CubeCL CPU and prints proposal comparisons, accepted programs,
settled obligations and financial outcomes. Generated logs belong in ignored
`output/`; this document records the reviewable findings. The historical v1 release
baseline is unchanged; this is subsequent opt-in integration work.
