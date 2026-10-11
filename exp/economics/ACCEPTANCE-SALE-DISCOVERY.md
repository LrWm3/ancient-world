# Sale-linked production acceptance

This ten-iteration follow-up tests whether signed sale opportunities can replace
inventory objectives as the reason to discover productive work. It builds on
[bounded peer expectations](ACCEPTANCE-PEER-EXPECTATIONS.md). Initial identities,
consent, prices, employment and physical endowments remain supplied.

## Iteration record

1. **Signed issuer-input sales.** The existing `stock_markets` authority and
   `PeerStockQuotes` program now also support stock sales into issuer-input markets.
   They route to the existing mint counterparty book; independent stocks retain
   the peer book. Only signed issuer-input slots are replaced, including explicit
   stops; unrelated legacy quotes and other agents remain unchanged. As before,
   the actor's complete peer portfolio is replaced. Capacity sales and input-market
   buys are excluded. Two atomic validation/routing tests and nine existing
   authority, discovery and execution tests pass. This adapter still requires an
   enabled peer-stock book; it is not universal market authority.

2. **Dated execution and access.** A signed combined ore-purchase/metal-sale
   program is selected through ordinary forecasts, becomes effective at the next
   Open, and produces real sales on both reference and CPU. A matched legal-access
   revocation suppresses sales while retaining accepted history, installed quotes
   and employment obligations. In-memory reconstruction preserves the boundary.

3. **Sale-linked productive discovery.** Signed stock-sale options whose payment
   could meet an organizational cash-reserve or funding-gap objective now root
   productive alternatives. A target is the signed floor plus the authorized lot
   count times listing quantity, using checked arithmetic. Only signed productive
   recipes can supply it. Explicit inventory objectives retain their original
   targets; otherwise recurring work replaces the one-shot candidate and combines
   with existing quote portfolios. Catalogs depend on static options, never live
   balances or quotes. Two new tests and five discovery/execution regressions pass.
   A target is a persistent stock buffer, not a promise of demand or a rule that
   automatically stops work when the cash goal is reached.

4. **Cash-only cold-start control.** A new fixture removes workshop/town
   inventory objectives and the initial workshop metal ask, while retaining
   signed price options and finite endowments. Its 48-month reference run fails:
   zero mints and productive batches, 67 food deficits and ending wage arrears 2.
   All first workshop cash projections end at zero, so the normal retain-on-tie
   rule selects no program. Sale-linked candidates exist, but observed-recurrence
   expectations have no successful demand history to repeat. This is an explicit
   negative control, not a passing acceptance claim.

5. **Bounded catalog and compatibility controls.** Unsigned productive recipes
   cannot become sale candidates. Oversized signed production catalogs reject
   initial installation atomically instead of silently truncating alternatives.
   Explicit inventory objectives retain their original target rather than taking
   the derived sale buffer. Four sale-discovery and two joint-authority tests pass,
   including arithmetic overflow and historical catalog stability.

6. **Finite startup comparison.** Monthly review and an initial metal ask alone
   yield one mint and no recurring work. Four opening metal units yield four
   mints and two workshop batches, but no town production: the chain still stops.
   Supplying four opening ore units and an initial signed town ask as well yields
   44 mints, 43 workshop batches and 40 town batches over 48 months, with no food
   deficits or wage arrears. The unchanged governed readiness check passes all
   three post-startup years. These are finite opening inventories and posted
   quotes, not supplied work targets or future purchase decisions. The cash-only
   cold fixture retains annual reviews; all posted-startup comparisons use monthly
   reviews, with workshop/town forecast horizons still 12/4 months.

7. **Sale provenance in decision observations.** Discovered decisions record the
   signed market/recipe/lot terms behind productive alternatives. Each private
   projection records only accepted market sales, with month, buyer, resource,
   quantity, payment and value. Existing organization observers serialize these
   records without injecting telemetry into economic execution. A filtered month-5
   observer pins the two-metal/four-coin sale tuple, hypothetical peer work/ballots,
   forecast dates and identical live world/state/ledger/accounting. That test and
   eleven existing expectation/telemetry controls pass. These are private forecast
   receipts from the current mint/peer market batches, not promised live income or
   coverage of every financial arrangement.

8. **CPU continuation and causal replay.** The cash-only successful startup joins
   the shared 48-month continuation harness. Reference and CubeCL CPU agree on
   world, state, ledger, reports and accounting, including CPU reconstruction after
   every phase and replay with dated world inputs. Independent catalog reordering
   preserves results. The first accepted workshop choice is month 5, effective 6;
   the town's is month 8, effective 9. All nine continuation tests pass, retaining
   the existing failure and interruption checks for earlier variants. This is
   in-memory reconstruction and dated-context replay, not a durable checkpoint or
   proof that ledger records alone reconstruct governance.

