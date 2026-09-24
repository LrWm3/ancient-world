# Verification stress test: economic and institutional primitives

Status: proposed verification program. No stage below is certified complete by this
document. Implement as many cases as practical, advancing through demonstrated
capabilities rather than treating the list as a release checklist. The immediate
focus remains minting, markets, forwards and basic lending. The
[integration matrix](INTEGRATION-STATUS.md) is the authority on current support;
[GOALS.md](GOALS.md) defines the institutional direction.

## Purpose and boundary

Ancient World should eventually support civilizations and species with different
needs, information, currencies, lending terms, markets and organizational norms.
The experiment tests whether generic agents, agreements, rights, processes and
bounded planning can express those differences consistently. Individuals remain
individuals; organizations coordinate actual members, assets and delegated work.
Neither an aggregate population nor a financial claim supplies extra people,
physical goods or labor.

Modern financial systems provide demanding test cases for these primitives. Banks,
funds, exchanges and derivatives are useful because they combine authority,
ownership, uncertain promises, dated settlement and failure in different ways.
They are experimental content, not features promised for the final Ancient World
experience. Their definitions may deliberately be technologically unrealistic.
Verification catalogs should remain separate from eventual playable catalogs;
historical availability is a content decision, not a limitation of the engine.

The ambition is to implement many of these cases, not merely describe them. A case
that cannot be represented without hidden balances, duplicated resources or special
privileges is evidence that the primitives need review. An unimplemented case is
not by itself proof of a defective architecture. Classify the gap first: missing
catalog definition, missing policy, missing observation, missing solver or missing
primitive. A new contract payoff or matching algorithm can be legitimate; a
product-specific escape from accounting or authority is not.

## What counts as verified

Every case advances through four levels, recorded separately:

1. **Representable:** identify parties, authority, resources, claims, dates,
   observations and every success/failure transition without running it.
2. **Mechanically correct:** execute a small scripted example with independently
   calculated balances, obligations and outcomes, including rejection and default.
3. **Agentic:** agents discover opportunities and choose among feasible alternatives
   from their needs, policies and observations. Scripted orders do not establish this.
4. **Composed and stressed:** combine with another verified system, then exercise
   scarcity, competing claims, shocks and continuation without special bypasses.

For each case retain a Markdown record of configuration, opening state, law and
charter parameters, decision policy, seed, observations, expected results, actual
results and limitations. Record requested, allocated, reserved and completed work;
submitted, matched, settled, expired and failed orders; and promised versus paid
amounts. Preserve a small hand-calculated control alongside any larger scenario.
Generated traces and checkpoints belong under ignored `output/economics/`.

The common evidence requirements are:

- One authoritative balance per account; mirrored creditor/debtor claims reconcile.
  Transfers conserve their resource. Issuance, redemption, production, consumption,
  writeoffs and revaluation have explicit, distinct provenance. Do not demand that
  money or marked wealth remain constant when the model explicitly changes them.
- Prices and valuations are observations or estimates, not available cash. Track
  denomination, quantity, accrual, book value, market value and liquidity separately.
  State rounding and residual rules, including how fractional amounts accumulate.
- Permission, ownership, custody, pledge, delegation and settlement authority are
  distinct. A member's hours or assets cannot be spent again by its organization.
- Resource checks are joint across competing commitments. No repeated pledge,
  sale, reservation or forecast can silently reuse the same resource.
- Accepted agreements, due claims and legal finality are distinct from a matched
  order. Failed delivery has an explicit outcome; completed transfers are not
  retroactively erased to make a default look harmless.
- Decisions use dated available observations. Compare forecast with realized need
  satisfaction, completed production, payments and failures. Perfect foresight is
  allowed only in a fixture explicitly granting it.
- Replay, monthly/batched execution and checkpoint continuation agree. Reordering
  independent records cannot change outcomes unless an explicit ordering policy
  says it may. Forged/stale receipts and duplicate settlement fail atomically.
- Report economic outcomes as well as reconciliation: deficits, foregone work,
  volume, defaults, recovery, concentration, idle capacity and loss distribution.
  Correct accounting can describe a failing economy; it does not prove good policy.

## Dependency path

```text
0 identities, units, claims and evidence
  -> 1 minting and simple exchange
  -> 2 eligible marketplaces, bids/asks and observations
  -> 3 forwards, loans and delivery/default
  -> 4 organizational authority and contributed resources
  -> 5 custody, clearing, netting and finality
  -> 6 banking, liquidity and monetary institutions
  -> 7 securities, ownership and investment institutions
  -> 8 contingent contracts, margin and insurance
  -> 9 pooled claims, securitization and infrastructure
  -> 10 composed systems and alternative civilizations
```

