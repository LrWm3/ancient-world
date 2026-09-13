# Credit, issuance and currencies — design draft

Status: experimental council and commercial credit pilots and bounded shared-
currency issuance exist behind explicit switches. Bounded restructuring and delayed-
export negotiation and bounded account-estate settlement also exist; bounded operator/institution claim succession is implemented. Proportional recovery from closed-estate cash is implemented; broader
estate policies, extended calibration and currency exchange remain pending.
See the [implementation record](credit-implementation.md) for evidence and limits.
The [credit-event regression](credit-chronicle-regression.md) verifies six matched
200-year arms after adding causal loan events; it does not pass the balance gate.
These pilots remain opt-in; implementation does not mean their balance gates pass.

This is a toy-economy experiment: test whether moving existing cash across time
improves useful activity, then test bounded money creation, before introducing
multiple currencies. Neither financial realism nor automatic population growth
is an acceptance criterion. More cash cannot create food, workers or cargo.

## Scope of this draft

The proposed sequence is **existing-money credit → bounded shared-currency
issuance → distinct currencies and finite exchange**. The first two experiments
belong to Stage 1; Stage 2 is conditional. Constants cleanup is separate work.

| Comparison arm | New lending | New issuance | Question |
| --- | --- | --- | --- |
| Existing system | Off | Off | What happens without either intervention? |
| Credit only | On | Off | Does moving existing cash across time unlock feasible work? |
| Minting only | Off | On | Does additional money relieve a remaining liquidity constraint? |
| Credit + minting | On | On | Do the mechanisms complement each other or amplify losses? |

Start comparisons from the same debt-free checkpoint. Disabling new lending in
an already indebted world must still service its existing contracts; that is a
policy-withdrawal experiment, not a clean baseline. Keep harvests, starting
inventories, tax policy and unrelated settings matched across arms.

Treat the stages as experiments with decision gates, not a commitment to add every
mechanism regardless of the results. Credit first tests a timing problem using
existing money. Issuance then tests liquidity with an explicit external source.
Neither substitutes for harvests, available labor or reliable deliveries. Distinct
currencies follow only if the simpler experiments justify the added complexity.

The delivery checklist tracks complete acceptance milestones, not merely whether
code exists. An unchecked pilot can already run while its failure cases, reporting
or balance evaluation remain unfinished. The implementation record is the source
for detailed progress.

## Work order and decision points

| Increment | Reviewable result | Decision before continuing |
| --- | --- | --- |
| Stage 1A: existing-money credit | Council tax bridge and commercial payment pilot, with debt and cash receipts | Verify that loans fund otherwise feasible activity and reject borrowers without credible net receipts. |
| Stage 1B: bounded issuance | Independently enabled issuance into council treasuries; four matched experiment arms | Check whether benefits survive shocks without growing debt, issuance dependence or recurring rescues. |
| Stage 2A: currency ownership | Persistent denominations, account balances and explicit migration of existing claims | Verify ownership and obligations survive migration and checkpoint continuation. |
| Stage 2B: exchange | Trade/tax acceptance, finite dealer reserves and observed-demand quotes | Verify both currency legs conserve money and unavailable reserves actually constrain trade. |

Complete the credit experiment before adding issuance; run the combined comparison
before introducing distinct currencies. A failed gate means revisiting the model
or retaining the simpler stage, rather than automatically expanding the system.
The checklist below separates planned work from completed foundations; the linked
implementation record contains the detailed status and experiment results.

## Existing foundations and constraints

- Towns hold market cash in `Economy.finance`; councils and institutions have
  treasuries; households and workshop operators have separate accounts.
- Operators in [enterprises.rs](../src/enterprises.rs) sell completed workshop
  services to towns. They do not currently own the town's finished products or
  automatically receive export proceeds. Their ledger distinguishes revenue,
  wages, rent, dividends and returned capital.
- [Export contracts](export-contracts.md) reserve actual buyer cash in escrow.
  Producers receive payment at dispatch, not necessarily at final delivery.
  Expiry refunds unused escrow. Estimated surplus is not money or a receivable.
- `History::economy_residuals` already counts cash across towns, firms, institutions,
  councils, household accounts, journeys and contract escrow. Extend this existing
  ownership inventory; do not create a second authoritative balance table.
- Councils allocate finite resources to administration, relief and other duties.
  [Council allocation](../src/household_economy/council_allocation.rs) and the
  [monthly schedule](monthly-schedule.md) provide explicit policy and timing hooks.

