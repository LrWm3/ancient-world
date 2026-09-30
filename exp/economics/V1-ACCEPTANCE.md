# V1-01: household offer acceptance

Explicit common-offer bundles now carry citizenship, land access and process
starts through the household's existing before/core/after envelope. The person
remains the signatory and operator. Collective policy grants resources before
candidate work resolves; supplying a candidate cannot rewrite that policy.
Acceptance previews the next Productive boundary before publishing any part of
the package. It spends no seed or labor until ordinary dated execution.

One-plot allocation still uses `PriorityLottery`, with seed 7 in the control.
The household fallback retains all members and uses fixed individual priorities.
It cannot use the former isolated-person projection, which removed constituents
required by household validation. Allocation receipts are checked against the
original opening household before core transfers, rather than against already
distributed resources.

## Verification

From `exp/economics`, Rust 1.92.0, locked dependencies:

```sh
cargo +1.92.0 test --locked --test household_offers --test process_offers \
  --test competition --test household_farm_finance --test financial_offers \
  --test recovery_search
```

Initial affected gate: **30 passed**, no failures. The strengthened storage control
also passed in the five-test `household_offers` target. Raw output is local under
`output/economics/v1-acceptance.log`; final candidate verification is recorded
separately by the v1 release runner.

The new two-person/two-plot case runs 24 months on CPU and reference, resumes just
after prerequisite acceptance, and compares state, the remaining ledger and
separate accounting. It completes four crops and pools exactly 16 grain. Seed is
returned and allocated for repeated planting; household collection occurs once.
Five hours per person make the 20% contribution observable (one hour each), with
40 storage units per member. The household begins with two seed, ten grain and
four coins, in addition to the existing individual fixture endowments. Annual
native land dues remain separate member claims. Existing
`household_support_funds_member_dues_in_native_goods_or_coins_once` verifies the
shared cash/native payment path; a membership or work request itself has no coin
price.

Controls cover missing cultivation permission, insufficient shared seed,
overcommitted personal labor, insufficient shared storage, forged household
transfers, forged nested work receipts, stale opening seed and replay. No rejected
package publishes citizenship, rights, work or financial state. The one-plot
control awards one applicant, preserves the loser's lack of a land claim and
allows ordinary wood-collection fallback. With both adults unavailable in month
2, both crops abort and the two planted seed remain sunk.

This closes the bounded explicit-acceptance gap. Household `ConsequenceAware`
search, mixed financial/productive search bundles, collective cooperative credit
and autonomous joint prerequisite discovery remain unsupported. The relevant
validation guards remain in place. This result alone does not certify v1.