This is a dependency guide rather than a demand to finish every product in sequence.
Small alternative-species and no-minting controls should start early. Stage 4's
formation/authority work can proceed beside stages 1–3; organizational finance
requires both. Insurance can start after stages 3–4, while cleared derivatives need
stage 5. Keep the existing scheduler and explicit allocation boundaries; each new
case must name its observation, reservation, execution and visibility boundaries.

## Evidence so far — partial coverage, not stage completion

| Stage | Existing evidence | Missing evidence needed for the broader gate |
| --- | --- | --- |
| 0: audit | Shared claim executor, conserved transfers, CPU/reference and continuation controls, planning/settlement observers | All drivers composing under one reservation model; organizational authority and budgets |
| 1: minting | Grain-linked issuance and a separate physical minting/provision pilot with finite inputs and shortfalls | General redemption/backing arrangements and integration with credit and institutions |
| 2: markets | Bilateral fixed/concession/ZIP pricing, local town books and recurring need orders | General labor/right/asset markets and shared financial/institutional budgets |
| 3: forwards/loans | Prepaid deliveries, general consented advances, mortgages, alternative-tender allocation, crop-preserving enforcement and actual resale | Autonomous common financing choice; non-loan claim admission and broader liquidation |
| 4: institutions | Existing household pooling pilot; static law permissions/recognition | Constitution/charter governance, explicit contributed labor, lawful formation and death/dissolution estates |
| 5: custody/finality | Dedicated loan-estate custody, actual proceeds, secured/general distributions and explicit write-offs | General custody, netting, clearing, custodian failure and legal finality |
| 8: contingent claims | Capped original-loan guarantees, finite calls and matching recourse | Insurance, margin, options, broader contingent claims and lien subrogation |

The recovery checks include unfunded buyers, finite guarantee budgets, retained
deficiencies, optional discharge and unchanged attached production.
[Contract recovery](CONTRACT-RECOVERY.md) records the 451-test full run and 61-test
final focused run; those overlap and do not certify the stages above. Later
banking, securities, infrastructure and alternative-civilization gates remain
verification targets, not implemented systems. The
[integration matrix](INTEGRATION-STATUS.md) still governs permitted combinations.

## Stage 0 — Establish the audit baseline

**Needs:** existing agent, resource, process, transaction and condition components.

**Build:** a minimal catalog of stocks, dated service capacity, needs, assets,
rights and coin-denominated claims. Declare opening endowments and distinguish
legal title, economic benefit and authority. Use one person, two counterparties,
then a small organization before scaling to 32 agents.

**Show:** transfer, production, consumption, loan accrual and default each have
independent accounting controls. A resource used for food, collateral and an order
cannot be available three times. Institution membership does not duplicate personal
needs or capacity. A need deficit with consequences affects subsequent capability;
report when a fixture intentionally omits such consequences.

**Failure signal:** labels or agent types decide accounting semantics implicitly,
or closing summaries cannot be reconstructed from committed records.

## Stage 1 — Minting and simple exchange

**Needs:** stage 0, denomination and authorized issue/redeem actions.

**Build:** finite treasury and initial coins; then explicit issuance rules, including
the existing grain-linked issuance idea. Distinguish a commodity-backed redeemable
claim from an unbacked token: backing and redemption are terms, not consequences
of calling something money. Paper currency is a representation/issuer arrangement
to test, not automatic extra purchasing power.

**Show:** opening supply plus authorized issuance minus retirement equals closing
supply. Moving currency does not issue it. A repeated annual trigger cannot mint
twice. Insufficient reserves reject or limit redemption according to the declared
terms. Coins avoid grain storage costs without bypassing ownership or spending
limits. Compare fixed-supply, issuance-enabled and issuance-disabled controls.

**Failure signal:** money appears to repair an unfunded transfer, or a state can
issue merely because of its agent type rather than its defined authority.

## Stage 2 — Prices from eligible marketplaces

**Needs:** stages 0–1 and bounded order generation from needs/surplus.

**Build:** a small town market with month-start proximity admission and a catalog;
multiple eligible buyers and sellers submit dated bids/asks. Choose and document a
first monthly match/posted-price rule, including multiple matches, partial lots,
priority, expiry and the no-match case. Add optional ZIP after a fixed/concession
control. Store price, volume and unfilled orders separately.