Before the pilot, trace actual tax collection and every relevant sale payment to
its owner and settlement phase. A forecast must describe the borrower's future
receipts after existing commitments, not total regional production or taxes that
have already arrived.

## Stage 1A: lend existing money

### Accounts, contracts and ownership

Introduce a sparse CPU-owned credit subsystem, using stable IDs and serializable
numeric records. Dense production stays on its current execution path.

| Record | Required contents |
| --- | --- |
| Account reference | Account kind and stable owner ID; resolve into an existing balance |
| Currency | Stable ID; initially only the shared currency |
| Loan | ID, lender, borrower, currency, purpose, disbursement month, original/outstanding principal, fixed simple interest rate, accrued unpaid interest, maturity, installment schedule, status |
| Repayment evidence | Source kind/ID, observed receipts, conservative forecast, already-pledged share, observation date |
| Receipt | Requested, approved and transferred amount; principal/interest split; month; reason; causal references |
| Restructuring/default | Previous terms, consent, revised terms or write-off, missed amounts, event references |

Account references are adapters, not new stores of cash. A legal successor may
assume an obligation through an explicit event. Changing officeholders does not
cancel council debt. Closing a firm must settle or default its obligations before
returning residual cash to its owner; death, abandonment and institutional
relocation must not leave dangling counterparties.

Keep all contract accounting in f64 initially. Existing f32 accounts need an
adapter that returns the amount actually debited and credits exactly that amount.
Record any representation residual under a justified tolerance; never silently
round a lender down while crediting a borrower up.

### Voluntary underwriting and allocation

A loan requires both a feasible borrower request and an independent lender offer.
The lender reserves operating cash, limits exposure to each borrower, and compares
repayment evidence and risk with the offered return. Rejection is an ordinary
outcome. No automatic public guarantee or unlimited liquidity provider.

For the first experiment:

- Fixed simple interest on outstanding principal, accrued monthly after
  disbursement. No interest on interest or automatic late fees.
- Debt-service coverage uses conservative net receipts after essential operating
  costs, existing debt service and already-pledged proceeds.
- Cap individual principal, total borrower debt, lender exposure and term length.
  Quotes and caps are named experiment settings, not asserted historical constants.
- Reserve expected receipts against a single source ID across all loans. Multiple
  lenders cannot each count the same order or tax installment in full.
- Exclude simultaneous borrowing and lending by the same account in the pilot.
  New loan proceeds do not qualify as repayment income or support another loan.
- Collect requests and offers from a completed observation, then apply an explicit
  bounded allocation policy. Stable ordering ensures reproducibility; it must not
  silently substitute for a stated priority rule.

Lending changes who can spend today. It creates a financial claim and liability,
not new spendable currency. Principal received is financing, not sales, taxable
production or distributable profit. Interest paid is a separate income/expense
transfer; initially do not add a new tax-on-interest mechanism.

### First borrowers

**Council bridge loan against annual tax receipts.** Require recorded collection
history and a future collection window. Forecast actual collectible receipts using
current control, tax policy, administration and loss of tax base. New councils
without evidence are ineligible initially. Reserve essential service funding;
make the debt-service share of remaining funds explicit. If taxes fall short,
record arrears instead of redirecting food or inventing tax cash.

**Commercial bridge loan against a named payment.** Start with the actual payee:
for town-owned exports this is the town trading account; for an operator it is a
funded service order or a supported estimate of service receipts. Do not pretend
the existing workshop firm is an independent goods merchant.

A merchant/delivery-proceeds pilot needs an explicit beneficiary and payment
milestone on its contract. Preserve dispatch-paid contracts. A lost shipment
cannot retroactively erase dispatch revenue already received; a delivery-paid
pilot needs a distinct escrow-release rule and a defined recipient of refunds.
Add that small contract extension before claiming to test cargo-loss credit risk.
Borrowing cannot pledge the buyer's escrow as if it were already seller cash.

### Arrears, restructuring and default

Apply available debt-service funds proportionally to due claims under one stated
policy, with interest/principal allocation fixed in the contract. Partial payments
reduce only the amounts actually paid. Do not let loan iteration order determine
which equally ranked lender receives everything.

