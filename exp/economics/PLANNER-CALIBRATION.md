# Planner forecast calibration and market-order adapter

Status: bounded follow-up to [the first comparison](PLANNER-COMPARISON.md).
Both searches and all new scoring variants remain opt-in. The production default
and monthly scheduler are unchanged. This is additional experimental verification,
not a rerun of the full [v1 release gates](V1-RESULTS.md).

The forecast reproduces its fixed continuation exactly, but monthly replanning
can produce substantially different harvest dates and shortages. Including pooled
household buffers changes choices without consistently improving outcomes. A new
market adapter successfully composes wood production, coin sales and food purchases
in a deliberately small, finite market; the existing fixed policy also succeeds.

## Matched forecast and execution

`examples/planner_calibration.rs` reuses B3, B5 and B5-one-seed without retuning
their resource quantities. Each search has 256 expansions, 32 complete forecasts,
and a 6-, 12- or 24-month horizon. Beam width remains eight. For each opening:

1. Select a package and record its fixed-policy forecast.
2. Accept the same package and execute that fixed continuation. Assert identical
   ending state and monthly reports against the forecast.
3. Accept the same package and reconsider at every subsequent Acquire boundary.
4. Compare outcomes over **the same interval as the opening forecast**.

Thus the six-month rows below cover six actual months, unlike the previous report's
24-month runs using shorter rolling horizons. There are 54 configurations: three
fixtures, two searches, three horizons and three scoring variants. All fixed
execution/forecast equality assertions passed.

Observations include food/warmth deficits, dated process completions, ending pooled
holdings, native-resource land arrears and the consequence score. The runner also
counts changes in requested packages. This count includes ordinary stage/start
changes; it is not a measure of unnecessary plan abandonment. Accepted obligations
and paid amounts remain in the ordinary reports and ledger.

Selected results using the original private-buffer score:

| Case / search | Months | Forecast crops / food deficit | Replanned crops / food deficit | Other evidence |
| --- | ---: | --- | --- | --- |
| Competing crops / either | 6 | 1 / 0 | 1 / 0 | Same completed crop count |
| Competing crops / either | 12 | 2 / 0 | 1 / 0 | Forecast harvests in months 6 and 12; replanning completes only month 6 |
| Competing crops / either | 24 | 4 / 0 | 3 / 0 | Ending buffer gap grows from 166 to 1,666 |
| Two-seed household / beam | 12 | 1 / 0 | 2 / 0 | Ending pooled grain 4 versus 12 |
| Two-seed household / best-first | 12 | 2 / 0 | 3 / 0 | Ending pooled grain 8 versus 16 |
| Two-seed household / beam | 24 | 3 / 6 | 5 / 2 | Replanning improves this forecast |
| Two-seed household / best-first | 24 | 4 / 0 | 4 / 6 | Forecast aborts two crops; realized execution aborts none |
| One-seed household / either | 24 | 3 / 6 | 3 / 8 | Both paths retain zero land arrears |

The best-first two-seed household at 24 months is particularly instructive.
Its forecast harvests twice in month 7 and twice in month 13. Replanning instead
harvests in months 12, 14, 18 and 24. Equal ending crop counts conceal a materially
different food supply schedule. The realized path avoids the forecast's two
aborts but develops six units of food deficit. Warmth deficits and ending land
arrears are zero throughout this calibration matrix.

This isolates a policy mismatch: reproducing the same opening choice and fixed
continuation reproduces the forecast. It does not prove that recursive replanning
is the only cause of poor choices, or that running recursive search inside every
forecast is the appropriate fix. Search pruning and horizon effects remain.

## Household buffer ablation

`composition::choose_with_scoring` accepts three experimental options:

- `PrivateBuffers`: unchanged comparison score, using individual holdings.
- `HouseholdPrivateBuffers`: use the existing household consumption projection,
  but hide collective stocks from that projection.
- `HouseholdBuffers`: the same projection with collective stocks available once
  through the ordinary household allocation policy.

The last two form the controlled pooled-stock comparison. The intermediate
control matters: household consumption also respects permissions, private claims
and allocation rules, whereas the legacy private-buffer calculation has different
coverage semantics. Neither variant transfers real property or changes membership
or consent. Last month's fulfillment is cleared before projecting future coverage;
food already eaten cannot become stored food. The score remains subordinate to
terminal states, impairment, deprivation and broken commitments.

For equal-demand members the coverage scale matches the original per-person scale.
For unequal demands this pilot weights unmet quantities within each provision;
individual fairness is not separately optimized. The existing consumption helper
abstracts away storage for this valuation; real settlement still checks storage.
This is a static consumption-buffer estimate, not a general terminal balance-sheet
valuation or a promise that future obligations are all funded.

| Two-seed household | Private / household-private control | Household stocks included |
| --- | --- | --- |
| Beam, 6 months | 0 crops, 0 food deficit, pooled grain 8 | Same |
| Beam, 12 months | 2 crops, 0 deficit, pooled grain 12 | 2 crops, 0 deficit, pooled grain 11 |
| Best-first, 12 months | 3 crops, 0 deficit, pooled grain 16 | 2 crops, 0 deficit, pooled grain 11 |
| Beam, 24 months | 5 crops, 2 deficit | Same |
| Best-first, 24 months | 4 crops, 6 deficit | Same |

