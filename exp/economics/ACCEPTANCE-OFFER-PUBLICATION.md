# Stocked offer publication acceptance

This follow-up removes supplied opening workshop/town asks from the
[sale-discovery experiment](ACCEPTANCE-SALE-DISCOVERY.md), while retaining signed
terms, finite initial inventory, employment, identities and consent. The default
controller continues to retain its plan on equal forecasts.

## Iteration record

1. **Opt-in stocked publication.** `StockedAsksOnEqualProjection` selects an
   existing quote-only alternative only when retaining is the best valid forecast
   and the governing loss vectors are exactly equal. It requires current stock
   above the signed floor and protected claims, an unmet payment objective and
   lawful organizational authority. It adds only asks, preserving every replaced
   live quote, and uses normal next-Open acceptance. It creates no purchase,
   production, balance or counterparty decision. The new policy is sealed with
   the existing configuration. The default/equal/worse/invalid projection test
   passes. Publication eligibility does not itself guarantee later execution.

2. **Dated publication and independent demand.** A five-month reference/CPU
   comparison begins with no workshop/town asks and four metal/four ore opening
   units. Both governors choose quote-only publication in month 1, effective at
   month-2 Open. Month 1 has no sale or stock mutation from publication. Subsequent
   independently selected minting produces real metal sales. World, state, ledger
   and accounting match across backends. This short test establishes timing and
   actual demand, not sustained readiness.

3. **Publication safeguards.** Five selector tests cover exact ties, the current
   governor's payment goals, existing buy/ask preservation, prohibited new buys or
   work bundles, insufficient whole lots, signed floors, accepted delivery claims,
   revoked trading permission and a vacant office. These are eligibility controls;
   later clearing must independently protect stock and recheck participation.

4. **Issuer-input claim protection.** Fixed/generated input-stock asks now protect
   accepted claims at Acquire using the same reader as peer stock asks, including
   claims added after an earlier plan evaluation. Protection uses the longer of
   the configured private/peer horizons, with a one-month fallback; the signed
   floor also applies. Four boundary tests cover a six-unit stock/four-unit claim,
   actual funded CPU/reference settlement leaving four units, horizon/floor limits
   and incomplete-package rollback. Claims in these tests are installed accepted
   history, not a test of voluntary forward admission. Existing mint-order and
   peer-order tests pass. The legacy provisioning adapter rebuilds its own asks;
   it now discards the superseded fixed-route receipts. Its separate provisioning
   policy is unchanged and is not covered by this new stock-protection guarantee;
   all 15 provisioning regressions pass.

5. **Matched acceptance fixture.** `acceptance::publication` removes both supplied
   organization asks, retains the exact finite inventories and signed options,
   and enables only the new publication policy. A structural test compares whole
   initial worlds/states after changing just that field, and pins cash-only goals,
   empty supplied program menus and absence of organizational work orders/asks.
   The measured reference comparison produces 44 mints, 43 workshop batches and
   40 town batches with publication, versus zero of each with retention. Readiness
   passes only with publication; food deficit/ending arrears are 0/0 versus 67/2.
   The initial forecasts contain no hypothetical peer work. Later forecasts still
   use the existing observed-process/state-ballot recurrence model; this experiment
   does not remove that assumption.

6. **Governance timing and sealed configuration.** Two execution tests verify
   that the new choice obeys three-month review dates and six-month minimum tenure,
   and that changing its sealed mandate after first Open fails without changing
   state or decision history. A publication accepted in month 1 still becomes
   effective in month 2; no extra policy review or early execution is introduced.

7. **Partial-inventory controls.** With monthly reviews and voluntary publication,
   the original small workshop stock produces one mint and no workshop/town work;
   increasing workshop metal alone to four units produces four mints and two
   workshop batches, but no town batches. Food deficits remain 61 and 37, with two
   ending wage arrears in both cases. Publication cannot replace missing working
   inventory elsewhere in the chain. `Startup::PostedMetal` is an inherited fixture
   label: this variant removes that supplied ask too. These two controls and the
   successful treatment all use monthly reviews.

8. **CPU continuation and replay.** The 48-month scenario passes the existing
   continuation harness with CPU/reference world, state, ledger, report and audit
   equality, reconstruction at every CPU phase, dated event replay and independent
   catalog reordering. Both organizations first accept publication in month 1;
   later productive choices and leadership rotation still satisfy the unchanged
   governed-readiness check. This is in-memory reconstruction, not a new durable
   checkpoint format, and tests three complete post-startup operating years.

9. **Buyer and economic failure controls.** Eight controls compare complete CPU
   and reference world/state/ledger/accounting/readiness results: absent, satisfied
   or non-crossing issuer demand; unavailable ore production, replacement orders
   or metal-market access; inadequate sale margin; and late payroll. All fail
   readiness as intended. The three demand controls generate no hypothetical peer
   work or late issuer purchases. The payroll control recovers to zero arrears and
   zero food deficit, but still fails because wages entered arrears; later recovery
   does not erase the historical failure.

