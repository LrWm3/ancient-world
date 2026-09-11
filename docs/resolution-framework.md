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

With comparison enabled, demographic receipts also retain a replayable
`DemographicSnapshot`: opening stocks, completed ration/disease-derived projection,
seed, month, birth carry, and (in individual mode) opening identities and birthdays.
`snapshot.compare()?` runs both pure resolvers from those same inputs. The returned
aggregate and individual outcomes include closing age stocks, births, deaths,
birthday transitions, deceased IDs and remaining birth carry. No births are registered,
no events emitted and no live state or random stream changed by replay.

Only the latest month's inputs are retained, so snapshot storage scales with the
current resident roster rather than elapsed history. Snapshots serialize with receipts;
older receipts without them still load. Public replay types are available through
`ancient_world::resolution`. Validation rejects duplicate identities, inconsistent
age bands, mismatched population stocks and invalid numerical inputs.

Aggregate-mode snapshots explicitly return an unavailable reason for individual
replay: they do not invent a reconciled population from sparse named records.
Independent long-running counterfactual histories remain separate runs. This tool
isolates demographic resolution under identical completed exposure; it does not
predict how an alternative population would have changed food consumption earlier
in the month. Nor does it replay the full scheduler or workshop hiring.

## Workshop pilot

Existing firms forecast affordable shifts after lease costs. Existing shared service
capacity, funding and demand remain constraints. In refined mode, present named
adults offer a fraction of their uncommitted time. Ambition and household cash/food
pressure influence that fraction, evaluated against the employer's actual wage
relative to the local food-price wage reference. Cash pressure uses a two-month gross
food bill as a buffer target; hunger uses the previous completed household food
allocation. Food records from another settlement are ignored after relocation.
Staffing precedes the retail reset, so current production is not anticipated.

Craft occupation familiarity and completed workshop practice influence stable hiring
priority. Each resident now retains completed worker-months for the four existing
recipe families. A family counts its own practice fully and other/untyped practice
at 20%; the score is `relevant / (12 + relevant)`. This gives diminishing returns
and partial transfer between trades without changing recipe yields. Existing archives
retain their total as untyped practice rather than receiving invented specializations.
New work is assigned its family only when its commitment settles; paid idle time
earns none.

Coworkers also learn by observation during completed production at the same firm.
The month uses opening family experience, so newly completed work cannot cascade
into instant expertise elsewhere. Each more experienced coworker's completed work
is shared across less experienced coworkers in proportion to their completed work.
Learning depends on the experience gap, is capped by the learner's own completed
work, and slows as a separate 0–1 competence score fills. Its gain scale is 0.05 per
worker-month of effective exposure. A fully idle or absent expert contributes zero.
This is incidental learning during production; no extra teaching labor is claimed.

This score adds at most one quarter of the remaining gap in practice-based hiring
priority. It does not add worker-months, wages, goods or recipe yield. The separate
`workshop_learning[4]` field defaults to zero in old archives. Structured courses,
explicit mentor contracts and teaching time remain absent.

Family sums cannot exceed total completed work. The 20% transfer and
12-month scale are game parameters, not measured skill acquisition. Sparse agents use neutral ambition where
no traits were recorded. Offers remain bounded between zero and remaining capacity;
zero pay produces no paid-work offer. These response constants are toy behavior
parameters, not calibrated labor economics. Employers still post the existing
food-indexed wage reference. Refined firms now post bounded employer-specific wage
multipliers, using completed prior-month hiring and production. Formal bargaining
and skill-dependent output quality are absent.

At settlement, firms remember vacancy pressure (75% prior memory, 25% new shortfall).
A raise needs pressure above 0.15, at least 80% utilization, payment covering payroll,
and three payrolls of remaining cash. It rises by at most 8% times remembered
pressure. Low utilization (below 50%) or payments below payroll instead reduces
the offer by 3%. The multiplier is bounded to 0.6–1.125 of the local wage reference;
these are explicit toy safety limits. Changes are pending until the next posting,
not retroactive adjustments to existing commitments.

Refined service contracts quote 1.25 times the food-indexed reference independently
of the firm's chosen wage. Rent also uses that reference. Thus raising wages cannot
increase the fee paid by the town. The profitable raise ceiling depends on observed
utilization. In aggregate/legacy mode the prior wage and fee rules remain.
Local hiring now uses bounded proposal rounds after all firms post affordable
quotes. A resident ranks jobs by wage/reference plus 0.25 times relevant experience,
then applies to one employer per round. Employers rank that round's applicants by
family skill and ambition, with person-ID ties. Rejected or partly accepted applicants
can try another job in the next round. Each job is tried at most once per resident;
with four workshop families this needs at most four rounds.

An accepted assignment reserves real remaining time immediately. Since each resident
has only one application per round, employers cannot double-book the same time.
Neither job grants nor cash budgets expand during matching. Canonical IDs break exact
preference ties and keep input-array order from deciding the result. Residents may
split time across jobs, but retained assignments are not displaced by later applicants.
This is a bounded local matching rule, not a stable-market equilibrium or a global
job search. Firms can still underfill due to preference, willingness or eligibility.

Commitments share
the same personal time ledger as research and culture. Time already committed there
cannot staff workshops. Payroll goes to actual participating ownership accounts,
including paid idle time, and is deducted from the firm's existing cash. Zero-staff
shifts neither receive GPU labor grants nor pay wages.

GPU recipes retain authority over physical output and material consumption. Actual
completed work settles personal commitments and lifetime workshop practice totals.
It changes firm revenue, household purchasing power, stocks and subsequent planning
through existing paths. Practice now improves hiring priority, but does not increase
physical output per hour or technical efficiency. The four enterprise workshop families participate; other production labor and vessel
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

- Extend snapshot comparisons beyond demography; individual replay from aggregate-only
  worlds still requires explicit roster reconciliation.
- Richer birth eligibility and reproductive circumstances; births currently retain the
  community-level expectation and unknown-parent fallback.
- Dedicated teaching, skill loss and broader labor-market matching; explicit coordination with
  agricultural and other aggregate labor before removing the service ceiling.
  Family-specific experience currently affects hiring, not technical output quality.
- More subsystem-specific input revisions and reservation references as contracts
  become stable. Keep diagnostic summaries separate from behavioral feedback.

See [verification](resolution-framework-verification.md).