**Show:** two buyers competing for one seller's lot cannot both receive it. No match
means zero completed volume and no fabricated current price. Accepted prices obey
limits and physical settlement still checks funds/storage. A price-taking planner
uses prior observations without reading hidden future prices. A state and a person
can trade by the same rules when eligible. Later add labor and transferable rights
with their own delivery and permission terms. Arbitrage must consume actual funds
and respect access and delivery timing.

**Failure signal:** matching automatically counts as settlement, prices come only
from a state constant, or a venue silently makes every agent globally eligible.

## Stage 3 — Forwards and loans with real consequences

**Needs:** stages 0–2, dated obligations, collateral and condition consequences.

**Build:** first a production forward, then an unsecured loan, then a secured loan.
Keep financing and delivery explicit: a prepaid forward exchanges funding now for
goods later; a forward with payment at delivery is a different agreement. Exercise
fixed terms before adjustable rates or transferable claims.

**Show:** a producer compares declining, self-funding, borrowing and forward sale.
Forecasts cannot sell the same harvest twice. Loan origination creates matching
claims; principal repayment, interest accrual and interest payment remain distinct.
A missed harvest causes a dated shortfall, arrears or default rather than invented
output. Interest does not mint settlement coins merely by accruing. Repossession,
resale, surplus return and residual debt follow explicit terms. Preserve attached
crop work obligations and distinguish collateral value from crop value.

**Failure signal:** forecasts authorize resources execution cannot deliver, paying
interest silently reduces principal, or collateral transfer erases unrelated claims.

## Stage 4 — Formation, mandates and organizational budgets

**Needs:** stages 0–3 for the financial examples; law and formation rules from goals.

**Build:** a household and a workshop cooperative under different legal templates.
The constitution is the template; the charter fills its parameters and remains
static initially. Persons hold governance roles and select operational policies
within that boundary. Separate membership, ownership, governance and valuation.
Start with the household's 20% labor contribution and explicit allocation tiebreaks.

**Show:** contributed hours are unavailable for simultaneous personal sale. A group
license enables only authorized organizational work, not every member's private
work. A household can buy food or service a member's debt only under a valid
mandate and finite budget. Policy comparisons change allocation without changing
opening resources or law. A posted production opportunity can motivate formation
in a bounded candidate search. Reject unlawful formation and prohibited charter
parameters. Death/dissolution retains claims in an estate pending legal disposition.

**Failure signal:** organizations gain free labor, leaders rewrite founding terms
to escape debt, or an owner automatically receives custody or operational control.

## Stage 5 — Custody, clearing, netting and finality

**Needs:** stages 3–4, explicit claims and legally permitted settlement mechanisms.

**Build:** custody without title transfer; gross settlement; then bilateral netting
of eligible obligations; then a bounded multilateral clearing cycle. Define which
parties, denominations, due dates and agreements belong to each netting set. Do not
collapse different currencies or unrelated legal claims merely because values match.
Distinguish payment netting from termination/closeout after default.

**Show:** for same-date same-currency claims A owes B 10 and B owes A 7, an authorized
net settlement of 3 discharges the specified claims with auditable gross provenance.
For A→B→C→A obligations of 10 each, a legally authorized multilateral offset can
close the balanced cycle without minted cash. Break one leg and demonstrate the
remaining exposure; netting cannot erase an unbalanced debt. Compare gross versus
net liquidity needs against the same opening obligations.