10. **Runnable comparison and scope documentation.** The CPU acceptance example
    accepts `--offer-publication`, optionally with `--continuation-matrix` or
    `--sale-startup-matrix`. Publication startup labels describe actual inventory
    and review settings, rather than implying a supplied ask. README, integration
    status and controller documentation describe the opt-in policy and the
    remaining initialization assumptions. The standalone CPU positive run passes.

## Results

All runs below cover 48 months and use monthly organizational reviews. None starts
with workshop metal asks or town ore asks. Other existing market offers remain.

| Publication policy and stocks | Mints | Workshop batches | Town batches | Food deficit | Ending wage arrears | Ready |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| Retain; four workshop metal + four town ore units | 0 | 0 | 0 | 67 | 2 | No |
| Publish; original two workshop metal + zero town ore units | 1 | 0 | 0 | 61 | 2 | No |
| Publish; four workshop metal + zero town ore units | 4 | 2 | 0 | 37 | 2 | No |
| Publish; four workshop metal + four town ore units | 44 | 43 | 40 | 0 | 0 | Yes |

Every treatment also retains four opening workshop ore units and the baseline's
other endowments. The top and bottom rows differ only in publication policy. The
partial-inventory controls use reference execution; the successful treatment also
passes CPU, replay and reconstruction checks. No fixture replenishes inventories
at runtime.

In each of the three full post-startup years, both organizations complete twelve
batches and pay twelve wage coins each. Twenty-four ore units change hands per
year. Workshop/town annual net income is 12/13 coins; their closing cash reaches
54/47 coins. Both household delivery units settle on time, the state loan is repaid,
and nine separate accounting books reconcile.

## Interpretation and remaining work

This closes one specific coordination gap: a governor may make a stocked sale
opportunity visible even when its conservative forecast predicts no gain.
It does not project a fictitious buyer or treat publication as income. Ordinary
buyers still decide whether to procure, and purchases still require funded
packages and ordinary settlement. The decision reason explicitly distinguishes equal
projection publication from an improving forecast.

The experiment retains signed prices, lot ceilings, reserve floors, purchase
options, employment, consent, organizational identities and existing state/household
policies. It retains full-world forecast information, the workshop's twelve-month
horizon and observed peer-process/state-ballot recurrence with 24-month memory;
the town uses a four-month horizon. Initial absence of hypothetical peer work does
not imply the later forecasts stop using observed recurrence. No new learning,
ZIP update, hiring, founding, borrowing or self-directed personal policy changes
are introduced. Startup without sufficient working inventory remains unresolved.
Persistence beyond 48 months and recovery from unanticipated shocks are untested
here.

The default publication policy remains `Retain`. The fixed-route input stock claim
fix intentionally changes order sizing when accepted obligations encumber stock;
it is not limited to the new scenario. Protection is bounded by configured horizons,
not every possible future obligation. Legacy provisioning rebuilds its own asks and
remains outside that new execution guarantee; stale fixed-route budget receipts
are cleared rather than misreported as provisioning decisions.

A useful next experiment is bounded startup acquisition or funding for the missing
ore inventory, with explicit consent, repayment/default consequences and refusal
controls. It should start from the failing partial-stock cases above and retain
these readiness requirements.

## Run

From `exp/economics`:

```sh
cargo +1.92.0 run --locked --release --example acceptance -- --offer-publication
cargo +1.92.0 run --locked --release --example acceptance -- --offer-publication --sale-startup-matrix
cargo +1.92.0 run --locked --release --example acceptance -- --offer-publication --continuation-matrix
cargo +1.92.0 test --locked --release --lib agency::publication -j8
cargo +1.92.0 test --locked --release -j8 \
  --test publication_execution --test publication_governance --test input_sale_claims \
  --test acceptance_publication --test publication_startups --test acceptance_publication_controls
cargo +1.92.0 test --locked --release --test acceptance_continuation voluntarily_published -j8
```

The CLI startup matrix additionally includes `AnnualBaseStocks`; unlike the table's
monthly treatments, that inherited cold fixture has annual reviews. Do not read
that extra comparison as changing inventory alone.

## Verification

**131 targeted tests passed:** 25 library tests and 106 integration tests across
24 suites. These include the publication safeguards, matched and partial-stock
comparisons, governance gates, fixed-route stock claims, eight CPU/reference
failure cases grouped into two tests, the publication continuation/replay/reordering
harness, and regressions for discovery, authority, forecasting, market orders,
provisioning, telemetry and integrated agency. Only the new publication test in the
broader continuation suite was selected in this batch; previous continuation tests
are not counted as rerun. The standalone CubeCL CPU example also passed.

Independent review identified the stale legacy-provisioning receipt issue, which
was corrected and regression-tested; final review found no further blocker.
All-target release Clippy with warnings denied, formatting and repository artifact
checks passed. Generated logs and binaries remain under ignored `output/`. These
are targeted integration checks, not a claim that every repository test or every
possible organization/market arrangement has been verified.
