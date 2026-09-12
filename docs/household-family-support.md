# Local family support and food access

This opt-in policy connects known family relationships to existing household
wallets. It is a game rule for voluntary support, not a model of universal family
obligations. It does not increase yields, create food, spend vacant estates, or
replace household ownership with a communal balance.

## Monthly boundary and allocation policy

`HouseholdEconomy.family_support = Some(FamilySupportPolicy::default())` enables
sharing; new and older worlds default to `None`. The balance runner exposes
`--family-support`. Default policy values are a 25% monthly surplus share and a
90% recipient dietary target. Both values must be finite fractions in 0–1.

In Reserve, before mutable household payroll/retail preparation, the system
captures potential links from the current participation registry, known parents,
and expressed relationships. Resident adults (18+) can offer their household's
funds to parents, children or siblings in another household at the same site.
Missing ancestors never establish kinship; absent/dead helpers, different towns,
negative relationships and zero generosity restrict eligibility. Multiple members
of one family do not each get to spend its full wallet: the strongest willing
member establishes a single directed link per household pair. This is an abstract
household consent rule; internal disagreement is not modeled.

After current payroll and dividends, but before council relief:

1. Protect each donor's full current private food budget, after common access.
2. A donor can offer a policy fraction of the cash above that reserve, scaled by
   the strongest eligible generosity × relationship weight.
3. Divide that offer across eligible kin, weighted by relationship strength and
   their opening cash shortfall below the dietary target.
4. Cap combined incoming offers at each recipient's shortfall. Do not reoffer
   received money, and do not refill capacity released by recipient caps this month.
5. Transfer existing cash once, retaining dated household-to-household receipts.
   Council relief sees the smaller remaining deficits. Retail then rebuilds its
   funded food cap from the actual resulting wallets.

Proposals use one completed wallet snapshot and canonical pair ordering. The
policy does not let the first visited town or relative exhaust a shared budget.
A receipt records household IDs, site, amount and a shared monthly date. Repeating
family settlement at the same boundary cannot spend a second allowance.

Physical food availability still caps consumption. The policy can improve access
when food exists, but cannot remedy a harvest failure by printing entitlements.
A protected cash budget is not a reservation of physical food under scarcity.
Gifts are local accounting transfers, with no additional travel or work assumed;
cross-town remittances still require a separate transport/service mechanism.

## Shared relationships and accounting

`kin_support::support_affinity` is the unchanged ancestry/estrangement function
previously used only by neighbor care. Care and cash support now use the same
weights: parent/child 0.75, siblings 0.5, with expressed hostility weakening the
potential obligation. Cash sharing additionally requires generosity and known kin;
it does not inspect all pairs of residents or manufacture new relationships.

Household accounts track cumulative family cash sent and received. Per-account
cash ledgers include both, and the world requires their sums to balance. The
existing money residual still observes all wallets and public accounts. Saved
histories preserve the policy, latest receipts and cumulative transfers. Old
histories deserialize zero transfer totals and keep sharing disabled.

Cell/household inspection reports lifetime gifts received and given. Balance
reports separate family transfers from council relief and wages, so private
support cannot be misreported as new earnings or public expenditure.

## Verification and balance

The regular library suite passes (129 tests; 112 GPU/extended tests are ignored
by that command). The separately executed Vulkan causal fixture passes: real
parent/child households share existing cash, funded food access rises, and
physical food and total money remain unchanged. Hostile, absent and cross-town
controls remove the support link. Donors retain their private food budget.
Checkpoint continuation and three batched versus three single months agree.
Analytical fixtures cover competing donors, recipient caps, ordering, no
same-month relay, disabled policy and once-only settlement.

Both matched thirty-year histories complete. All retain sixteen active sites,
zero maximum population residual and maximum food residual below 2.6e-7.

| Seed | Population control → support | Cumulative access gap, % need | Family cash transferred | Donor / recipient households | Household council relief control → support |
|---|---:|---:|---:|---:|---:|
| 17 | 1,510 → 1,709 | 3.1972 → 2.6578 | 66,420.00 | 168 / 144 | 4,847.62 → 4,605.84 |
| 81 | 1,547 → 1,767 | 3.3144 → 2.5872 | 57,025.47 | 180 / 163 | 3,439.30 → 2,183.67 |

Transferred cash counts once, not sent plus received; their cumulative totals
agree within 2e-11. Counts are lifetime household participation, not monthly
recipients. Seed 17's absolute physical food gap remains 4,604.71; its share of
need falls from 0.0456% to 0.0440% because demand grows. Seed 81 has no physical
gap in either arm. The intervention improves purchasing access rather than
removing physical shortages. Later production, earnings and population can diverge
through the existing feedbacks; the controlled GPU fixture isolates the immediate
cash-to-access mediator.

Council-to-household relief falls in both seeds, but private transfers exceed that
reduction. This is not solely replacement of public aid. Broader council spending
also changes and must not be confused with household relief. These are two small
worlds over thirty years, not held-out century validation. The policy remains
opt-in. All-target Clippy with warnings denied also passes.
The population-access gate remains open; a new redistribution channel alone is
not proof that long-run population decline is repaired.

Reproduce the thirty-year comparison on the Quadro RTX 5000 / Vulkan backend:

```sh
cargo build --example cultural_work_calibrate
target/debug/examples/cultural_work_calibrate \
  --seeds 17,81 --years 30 --resolution 32 --crop-yield-scale 0.5 \
  --individual-demography --workshop-refinement --agriculture-refinement \
  --extraction-refinement --construction-refinement --compare-resolution \
  --household-diagnostics --output output/family-support-base.json
```

Repeat with `--family-support --output output/family-support-pilot.json`.
Both arms use ecology 16, one epoch, sixteen founders and living history. Common
access, council relief, production and nutrition settings are held fixed. Runs
were concurrent for behavioral comparison; their elapsed times are not a measure
of policy overhead. Only human-readable summaries are committed.

```sh
python3 scripts/compare_food_access.py output/family-support-base.json \
  output/family-support-pilot.json --allow-difference family_support
```