Exchange of value needs an explicit linked-finality rule: either both linked legs
settle or neither does. Legal finality also specifies when discharge becomes
irrevocable within the simulated jurisdiction; an atomic memory update alone is
not that legal model. These distinctions follow the concerns in the
[CPMI–IOSCO financial market infrastructure principles](https://www.bis.org/cpmi/publ/d101a.pdf),
used here as design prompts rather than a claim of regulatory compliance.

**Show additionally:** custodian failure does not make client assets its own estate;
a failed settlement releases only permitted reservations; later correction uses
new events rather than deleting finalized history. A default during a clearing
window has a declared loss allocation and cannot retroactively fund earlier work.

**Failure signal:** ledger reduction is mistaken for legal netting, custodian
inventory duplicates beneficial ownership, or batch order decides irrevocability.

## Stage 6 — Banking and liquidity

**Needs:** stages 3–5; claims distinct from settlement assets and equity.

**Build:** a deposit-taking bank with withdrawals and lending; a second bank and
interbank settlement; then a central bank and a bounded lender-of-last-resort offer.
Loan-created deposits, if allowed, create a loan asset and deposit liability; they
are not transfers of pre-existing coins. External payments still require permitted
settlement assets or explicit interbank credit. Keep cash lending as a comparison.

**Show:** a solvent but illiquid bank differs from an insolvent bank under stated
valuations. Withdrawals and payment queues cannot spend the same reserves twice.
Emergency loans require eligibility, terms, collateral where applicable, and an
authorized funding/issuance source. They do not repair solvency by definition.
Compare assistance, no assistance, runs and counterparty default. Losses have owners.

**Failure signal:** every financial asset spends like cash, all deposits are silently
state-guaranteed, or a rescue transfers losses to nobody.

## Stage 7 — Securities and investment institutions

**Needs:** stages 2, 4–6; transferable claims, custody and distribution rules.

**Build:** a joint-stock firm, primary share issue, secondary sale and dividends;
a government bond with coupons, maturity and default; then a fund owning a small
portfolio. Add multiple currencies and foreign exchange with linked delivery.

**Show:** issuance funds an issuer while secondary trading pays the previous holder.
Buying shares does not rewrite a constitution or automatically confer employment.
Dividend obligations respect declared distribution rules and actual funding.
Portfolio valuation is not spendable income. Consolidated reports eliminate
internal claims without deleting legal entity ledgers. Foreign exchange changes
denominated holdings without manufacturing either currency.

**Failure signal:** fund shares and underlying assets are counted twice as external
wealth, or market price changes directly credit bank balances.

## Stage 8 — Contingent promises, margin and insurance

**Needs:** stages 3–7 as applicable; observable triggers and bounded exposure.

**Build:** a simple insured loss with premium and finite claim capacity; an option
with exercise/expiry; a futures-style contract with defined marking and margin;
then interest-rate/currency swaps and credit default protection. Begin with tiny
hand-calculated payoff cases before endogenous pricing or intermediary networks.

**Show:** a contingent exposure exists before any payment is due. Trigger sources,
exercise, settlement denomination and timing are explicit. Premiums, collateral,
variation payments and realized losses are not interchangeable. Shared pledged
assets cannot cover unlimited promises. An insurer or protection seller can fail
when correlated claims exceed available resources. A margin call can force a sale
or default without silently cancelling unrelated final transfers.

**Failure signal:** only successful payoffs are representable, every contingent
claim requires a new unrestricted mutation path, or protection guarantees recovery
regardless of the protection seller's resources.

## Stage 9 — Pools, structured claims and shared infrastructure

**Needs:** stages 4–8, custody, default and auditable cash-flow allocation.

**Build:** a small loan pool with one class of claims, then a two-priority waterfall;
a funded liquidity facility for short-term paper; a securities depository; then
an optional central counterparty that explicitly assumes contracts and allocates
member-default losses. A clearinghouse need not be a central counterparty.

**Show:** every pool distribution traces to actual receipts or explicit financing.
Prepayment/default changes downstream cash flows; a waterfall redistributes losses
rather than removes them. Short-term liabilities cannot be repaid by an illiquid
valuation. Counterparty substitution, margin, default resources and replenishment
are explicit obligations. Separate exchange matching, custody, clearing, settlement
and supervision even when one organization performs several roles.

**Failure signal:** securitization creates another copy of the same loan asset,
central clearing hides exposures, or a pooled rescue fund has unlimited resources.

## Coverage targets for the later stages

These are deliberately minimal experimental analogues, not reproductions of all
real-world legal or operational detail. Each still needs the four evidence levels.

| Target family | Prerequisites and discriminating demonstration |
| --- | --- |
| Banks, mutual savings banks, credit unions | 4–6: vary member/owner claims and governance while preserving deposit and loan accounting |
| Central banks, last-resort lenders, paper currency | 1, 5–6: authorized issuance/redemption, interbank settlement and finite or explicitly issued emergency funding |
| Joint-stock companies, stock exchanges, government bonds, high-yield bond markets | 2, 4, 7: primary versus secondary flows, rights transfers, coupon/default and risky-debt valuation |
| Bills of exchange, letters of credit | 3–5: transferable payment claims; separately, a conditional bank undertaking with document/acceptance rules and failure outcomes |
| Merchant banks, investment banks, primary dealers | 4, 6–7: advisory/placement fees, underwriting commitments and funded dealer inventory are separate contracts |
| Venture capital and private equity firms | 4, 7: staged funding, constrained control rights, illiquid holdings and realized exit distributions |
| Sovereign wealth funds, index funds, hedge funds | 4, 7–8: different mandates, portfolio constraints and leverage using the same owned assets and liabilities |
| Money market mutual funds, REITs, ETFs | 5, 7, 9: liquid-claim versus underlying-asset mismatch; property income; basket creation/redemption distinct from exchange trading |
| Insurance and CDS | 3–4, 8: premium, observable covered event, claim priority and protection-provider default |
| Futures and options exchanges | 2, 5, 8: expiry/exercise, delivery or cash settlement, margin where defined, and participant failure |
| Interest-rate and currency swaps; foreign exchange (Forex) | 5, 7–8: scheduled contingent legs, separate currencies, linked settlement and bounded collateral |
| Mortgage-backed securities, asset-backed commercial paper | 3, 7, 9: underlying loan/asset receipts, pool claims, waterfalls and refinancing failure |
| Clearinghouses and OTC derivatives clearing | 5, 8–9: admissible netting, optional counterparty substitution, collateral and default loss allocation |
| Depository Trust & Clearing Corporation (DTCC)-like infrastructure | 5, 7, 9: compose depository, clearing and settlement services with distinct responsibilities; no omnipotent finance agent |
| Credit bureaus | 3–4: permissioned dated reporting, incomplete information and errors affect beliefs without changing the underlying debt |
| Financial Industry Regulatory Authority (FINRA)-like oversight | 4, 7–9: delegated membership/supervision rules, observations, violations, sanctions and review; not unrestricted state powers |

## Stage 10 — Alternative civilizations and composed stress

Use the same mechanisms with different catalogs, observations, constitutions,
charter parameters and policies. Run paired fixtures differing in one assumption
before combining differences. Each comparison must preserve an explicit opening
resource and claims baseline.

| Variant | What must be shown |
| --- | --- |
| Perfect global pricing knowledge | Agents observe every current eligible price, but not future shocks; knowledge does not grant market access, transport, stock or money |
| A species needing rest rather than food | A dated rest process satisfies its need and uses time/opportunity capacity; food-specific planner/accounting branches are unnecessary |
| Credit and netting without minting | Starting with no currency issuance, parties accept explicit claims under credit limits; authorized offsets discharge cycles, while unbalanced claims remain, transfer by agreement or default |
| Different lending norms | Zero interest, fixed interest, contingent repayment or prohibition change available agreements; alternative terms preserve claims and consequences |
| Different land and organizational law | Lease versus purchase, licensed generic firms versus specialized entities, and alternative estate rules change feasible plans without scheduler forks |
| Different leadership/allocation norms | Rotation, election or permitted hereditary rules change who sets policy; members, owners and contributors retain distinct identities and budgets |
| Local information and market segmentation | Observation delay and admission limits affect choices; observed and realized prices remain dated and comparable |
| Nested organizations | A coalition of states delegates resources by the same mandate principles as persons in a household, without recursively multiplying assets or work |

A moneyless credit fixture must distinguish its accounting unit from a circulating
settlement asset. Issuing a claim is not minting, but it does create credit exposure.
Neither perfect information nor circular debts can satisfy physical needs without
actual goods or services.

Then combine small networks: households, producers, a market, two lenders and an
issuer/clearing service. Stress harvest failure, simultaneous withdrawals, price
gaps, unavailable collateral buyers, correlated defaults, death, dissolution and
loss of market access. Compare gross/net settlement, price policies, governance
policies and emergency support against matched controls. Report who loses assets,
who misses needs, which obligations survive and which institutions cease operating.
Do not use a single wealth or default-rate score as proof of success.

## Immediate sequence and review gates

1. Audit the existing minting, market, forward and loan pilots against stages 0–3;
   retain their documented exclusions. They are starting evidence, not a completed
   integrated financial system.
2. Extend the current generated-order pilot toward multiple eligible bids/asks,
   monthly posted-price observations and explicit no-match outcomes. Keep supplied
   valuations as a control before asking ZIP or projections to determine them.
3. Exercise production-funded obligations together with deprivation consequences,
   then integrate one forward/loan comparison under shared resource reservations.
4. Develop household contribution/authority and static charter parameters in parallel;
   integrate a household as an eligible market participant only after its budget
   and mandate receipts are explicit.
5. Add a tiny custody and bilateral-netting case, followed by the three-party
   moneyless cycle. Use this to expose cash-only assumptions before adding banks.
6. Advance one later-stage mechanism at a time, including a failure case and a
   composition control. Keep a coverage register naming stage, evidence level,
   test/report links, unsupported combinations and the next unresolved primitive.

Progress means that a new institution can mostly be defined through agreements,
rights, mandates, observations and policies while reusing reliable execution and
accounting. When it cannot, document the precise missing abstraction and repair it
at the smallest useful scope. Do not build a universal solver in advance, and do
not conceal a missing primitive behind another specialized institution update loop.
