# Flood isolation and cultural contact

Temporary road closures must affect more than cargo. Previously, cultural
pilgrimages, small personal relief gifts and annual overland contact checked
`Route.open` but ignored `flood_months`. A politically open flooded road could
therefore block ordinary trade and relocation while continuing to convey visits,
money and cultural contact.

## One basic passability rule

`Route::passable()` requires political openness and zero flood-closure months.
Route-cost queries, land path searches, monthly overland exchange and relocation
arrivals retain their existing behavior using this common predicate. Pilgrimage,
personal charitable gifts and annual overland cultural contact now use it too.
Existing additional requirements remain: provisions and reserved work for visits,
witnessed-appeal policy for relief where enabled, and cultural contact thresholds
for learning across different traditions. Basic road passability is not a substitute
for separate war, capacity, destination or affordability checks.

A blocked pilgrimage spends neither provisions nor offerings. A blocked personal
gift does not debit the donor or credit the recipient. Annual road contact cannot
accumulate through the blocked edge, so its knowledge and religious-transmission
opportunities are withheld. Reopening restores eligibility rather than granting
an automatic visit, gift, conversion or lesson.

## Preserve completed journeys as evidence

Completed market deliveries from the last twelve months remain a separate source
of annual contact. A road closing today does not erase an arrival already witnessed,
and a shipping route can provide contact even when an overland connection is closed.
The existing reverse-contact convention remains. Arrivals exactly twelve months
old or older fall outside this window.

Delivery-based knowledge events now link to the most recent qualifying arrival
for the relevant directed pair, or the reverse pair where needed, alongside any
recorded teacher source. Stable chronological iteration and ordered maps preserve
determinism. No extra world state or archive fields are required.

## Controlled checks

The GPU-backed history fixture holds household and settlement faith constant to
isolate transport from the existing cross-faith contact delay. It checks:

- Flood closure blocks new knowledge contact and accumulated contact years.
- A passable matched route permits contact; reopening the closed route restores it.
- A recent completed arrival permits learning despite present closure and appears
  in the lesson's causal references; a stale arrival does not.
- A personal relief gift stays blocked during flooding, then transfers three
  existing money after reopening without changing total town cash.

The pilgrimage fixture separately verifies unchanged food and offerings while
blocked and successful finite travel after reopening. An ordinary predicate test
covers both political states and zero, one and twelve months of flood closure.
Existing society, culture and relocation tests cover route continuity, closure
history, accounting and checkpoint continuation.

```sh
mise exec rust@1.89.0 -- cargo test --lib culture::practices::tests -- --include-ignored --test-threads=1
mise exec rust@1.89.0 -- cargo test --test society --test culture -- --include-ignored --test-threads=1
mise exec rust@1.89.0 -- cargo test --lib relocation_conserves -- --ignored --test-threads=1
```

## Limits

This is a correction to how existing systems use closure state, not a new travel
simulator. Annual overland contact still samples road availability at its update
boundary rather than integrating monthly visitor traffic. Cultural diffusion can
still cascade within an annual update. Small personal monetary gifts remain
immediate abstract transfers over eligible roads, without a cash-courier cohort.
The controlled test does not establish population effects or a calibrated rate of
cultural isolation. Monthly exposure histories, explicit visitor journeys and
snapshot-based diffusion remain possible later extensions.

## Recorded validation

50 ordinary tests and 18 explicitly executed GPU tests passed, including six
cultural-practice fixtures, nine culture integration fixtures, two society fixtures
and conserved household relocation. Formatting, whitespace checks and Clippy with
warnings denied passed. Hardware: Quadro RTX 5000 Max-Q, Vulkan.

The isolation fixture uses seed 17 at terrain and ecology resolution 64, after one
geological epoch and sixteen foundings. It compares controlled branches rather
than fitting a multi-seed historical outcome. [Logs and source checksums](evidence/transport-contact/)
retain the successful final-source runs. Earlier development runs exposed a fixture
mismatch between household faith and site labels; both are controlled consistently
in the retained comparison.
