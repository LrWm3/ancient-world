# Aggregate projections and individual resolution

The first implementation keeps the five monthly phases and introduces a shared
boundary, reconciliation receipt and commit guard for **demography and enterprise
staffing**. Other history systems retain their existing contracts. This is a toy
model comparison facility, not a claim of empirical calibration.

## Authority and data flow

A pure demographic projection reads opening age stocks and completed GPU ration/
disease exposure. An aggregate or individual resolver produces an outcome without
changing the world. The batch checks population balance and opening stocks before
committing births, deaths, age stocks and identity changes. Aggregate mode uses
fractional transitions; individual mode uses named birthdays and sampled deaths,
whole registered births and explicit anonymous residuals.

Both use the same CPU demographic settlement. The GPU continues production,
rations, disease and weather but suppresses its own demographic transfers when this
framework is enabled. Worlds without the framework retain the legacy GPU path.
The CPU aggregate resolver uses f64 arithmetic; exact numerical parity with the
legacy f32 path is not promised. Game rates remain unchanged.

Each `Boundary` contains month, subsystem, site, subject and a numeric input
fingerprint. `ResolutionState` rejects stale supplied boundaries, duplicate keys
and backward commits. Concrete subsystems also validate their stocks, grants and
references. This is not a universal inventory transaction engine: existing monthly
History transaction handling and subsystem resource ledgers still own rollback and
conservation. Projections must not be treated as spendable resources.

## Reconciliation

`History::resolution_report()` exposes actual mode, the latest month's committed
receipts and running comparison summaries. With comparison enabled, metrics record
expected and actual values, units and mechanically identified differences. Summaries
retain expected/actual totals, absolute error and squared error. They are bounded by
subsystem, mode and metric, rather than growing with months or residents.

Demographic metrics distinguish birth carry, birthday/survival structure and the
remaining mortality difference. Mortality variation is not automatically a model
error. Workshop metrics distinguish requested staffing from granted personal time,
and granted time from completed work. The latter is unused paid employment, not
missing inventory. Comparisons never write ordinary historical events or alter
random streams, grants, demographic carries or choices.

This increment compares actual outcomes with the aggregate conditional projection.
It does **not** run a complete alternative individual history in aggregate mode.
Independent counterfactual histories remain separate runs; individualized shadow
resolution from incomplete aggregate rosters would require explicit reconstruction.

## Workshop pilot

Existing firms forecast affordable shifts after lease costs. Existing shared service
capacity, funding and demand remain constraints. In refined mode, present named
adults offer a fraction of their uncommitted time. Ambition influences that fraction;
craft occupation familiarity and ambition influence stable hiring priority. Sparse
agents use a neutral default where no traits were recorded. This is a deliberately
small preference model, not a full employment market or household utility solver.

Hiring uses canonical site/family/firm order and person-ID ties. Commitments share
the same personal time ledger as research and culture. Time already committed there
cannot staff workshops. Payroll goes to actual participating ownership accounts,
including paid idle time, and is deducted from the firm's existing cash. Zero-staff
shifts neither receive GPU labor grants nor pay wages.

GPU recipes retain authority over physical output and material consumption. Actual
completed work settles personal commitments and lifetime workshop practice totals.
It changes firm revenue, household purchasing power, stocks and subsequent planning
through existing paths. Practice totals do not yet increase skill or efficiency.
The four enterprise workshop families participate; other production labor and vessel
crews still rely on their existing aggregate allowances. The aggregate service ceiling
remains in this pilot, so named employment is not yet a complete resident labor model.

Aggregate staffing uses the same receipt/settlement interface, with the existing
household payroll weights and no new personal commitments. Business entry, rent,
lease obligations and invoices continue their existing timing and accounting.

## Controls and conversion

- `History::set_demographic_resolution(Mode::Aggregate, compare)` selects fractional
  resolution through the shared path.
- `History::set_demographic_resolution(Mode::Individual, compare)` invokes the
  existing atomic roster baseline conversion, then selects individual resolution.
- `History::set_workshop_refinement(true)` requires individual demography and the
  participation ledger. Disable workshops before disabling those providers.

Settings change at completed work boundaries. Conversion preserves real population
stocks and known IDs; it rejects incompatible cohort overhang and people in transit
or service. Returning from aggregate to individual mode after divergence may require
additional reconciliation. Arbitrary switching is not lossless. Geographic mode
selection is not implemented. Old aggregate archives default to the legacy path.
Pre-framework individual archives initialize the commit ledger on their next step; demographic
and workshop receipts, grants and comparison settings serialize with History.

    cargo run --release --example cultural_work_calibrate -- --aggregate-resolution --compare-resolution --seeds 17,81,256 --years 25 --output output/aggregate-resolution.json
    cargo run --release --example cultural_work_calibrate -- --individual-demography --workshop-refinement --compare-resolution --seeds 17,81,256 --years 25 --output output/individual-resolution.json

## Next extensions

- Snapshot comparison tools that run both resolvers without committing an alternative.
- Richer birth eligibility and reproductive circumstances; births currently retain the
  community-level expectation and unknown-parent fallback.
- Skills, wages and household circumstances in work offers; explicit coordination
  with agricultural and other aggregate labor before removing the service ceiling.
- More subsystem-specific input revisions and reservation references as contracts
  become stable. Keep diagnostic summaries separate from behavioral feedback.

See [verification](resolution-framework-verification.md).
