# Replacement jobs and toolmaking capability

This is the first capability increment, motivated by the
[maintenance ablation](maintenance-ablation.md). It prioritizes concrete local tool
orders and derives a local labor advantage from completed work. It does **not**
yet implement portable product quality, durability, reputation diffusion, expert
migration or manuscript-derived proficiency. Existing shipment and repeat-customer
systems consume the resulting ordinary goods and prices without a new trade model.

## Controls and state

Both settings default false, including missing fields in older catalogs:
`production.replacement_tool_jobs` and `production.toolmaking_expertise`.
They require planned production. They can be switched independently in the explorer
or through `configure_economy`; neither requires the experimental food allocator.
Disabling learning freezes its stored expertise and removes its labor benefit.

`Economy` adds fixed, aligned GPU state (288 bytes/site):

- `tool_craft`: expertise [0,1], priority enabled, learning enabled, cumulative
  completed toolmaking worker-months while learning is enabled.
- `tool_work`: this month's priority budget, priority work actually used,
  completed toolmaking work and output kg.
- `tool_orders[64]`: planned batches serving local working-tool demand, capped
  by the existing ordinary recipe orders.

Old states initialize these fields to zero. They serialize with the existing
monthly world archive. The explorer and `History::production_summary` expose the
measurements. No extra GPU readback, dispatch or separate population is introduced.
The existing CPU planner constructs orders; GPU execution decides what can actually
be completed. This is not a claim that planning itself moved onto the GPU.

## Order execution

The local working requirement is 0.5 effective kg/person, including existing goods,
incoming shipments and funded expected supply. Generic and bronze tools count at
1; copper tools count at 0.6, matching existing production. The normal source-aware
recipe planner traces inputs and respects recipe knowledge. Export expansion and
the larger ordinary reserve target do not independently justify priority work.

After existing service and infrastructure commitments, priority execution can spend
at most 20% of the remaining recipe labor. It uses the same inventories, finite
workshop capacity, residue capacity, storage ceilings and targets as ordinary work.
Completed batches count against ordinary orders so the second pass cannot duplicate
them. Blocked work does not hold labor or inputs across months; unused work returns
to the ordinary pass immediately. This is a bounded scheduling priority, not a
persistent exclusive material reservation or a guarantee of tool output. Rotating
recipe order can defer an upstream/downstream chain to another month.

## Practice and labor efficiency

Only recipes actually producing generic, bronze or copper tools earn toolmaking
practice. Queued jobs, extraction and intermediary smelting do not. If S is the
previous skill and p is completed toolmaking worker-months divided by available
workers, clamped to [0,1]:

```
actual tool recipe labor = catalog labor × (1 − 0.2 S)
S_next = clamp(S + 0.04 p (1 − S) − 0.002 (1 − p) S, 0, 1)
```

The same actual labor pays against worker and workshop budgets. Catalog inputs,
outputs, waste and wear are unchanged. Labor efficiency can increase output when
labor binds; it cannot bypass materials, demand or installed capacity. Current
staffing forecasts remain conservative, using catalog labor costs.

Constants are provisional game rules, not empirically fitted learning rates.
Expertise belongs to the site in this increment; it does not yet follow people,
and inactive/abandoned sites outside production do not receive these updates.
A zero-skill baseline grants no inherited expertise to old histories.

## Verification and development comparison

The GPU fixture tests finite priority and total work, zero-skill equivalence,
analytical learning/decay, the 0.1 → 0.08 worker-month/kg labor bound in a controlled
tool recipe, material residuals and monthly/batched checkpoint continuation.
All 17 economy GPU fixtures passed; the extended expertise check also passed.
A seed-17 two-month smoke comparison of both food-policy/fixed and open/closed-mine
branches preserved all eight recorded monthly observations against the previous
archive with the new controls disabled. This is limited trajectory evidence, not
proof of every GPU byte or arbitrary long-run equivalence.

Development comparison (coefficients fixed before results): seeds 17, 81, 256;
64/32 terrain/ecology; one epoch; 16 founding civilizations; living environment,
shared resources; 12-month spinup; yield 0.33; accessible starting tools withheld
in conserved custody. Three policies each run 120 further months: food-only,
replacement jobs, replacement jobs plus learning. Food pressure is enabled and
broad industrial maintenance reservation disabled in all branches. Mines remain
open. This is 1,080 monthly observations, using development seeds, not a held-out
or empirical calibration. All monthly site measurements and economy residuals
are saved, including incomplete results if a later branch fails.

```sh
mise exec rust@1.89.0 -- cargo test --test economy -- --ignored --test-threads=1
mise exec rust@1.89.0 -- cargo run --example toolmaking_evidence -- output/toolmaking-results.json
```

Portable quality should follow as a separate, audited extension: quantities and
quality moments must follow every transfer to households, workshops, shipments,
armies, expeditions, relocation cohorts and objects before durability depends on
quality. Reputation should then come from witnessed deliveries, extending the
existing repeat-customer contracts rather than giving buyers global rankings.

## Development results at dfa1061

All nine branches completed (1,080 monthly observations). Relative to food-only,
replacement priority increased cumulative tools made by 81.37–107.86 kg, increased
cumulative harvest by 99,974–141,876 kg, reduced unmet rations by 17,030–23,523 kg
calorie equivalents, and increased final settled population by 51.45–71.72 people.
These are outcomes of coupled histories, not isolated mortality effects.

Adding expertise to replacement jobs changed final population by −0.251, +0.324,
+0.479 across seeds 17, 81, 256. Final mean skill was 0.0061, 0.0051, 0.0069;
maximum local skill was 0.0190. The large effect in this experiment is job priority,
not learning. Learning is deliberately left weak pending longer histories and
teacher/workshop continuity models; the learning coefficients were not changed
following this comparison. All settings remain opt-in.

Maximum absolute normalized economy residual was **2.508975e-6**. All 42 ordinary
CPU tests passed (100 hardware tests skipped in that invocation). The 17 economy
GPU tests and extended expertise fixture passed, as did formatting and Clippy.
This increment did not rerun the complete ignored GPU suite or the held-out policy
factorial. The example validates monthly economy state and material/money residuals;
it is narrower than the full environmental evidence runner.

- [Generated table](evidence/toolmaking-tables.md)
- [Artifact retention policy](evidence/README.md)
- [Artifact retention policy](evidence/README.md)
- Logs: [Artifact retention policy](evidence/README.md),
  [Artifact retention policy](evidence/README.md),
  [Artifact retention policy](evidence/README.md),
  [Artifact retention policy](evidence/README.md),
  [Artifact retention policy](evidence/README.md)

```sh
python3 scripts/summarize_toolmaking.py docs/evidence/toolmaking-results.json.gz
```