9. **Failure controls against the successful startup.** Absent, satisfied and
   non-crossing issuer buyers must not become recurring demand hypotheses or
   produce later sales. Missing ore supply, replacement orders, metal access,
   operating margin and timely payroll remain readiness failures. These controls
   use the same finite opening inventories and quotes as the positive treatment;
   they do not merely compare already-failing cold-start economies. Reference and
   CPU world/state/ledger/accounting/readiness are compared for all eight controls.
   All eight controls fail readiness with matching CPU/reference results. The
   late-payroll case ends with zero arrears and no food deficits, but still fails
   specifically because wages entered arrears: later payment does not erase it.

10. **Runnable comparison and scope documentation.** The acceptance example adds
    `--sale-discovery`, its existing `--continuation-matrix` combination, and
    `--sale-startup-matrix`. README, integration status, operating authority and
    forecast documentation distinguish the successful bounded treatment from
    cold startup. The standalone 48-month CubeCL CPU treatment passes. Existing
    default scenarios and `AcceptedOnly` expectations remain unchanged.

## Startup results

All counts cover 48 months. These are reference runs; the successful treatment
also passes the standalone CPU run and CPU/reference continuation checks.

| Initial treatment | Mints | Workshop batches | Town batches | Food deficit | Ending wage arrears | Ready |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| Cold, no initial workshop metal ask, annual reviews | 0 | 0 | 0 | 67 | 2 | No |
| Metal ask, monthly reviews | 1 | 0 | 0 | 61 | 2 | No |
| Metal ask + four workshop metal units, monthly reviews | 4 | 2 | 0 | 37 | 2 | No |
| Both asks + four workshop metal/four town ore units, monthly reviews | 44 | 43 | 40 | 0 | 0 | Yes |

The successful fixture retains four initial workshop ore units from the existing
baseline. The listed metal/ore stocks are opening endowments, booked at setup;
they are never replenished by the fixture during execution. Initially posted
quotes respect signed lot caps: one metal lot and two ore lots per boundary.
Tests pin the sole organizational COIN-reserve objective, absence of preference
overrides, discovery mode, empty supplied program menus and no initial workshop
or town work targets.

In each complete post-startup year, workshop and town complete 12 batches each,
24 ore units are delivered, and each employer pays 12 wage coins. Workshop net
income is 12 coins/year; town income is 13. Their closing cash grows from 18/8 at
month 12 to 54/47 at month 48. The first selected work/trade programs become
effective in months 6 and 9 respectively. Both existing household delivery units
settle on time, the state loan is repaid, and nine separate books reconcile.

## Follow-up

The [stocked-offer publication experiment](ACCEPTANCE-OFFER-PUBLICATION.md) now
compares this finite-inventory treatment without supplied workshop/town asks.
It adds an opt-in exact-tie publication policy; the default is unchanged. The
startup results above remain the original supplied-ask controls.

## What remains open

A signed sale opportunity establishes an admissible alternative, not a buyer.
The cold case exposes the remaining coordination gap: conservative cash forecasts
can reject every new productive commitment before demand history exists. Supplying
only a metal outlet, or working inventory at only that stage, does not sustain the
whole chain. The successful result establishes recurrence after a finite startup;
it does not solve endogenous outlet publication or startup finance.

Prices, market lots, stock floors, purchase targets, employment, consent,
organization identity and existing household/state policies are still supplied.
The production target is a bounded buffer derived from signed sale terms; it is
not optimized inventory management or a rule that stops at cash-goal satisfaction.
Town ore production uses the existing abstract recipe. Full-world information,
12/4-month horizons and bounded observed peer process/state-ballot recurrence
remain explicit forecast assumptions. There is no new price learning, hiring,
formation, self-directed personal policy change, durable restart, unforeseen-shock
recovery claim, or evidence of persistence beyond the measured 48 months.

Voluntary initial publication is now tested in the follow-up above. Further
startup coordination should preserve refusal, funding limits and ordinary
settlement, using these cold and partial-startup failures as controls without
supplying future demand or weakening readiness.

## Run

From `exp/economics`:

```sh
cargo +1.92.0 run --locked --release --example acceptance -- --sale-discovery
cargo +1.92.0 run --locked --release --example acceptance -- --sale-startup-matrix
cargo +1.92.0 run --locked --release --example acceptance -- --sale-discovery --continuation-matrix
cargo +1.92.0 test --locked --release -j8 \
  --test governed_input_sales --test sale_discovery --test acceptance_sales \
  --test sale_startup_comparison --test acceptance_sale_controls \
  --test sale_telemetry --test acceptance_continuation
```

## Verification

**161 targeted tests passed**: 20 library tests and 141 integration tests across
29 suites. This includes the three startup assertions, eight CPU/reference failure
cases grouped into two tests, all nine continuation tests, the twelve observer
checks, and regressions for authority, discovery, organizational programs, markets,
forecasting, state/household integration and prior acceptance variants. The sale
observer's exact receipt tuple was also rerun after strengthening its assertion.
The standalone CubeCL CPU positive scenario passed. All-target release Clippy with
warnings denied, formatting and repository artifact checks passed. Generated logs
remain under ignored `output/`; no binary or raw run data is committed.

Independent review found no blocking issue. Its requested cash-only/no-supplied-work
assertions and explicit startup/forecast qualifications are included. These checks
establish this bounded integration, not all repository tests or all future agent
and market combinations.
