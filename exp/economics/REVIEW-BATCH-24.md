# Twenty-four planning and startup integration iterations

This batch follows [production preferences](PRODUCTION-PREFERENCES.md). It repairs
composition gaps in existing planning and finance, then tests an opt-in policy for
publishing output offers earlier. It does not change endowments, prices, settlement
order, signed authority or acceptance gates. The separate local trace-search utility
was reviewed, tested and committed before these economics iterations.

## Iteration record

| Iteration | Change or integration check | Evidence |
| --- | --- | --- |
| 1 | Financial loan discovery collects completion evidence when inherited goals require it. | A flow-enabled issuer borrows, completes minting and repays. |
| 2 | Forward discovery uses the same optional flow evidence. | A delivered wheat forward retains the correct zero-mint shortfall. |
| 3 | Financial comparisons select the current governor's supplied preferences. | Shared preference selector and financial comparison tests. |
| 4 | Forward demand uses that same effective preference list. | Governing reserve demand produces a proposal; another member's preference does not. Debt and payment safeguards remain in the comparison. |
| 5 | Flow-enabled financial discovery composes with continuation and scarcity. | Ten-month CPU/reference comparison, reconstruction after each phase, and unavailable-metal control. |
| 6 | Personal process goals seed discovered choices. | Signed process permission is still required; reversed catalogs retain choices. |
| 7 | Personal non-flow goals also reach recipes, with bounded enumeration. | A private metal-reserve goal discovers refining; oversized installation fails atomically. |
| 8 | Exposure and flow scoring share consecutive completed-month validation. | Missing, duplicate, shifted, unclosed and overflowing intervals are rejected. |
| 9 | Reserve scoring widens before subtracting balances. | Extreme arithmetic cannot wrap into an apparent improvement. |
| 10 | Startup forward eligibility applies finite resource use across either role. | Previous buyer/seller use blocks reuse; another resource remains independent. |
| 11 | Flow scoring includes completion of work begun before the forecast. | A live crop enters the branch active and counts only at harvest in month 6. |
| 12 | Production shortage can trigger early review. | Prior-Close shortage triggers an otherwise off-schedule review; reconstruction agrees. |
| 13 | Effective preferences follow the active office and retain frozen beneficiaries. | Rotating office selects personal goals, then fallback where no preference exists. |
| 14 | Startup diagnostics expose dated stocks, choices and procurement receipts. | `startup_publication` example prints phase boundaries for the first eight months. |
| 15 | Add opt-in productive-output publication on equal projected losses. | Same productive choice and quote portfolio, plus signed output asks only. |
| 16 | Preserve effective quote semantics for work-only choices. | Missing quote command inherits live quotes; explicit empty command clears them. |
| 17 | Bound publication by governing preferences, authority and economic interest. | Unequal losses, changed work/buys, unauthorized asks, market exclusion and satisfied payment goals reject augmentation. |
| 18 | Exercise the measured founding tie through ordinary decisions. | Same loss vector, earlier ask, ordinary effective date, actual first mint in month 3. |
| 19 | Compare all four opening-stock variants over 48 months. | Reconciled evidence and unchanged readiness checks. |
| 20 | Retain adverse supply and price controls. | No-ore and low-price variants remain separately measured. |
| 21 | Test labor, storage, process law and market access limits. | Posting cannot create missing capacity, inventory space, permission or a fill. |
| 22 | Remove startup-forward authority explicitly. | Separate ore-only and neither-stock comparisons avoid inferring independence merely from no signed forward. |
| 23 | Check continuation, replay and catalog order for the new policy. | Twenty-four-month ore-only and neither-stock CPU/reference comparisons. |
| 24 | Consolidate current documentation and verification. | README, integration status, forecast contract and this reproduction record. |

## Why earlier publication matters

The unchanged ore-only opening has workshop cash and ore, but no finished metal.
In month 1, refining plus an ore bid ties refining plus that bid and a metal ask at
losses `[11, 11972]`. The ordinary catalog tie selects the former. Refining completes
in month 2, while the later ask becomes effective in month 4.

The issuer's first mint attempt competes with existing commitments. After its
month-2 loan installment it has eight coins, then its household wheat forward
uses two at Acquire. The input package requires eight. The procurement receipt
therefore reports insufficient funds. Later work and prices must be evaluated from
that changed boundary; a workshop completion alone is not evidence of demand.

The new policy can select the equal-score program with the output ask in month 1,
effective in month 2. The issuer then independently chooses minting for month 3.
The eight-month regression records mints in months 3, 4, 6, 7 and 8, versus none in
the matched control. The policy applies at later reviews too, unlike the initial
one-time diagnostic intervention. No buyer promise is inserted by the publication policy.

## Policy contract

`ProductiveAsksOnEqualProjection` is opt-in at founding. `Retain` remains the
default; `StockedAsksOnEqualProjection` retains its previous behavior. The new
policy also preserves that older stock-backed publication rule when retention is
the best choice.

When the selected alternative contains productive work, a candidate can replace
it only when the active governing loss vector is identical, its non-quote commands
are identical, and it preserves the selected choice's effective quote portfolio.
Additional quotes must be signed stock asks for outputs of that work, with an
unmet governing objective for their payment resource. It prefers more added asks;
remaining ties use stable catalog order. A work-only choice retains live quotes;
an explicit empty portfolio means deliberate withdrawal.