The one-seed household's crop counts and food deficits also remain unchanged
across these scoring variants. No pooled-buffer variant is promoted. A more
complete view of reserves alone does not repair the continuation mismatch.

## Executable market-order adapter

`town_market::OrderSelection` names one actor and the `(market, side)` orders it
wants to submit. `evaluate_selected` derives the ordinary eligible need orders
and withholds the actor's unselected ones, retaining `PlannerWithheld` receipts.
Other participants retain their ordinary orders. It cannot invent an eligible
order, change a quote, waive admission or spend a counterparty's holdings.

The experimental composition search now includes these order choices alongside
current process offers. `composition::market::prepare` supplies the validated town
receipt to the existing offer preparer, which checks and reserves work against the
post-acquisition state. The selected batch publishes through ordinary settlement;
settlement independently regenerates its market receipt and rejects alteration.
The dated snapshot check rejects stale selections.

Timing remains:

- Open captures marketplace admission.
- Acquire observes one opening book, reserves actual trades and publishes them
  with the selected dated productive plan.
- Productive executes that plan; its outputs become stock after the book closes.
- Consumption and Close retain their existing timing.

**Incoming sale proceeds cannot finance another trade in the same book.** Newly
produced wood cannot be sold before it exists. A productive sale can instead fund
a later month's food purchase. Prices, finite cash, stock protection, storage,
matching limits and legal eligibility remain the existing matcher's responsibility.

The current search scope is one planning person and passive counterparties with
zero productive capacity. Only the person's outcomes determine its score. Future
counterparty need orders are simulated from the finite observed state using their
ordinary fixed policy; this is a declared full-information hypothesis, not a
guarantee of future demand. Current submission choices do not reserve future fills.

Still excluded from this adapter:

- Active counterparty production planning, household market mandates and finance
  drivers; existing separate drivers for those systems continue to work.
- Simultaneous land/membership acceptance and town orders. Town validation still
  excludes land-offer discovery, and this preparation path accepts process offers.
- Arbitrary input-procurement bids, chosen order sizes/prices or dated future orders.
  It selects subsets of existing whole-lot need orders at supplied quote policies.
- General backward search through exchange chains. The demonstrated wood output
  is already relevant to the person's warmth need; forecasts reveal its sale value.

## B4b: a viable finite wood/food route

This is a new fixture, not a retuning or replacement of frozen B4. B4's original
four active producers remain outside this composition adapter's supported scope.
B4b uses the existing grain/wood market, consumption and process machinery:

- Person 88 starts with three grain, two wood and zero coins; needs one food and
  one warmth per month. One labor hour can produce three wood. Farming is disabled.
- Person 89 has zero productive labor, 100 grain and 100 coins, and needs one food
  and one warmth monthly. It sells grain and buys wood. Its inventory is finite.
- Both market lots are one unit at one coin, with a two-month buy/reserve horizon,
  fixed registered sides and 200 storage capacity per person. The planning person
  has the existing illustrative deprivation consequences. The passive merchant's
  needs generate demand without a mortality model in this fixture.

`examples/planner_market.rs` runs 24 actual months for fixed continuation, beam
and best-first, with 6/12/24-month search horizons and the same 256/32 budgets.

| Control | Result for all three policies and all horizons |
| --- | --- |
| Counterparty available and funded | 23 wood sold, 22 grain bought, zero food/warmth deficit, person survives, ending coins 1 |
| Wood match cap zero | No trades or income; food deficit 6, person becomes terminal |
| Counterparty starting coins zero | No trades or income; food deficit 6, person becomes terminal |
| Counterparty outside opening market reach | No trades or income; food deficit 6, person becomes terminal |

The cap-zero control retains orders but forbids completed wood fills; it is not
claimed to remove only a buyer. The distance control removes both the merchant's
bid and ask access. Deficits stop accumulating after the person's terminal state,
so six is not an estimate of demand over all 24 months. The successful route
preserves 100 total coins. Both searches evaluate 182 complete candidate rollouts
over the 24 positive-case boundaries; neither beats the fixed policy here.

## Verification and next boundary

100 tests passed across composition, calibration, town markets, production markets,
cooperation, shared acquisition, household markets/offers, process/financial offers,
forecast context, planning and search. The new six tests cover pooled-stock
accounting and expired fulfillment; exact fixed forecast execution versus replanning;
sale proceeds timing, tampered receipts and stale terms; withheld/reordered orders
and unsupported mandates; finite demand controls; and CPU/reference equality with
checkpoint continuation and separate double-entry books. Final focused reruns cover
the market and calibration changes. Strict all-target Clippy and formatting pass.
The full release suite and ignored population stress test were not rerun.

From `exp/economics`, redirect generated outputs under root `output/economics/`:

```sh
cargo +1.92.0 run --locked --release --example planner_calibration
cargo +1.92.0 run --locked --release --example planner_market
cargo +1.92.0 test --locked --release --test composition_market --test planner_calibration
```

The next useful experiment is a bounded continuation-policy comparison: forecast
the same work-selection policy the agent will actually use, or explicitly retain
and repair a dated plan. Compare supply timing and shortages before expanding the
catalog or increasing search budgets. This remains a proposal, not implemented work.