Missed installments enter arrears. After a bounded grace period, allow one
consensual restructuring with an evidence-backed revised schedule, or default.
No automatic refinancing, perpetual maturity extensions or repeated bailouts.
A write-off removes the financial claim and liability; it does not destroy or
create cash. Preserve both the original loss and any later recovery in history.
The first default consequence is restricted new credit and lender exposure loss;
complex collateral seizure, household subsistence debt and imprisonment are out
of scope.

### Monthly contract

Fit the existing five stages; do not rewrite the scheduler.

| Phase | Credit responsibility |
| --- | --- |
| Open | Activate dated terms; settle due arrivals; accrue previously outstanding interest once; settle obligations due from already available cash under the explicit debt-service allowance; apply contracted grace-period default after the final affordable payment |
| Reserve | Collect offers/requests from opening evidence; jointly limit lender cash, borrower exposure and pledged sources; transfer accepted principal once before dependent work is funded |
| Execute/settle | Existing work and commerce run normally; financing never counts as work or output |
| Respond | Record realized receipts and shocks; assess arrears and proposals for consensual restructuring; decisions become effective at a stated future boundary |
| Close | Reconcile accounts, debts and source reservations; archive receipts and significant events |

Income received after the Open payment window is available for the next monthly
payment window. Set initial council maturities accordingly. Do not run a second
implicit collection pass after annual taxes. Any later same-month proceeds sweep
must be an explicit extension with a boundary test.

## Remaining Stage 1 ownership boundary: closure and succession

New-credit account adapters reject abandoned towns, inactive institutions and
closed operators. Existing town debts use retained town treasuries after
abandonment. [Closed operator accounts](operator-credit-estates.md) now retain
cash for borrowing claims, repay proportionally using existing early-repayment
terms, and return residual cash to the existing owner. They can receive later
repayments without reopening the enterprise. Unpaid claims keep their maturity
and grace/default rules. Explicit post-default recovery is now available through
the separate settlement API. An opt-in [late-export policy](export-default-recovery.md)
allocates newly received proceeds; [closed-estate recovery](estate-default-recovery.md)
shares opening cash across live claims and remaining default losses.

[Institutional estates](institution-credit-estates.md) now retain shutdown cash
for the same claim settlement, with only actual residual transfers going to the
home town. Temporarily inactive relocating institutions are excluded. The automatic
pilots use councils and town commercial payees, while the broader account types
also support explicit caller-supplied contracts.

[Dated claim ownership](credit-claim-succession.md) now supports explicit
consensual assignment among existing operating account types. Payments, recovery,
consent and exposure follow dated ownership while original contracts remain intact.
Household beneficiaries now have settlement-only wallets and separate receipt
accounting. Closed solvent operators now schedule remaining claims to their owner
household; closed institutions schedule them to their home town. Activation is next
month, and live debt or unrecovered default losses prevent distribution of either
cash or claims. Relocating institutions and missing/lost household successors retain
their claims. This is bounded succession, not general estate adjudication.

Remaining estate work and the broader acceptance requirements:

1. Separate operating eligibility from legal account/estate existence. Closing
   prevents new borrowing and offers immediately. Existing claims keep their IDs
   and original contractual parties; retaining a balance for settlement does not
   reopen an enterprise or restore institutional services.
2. At each existing closure boundary, identify obligations and receivables before
   returning residual cash. Collect outstanding amounts under an explicit
   liquidation policy, allocating equally ranked debts proportionally. Cash
   unavailable after actual liquidation becomes a recorded loss; closure is not
   an implicit repayment and cannot count principal as an operating expense.
3. Persist assignment/succession records for surviving receivables, naming the
   successor, old account, effective month and legal reason. Do not rewrite loan
   origination evidence or overwrite transfer provenance. A household receiving
   an operator's assets may receive a creditor claim without becoming eligible
   for the deferred household-loan pilot.
4. Keep unsatisfied obligations attached to the resolved estate/default record;
   do not silently impose personal liability on an operator's owner. Council
   office turnover preserves the council account. Relocation preserving an
   institution's ID preserves its contracts. Abandonment without a legal
   successor needs an explicit retained claim or write-off decision.
5. Reconcile estate cash, liquidation payments, returned capital, interest income
   and assigned claims through the existing ownership inventory. Include
   same-month repayment/closure ordering, disabled-credit servicing, mixed
   account precision and serialized continuation.