An ask is permission to offer available output, not a delivery promise. The rule
does not establish feasible production, a counterparty, matching prices, funding,
profitability or completion. Actual clearing still clips to stock and signed floors,
protects existing claims and applies ordinary resource and legal limits. Work,
prices and buyers are not added to forecasts. Ordinary tenure and next-month
activation apply. Startup-forward candidate ordering is unchanged.

## Forty-eight-month comparison

The workshop uses monthly cash exposure and a production-first completion target;
the town retains terminal cash preferences. All rows retain startup-forward
authority unless explicitly stated. Food deficits are cumulative, wage arrears are
outstanding at the final boundary. Readiness also checks historical late payments
and three complete operating years.

| Opening workshop stock | Productive publication | Control | Mints | Workshop batches | Town batches | Food deficit | Wage arrears | Forward signed / activated | Ready |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | --- | --- |
| Both | On | Baseline | 45 | 44 | 43 | 0 | 0 | — | Yes |
| Metal only | On | Baseline | 44 | 43 | 44 | 0 | 0 | 3 / 5 | Yes |
| Ore only | On | Baseline | 45 | 46 | 46 | 0 | 0 | — | Yes |
| Ore only | Off | Baseline | 0 | 1 | 2 | 67 | 2 | — | No |
| Neither | On | Baseline | 0 | 1 | 3 | 67 | 2 | 1 / 3 | No |
| Neither | Off | Baseline | 0 | 1 | 3 | 67 | 2 | 1 / 3 | No |
| Metal only | On | No ore supply | 2 | 0 | 0 | 52 | 2 | — | No |
| Ore only | On | Low metal price | 8 | 8 | 8 | 22 | 2 | — | No |

The older both-stock and metal-only production-first results were respectively
44/43/43 and 44/43/44 mints/workshop/town batches. Those Off rows are historical in
[production preferences](PRODUCTION-PREFERENCES.md); only the ore-only and neither
Off comparisons were rerun in this matrix. Better publication does not repair an
unprofitable output price or unavailable input supply.

Neither-stock startup still completes its first metal too late for the issuer's
initial funded window. Its signed ore forward and one workshop completion do not
establish sustainable circulation. Coordinating later financing, protected payments
and production remains a separate next step. The policy does not mint liquidity,
reorder accepted commitments or infer recurring buyer demand from unrealized work.

## Negative controls and independent limits

Twelve-month controls remove worker capacity, process permission or marketplace
access separately; none completes a mint or sells workshop metal. The labor control
removes the smith's whole capacity endowment, not only one employer's allocation. The storage
control removes both workshop capacity and its opening ore, so it demonstrates a
blocked startup rather than isolating the marginal effect of storage alone.

Two additional controls retain productive capacity: a satisfied issuer cash goal
and a bid below the workshop ask. Both still produce metal, but neither sells that
metal to the issuer or completes a mint. Publication therefore does not manufacture
crossing demand. These controls do not establish general price discovery or prove
all buyers correctly anticipate changing future supply.

Removing both organizations' startup-forward authority leaves the ore-only result
unchanged: ready, with 45 mints and 46 batches at each organization. Neither-stock
startup then completes no mint or organizational batch, with cumulative food
deficit 67 and final wage arrears 2. The tests pin both outcomes and require no
startup-forward records. This establishes independence from that authority only in
the measured ore-only configuration, not independence from all existing finance.

## Reproduction and scope

From `exp/economics`:

```sh
cargo +1.92.0 run --locked --release --example startup_publication -- ore on
cargo +1.92.0 run --locked --release --example startup_publication -- ore off
cargo +1.92.0 test --locked --release --test startup_publication -- --nocapture
cargo +1.92.0 test --locked --release --test production_continuation
```

The example logs observations at the existing phase boundary. Output is diagnostic;
economic failure is reported by readiness and does not itself cause an execution
error. Store raw output under ignored `output/economics/`.

The ten-month financial continuation test compares world, state and ledger; both
branches execute audit checks. The 24-month production continuation tests additionally
compare reports and final audit, replay state at each phase using the live world,
and reverse independent catalogs. These are in-memory reconstruction and CPU tests,
not durable checkpoint certification, bare-ledger world reconstruction or CUDA tests.
Rotation coverage tests preference selection, not a new election or financing regime.

## Verification record

The selected regression coverage contains 108 distinct tests: 34 library tests,
57 finance/forecast/path/process integration tests, one governing-preference test,
four production-continuation tests and 12 startup-publication tests. It is targeted
coverage, not a full-crate suite or a rerun of every historical acceptance family.
The timing expectation from the one-time diagnostic was corrected to the persistent
policy's observed sequence and rerun separately. No economic rule was changed to
satisfy that expectation. The no-forward comparison pins readiness and production,
food and wage outcomes, in addition to checking that no forward was signed.

Verification also includes strict all-target Clippy, formatting, the repository
artifact check and the 48-month ore-only example. An independent review checked the
publication constraints, effective-quote handling and scenario evidence. Raw output
remains ignored; this document records the settings, results and limitations.
