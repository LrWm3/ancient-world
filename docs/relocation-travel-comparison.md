# Monthly relocation attrition comparisons

Relocating households already carry finite food, money, tools and age cohorts.
Known travelers share person identities with their home communities. Monthly
attrition previously updated those stocks without a corresponding aggregate versus
individual outcome receipt.

## Boundary and forecast

Open captures each funded journey before consumption. Age-weighted monthly need is
`10 * children + 18 * adults + 14 * elders` kg food equivalent. Predicted consumption
is the smaller of need and carried food. When the shortfall exceeds the existing
0.001 kg tolerance, expected mortality is `0.08 * (1 - eaten / max(need, 0.001))`;
otherwise it is zero. Expected deaths are opening population times that rate.
This describes the existing toy travel rule, not an empirical mortality model.

The existing update then consumes food and resolves either fractional cohort losses
or person-specific deaths with fractional anonymous losses. Observations report
actual food consumption, population lost and remaining travelers after attrition.
Existing cleanup of cohorts below 0.01 people is included in actual losses. These
survivors may subsequently arrive, be delayed or start returning in the same Open
phase; the metric is not an end-of-month count of people still on the road.

Receipts use `System::RelocationTravel`, grouped by origin and actual resolution
mode, at most two rows per site regardless of journey count. A roster under the
individual-demography switch uses Individual; rosterless/legacy cohort updates use
Aggregate. Expected deaths remain the aggregate conditional mean in both cases.
Individual sampling and the tiny-cohort cleanup can differ from that mean; those
differences remain visible rather than being relabeled as explained away.

Opening fingerprints include seed, traveler household/age/food inputs and roster
identities. Current-month comparison checks occur before any travel mutation.
Repeating a recorded origin/mode update is rejected before spending food again.
The coordinator propagates this error. Without resolution reporting, the original
travel behavior remains unchanged. This is not a general scheduler rollback.

No forecasts debit supplies or population. Only the existing update commits those
transfers. Latest-month receipts and cumulative summaries use the existing archive
and report interface; the receipt count bound expands by two per site. No separate
per-person history series or unbounded per-journey result list is added.

Departure/admission choices, expected arrival dates, migration preference comparisons,
warfare recruitment/casualties and campaign outcomes still need separate treatment.

## Verification (2026-09-12)

Quadro RTX 5000 / Vulkan, development/test profile.

- The hardware relocation fixture passes for a valid funded journey under both
  aggregate and individual travel-loss modes, with full provisions and zero food.
  The no-food conditional death expectation equals 8% of opening population;
  predicted food consumption equals actual consumption; actual deaths plus
  survivors equal the opening population. Aggregate deaths agree within 1e-5.
- Every arm produces identical physical history with reporting enabled or disabled
  after removing only the resolution report. Saved opening-state continuation
  reproduces the entire result. A repeated comparison-enabled update is rejected
  with the complete state unchanged.
- The blocked-journey fixture runs 200 monthly observations with serialized
  continuation. Travelers eventually die; accumulated actual travel deaths equal
  the original population within 1e-5. Existing population, food, estate, identity,
  route reopening, return and witnessed-relief checks remain in the same test.
- Regular library suite: 127 passed, 110 hardware tests skipped. The hardware
  relocation command above is separate. The three-seed full frozen scheduler test
  also passes monthly/batched/checkpoint equivalence; the controlled relocation
  fixture supplies guaranteed traveling-population coverage.

These tests verify reporting and conservation boundaries, not the credibility of
8% maximum monthly attrition or a migration policy. No balance parameters changed.

All-target Clippy passes with warnings denied. Artifact-policy and whitespace checks
pass; generated logs remain under ignored `output/`.