Required fixtures include an indebted firm closing with ample cash, an insolvent
firm, a closed creditor whose borrower later pays, simultaneous debtor/creditor
closure, institution relocation versus dissolution, and a council leader change.
Test cash and claims independently: an assigned receivable is not spendable money,
and a write-off does not erase cash already returned to an owner. Town settlement
and bounded operator/institution estate mechanisms cover part of this boundary;
automatic cash recovery and bounded claim succession are implemented. Contested
beneficiaries, missing successors and general asset disposal remain outside them.

## Stage 1B: bounded shared-currency issuance

Keep credit independently switchable. A dated civilization policy may issue the
shared currency into its own council treasury. Only existing spending mechanisms
move it onward.

Require a per-issue cap, rolling annual cap, lifetime experiment cap and cooldown.
Derive any capacity-related ceiling from lagged real activity or a frozen baseline,
not current nominal prices or the enlarged treasury: issuance must not raise its
own next cap. Expired allowances do not accumulate indefinitely. Leadership
turnover cannot reset issuance history. Start with a fixed, reviewable schedule;
a deficit-response policy can be a separate experiment after that works.

Record issuer, recipient, currency, amount, month, authorization and purpose.
Issuance occurs at Open before its permitted spending window. Disabling issuance
stops future creation without deleting cash already issued or forgiving debt.
No physical mint or metal-backing model is implied.

Shared currency means other civilizations are exposed to price and trade effects
without controlling the issuer. Report that explicitly. Track balances by account,
owner civilization and account class plus issuance receipts. Money is fungible:
these reports show distribution, not exact ancestry of every issued coin; tagged
money provenance would require an additional model.

## Accounting and persistence

For each currency, validate:

```text
closing spendable money = opening money + authorized issuance - explicit retirement
closing principal = opening principal + disbursements - principal paid - principal written off
closing interest due = opening interest due + accrual - interest paid - interest written off
```

Transfers include escrow and travel balances exactly once. Loan assets and
liabilities are reported separately from money supply. Preserve enterprise cash
and profit reconciliations by adding financing flows rather than misclassifying
borrowing as revenue or repayment as payroll. Do not count operating expenses
again when evaluating a loan's outcome.

Old worlds initialize with shared-currency identity, no loans and no retrospective
issuance. Archive policies, source reservations, accrued balances, schedules,
receipts and defaults at monthly boundaries. Experiment switches prevent new
contracts; servicing existing obligations continues. Save/reload cannot accrue
interest, issue money, release escrow or pay an installment twice.

## Experiments and gates

Run matched seeds 17, 81, 256, 409 and 1024 with baseline, credit-only,
minting-only and combined arms. Begin with short controlled fixtures, then
200-year comparisons and selected 500-year stress runs. Separate feature random
streams so enabling credit does not consume unrelated historical draws.
Use two seeds for initial tuning and retain the other three for held-out checks.
All numerical caps and failure gates must be fixed in the run protocol before
examining the held-out results.

Report distributions and time series of:

- Completed work and unpaid/unfunded work; loan-funded projects actually completed.
- Food produced, available for purchase, affordable, purchased and consumed.
- Nominal prices and purchasing power measured in a fixed basket; basket shortages
  remain visible rather than being replaced by a convenient price.
- Cash by owner/class, concentration, escrow, velocity proxy and idle reserves.
- Debt relative to realized receipts; service coverage, arrears, restructuring,
  default losses and lender operating shortfalls.
- Issuance, circulation and accumulation across civilizations; monetary residuals.

The [crop-scarcity comparison](monetary-crop-scarcity.md) extends the runner with an
explicit, shared production intervention and reports actual crop harvest separately
from aggregate food. All eight 200-year arms completed: the severe case collapsed with or without
credit/issuance, while control issuance remained mixed. The Stage 2 gate remains
no-go on present evidence. This sustained constraint does not replace transient shocks.

Required causal and boundary fixtures:

- Exact two-account principal and interest transfers; partial repayment; default
  and write-off; two borrowers competing for insufficient lender funds.
- Productive borrower with a timing gap versus insolvent borrower without receipts.
- Adequate food but inadequate cash versus failed harvest despite abundant cash.
- Lost tax base, blocked trade, lost delivery-paid cargo and already-paid cargo.
- Double pledge rejection, closed firm, relocated institution and leadership change.
- Caps across year boundaries, repeated requests, policy toggles and checkpointing.
- Disabled-coupling and unaffected-account controls; batching/checkpoint equivalence.

First assess the mediator: cash reaches a previously unfunded activity and that
activity completes. Later population differences alone are not proof of benefit.
Do not proceed to Stage 2 if results rely on cap exhaustion, rising debt without
receipts, repeated restructuring or transfers that bankrupt essential lenders.
A stable null result is useful evidence; it does not justify larger issuance until
some headline metric improves. Report game balancing separately from conservation
verification.

## Stage 2: civilization currencies and exchange

Proceed only after Stage 1 gates pass. Keep stable currency IDs independent of
current government identity: conquest need not abolish a currency automatically.

1. **Migrate ownership and contracts explicitly.** Stage 1 loans stay in the
   shared currency unless both parties accept redenomination. The safest first
   transition retains shared balances and introduces domestic currencies through
   declared exchanges/issuance. If instead converting balances, publish the
   assignment rule, conversion rates, extinguished balances and replacement
   amounts. Do not choose denomination merely from an account's current location.
2. **Make acceptance matter.** Contracts specify invoice and settlement currency;
   councils specify tax currency. An importer lacking accepted money must exchange
   it or decline/delay the order. No automatic forced acceptance.
3. **Use finite exchange inventories.** Dealers hold actual currency balances.
   Exchange debits and credits both legs atomically, with spread income explicit.
   Aggregate competing quotes against available reserves; support partial fills.
   No implicit reserve replenishment or unbounded credit from conversion.
4. **Move quotes with observed imbalance.** Lagged realized trade/tax demand,
   inventory pressure and bounded adjustment determine dealer quotes. A supply
   reference `starting_value × starting_supply / current_supply` may initialize
   or weakly anchor quotes, in a stated numeraire. Define behavior for zero supply
   and new currencies. Decay the anchor only after exchange has enough evidence.
5. **Separate quote from execution.** Missing reserves mean no executable trade,
   even at a published bounded quote. Check triangular arbitrage and ensure any
   arbitrage consumes existing reserves rather than minting profits.
6. **Restrict issuance to the issuer's currency.** Foreign-denominated loans retain
   their units. Report exchange gains/losses separately from principal, production
   and domestic-money conservation; there is no meaningful raw sum of unlike units.

Government bonds, household lending, complex collateral, bank deposit creation,
central-bank rescue, speculative FX debt and fully modeled mint inputs remain
later possibilities, not requirements for either stage.

## Delivery checklist

- [ ] Audit current account owners, tax timing, dispatch payments and ledger coverage.
- [x] Add shared currency ID, account adapters, loan records and exact transfer fixtures.
- [ ] Add dated underwriting, source reservation and explicit lender allocation.
- [ ] Pilot council tax-bridge credit; verify timing and failed-tax-base outcomes.
- [ ] Pilot commercial payees; add a delivery-paid contract only where needed.
- [ ] Implement arrears, one bounded restructuring, default and closure/succession.
- [ ] Add explorer debt/credit receipts, history events, archives and continuation tests.
- [x] Run credit-only comparisons and record null/negative results as well as benefits.
  See [held-out comparisons](monetary-estates-heldout.md) and the
  [request-capacity audit](credit-capacity-diagnostics.md); the latter distinguishes
  absent requests from submitted rejections. This does not pass the Stage 2 gate.
- [x] Add independently switchable capped issuance and supply-ledger reconciliation.
- [ ] Run four-arm tests, stress cases and held-out gates; write a go/no-go summary.
- [ ] If justified, implement Stage 2 currency ownership and migration before FX.
- [ ] Add accepted-payment demand, finite-reserve exchange and foreign-debt tests.

Keep implementation increments independently reviewable. Generated trajectories
and raw results belong under ignored `output/`; commit source and Markdown
summaries only. Council credit, commercial credit and shared issuance are opt-in pilots;
distinct currencies and exchange remain unimplemented.


### Post-default recovery foundation

The [explicit recovery operation](credit-default-recovery.md) now transfers
authorized cash against a recorded default loss without reopening debt, accruing
new interest or erasing default history. Dated replay-safe receipts retain both
account deltas. [Late-export recovery](export-default-recovery.md) now allocates new matching
proceeds after live obligations and operating reserves. [Closed-estate recovery](estate-default-recovery.md)
shares available cash proportionally with live claims. Succession without an eligible
beneficiary and broader bankruptcy rules remain pending; the closure checklist stays open.
