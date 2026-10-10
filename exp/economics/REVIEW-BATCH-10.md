# Ten follow-up review iterations

Start: `be31835`, following [the twenty-iteration batch](REVIEW-BATCH-20.md).
Each iteration receives independent review, focused verification and a separate
source-only commit. Defaults remain unchanged unless explicitly noted. Individual
self-directed policy changes remain deferred.

## 1. Private food sales through the existing market

An opt-in `discovery.private_sales` policy discovers person-owned surplus food asks
on the existing stock listing. It uses accepted claims and projected needs to
protect inventory, including claims-only protection for passive stock owners.
Private trades settle in the existing market ledger and accounting driver, even
when minting is idle or the issuer cannot trade. Failed mint-input packages do not
cancel independent food sales. Private purchase budgets reserve at the buyer's
limit so unequal ask prices cannot consume money protected for obligations.

Focused controls cover finite stocks/cash, denied permission, issuer exclusion,
passive owners, unequal prices and accepted coin claims, with CPU/reference and
phase-continuation accounting comparisons. This remains a bounded single-food
listing with configured valuation limits, not general market discovery or ZIP.

Verification: four new private-sale tests, twelve existing mint-order tests,
thirteen public-sale tests and three circulation tests pass. Independent review
identified the unequal-price budget and passive-owner cases before commit; both
have regression coverage. Failed mint packages retain actual private food sales
in the audited CPU/reference control.

## 2. Household-owned surplus and dated protection

Eligible active households can post food asks under the same opt-in policy. Their
supply comparison includes every current member's needs and failed processes.
A shared reserve reader handles participants, passive owners and households.

The integration control exposed an actual boundary issue: household distribution
moves protected food to members before market clearing. Keeping the original
collective stock floor would reserve the same food twice. Discovered household
quotes now freeze the authorized lot ceiling and recheck the collective/member
reserve at the market boundary; explicit user quote floors remain binding.
Matched with/without-claim controls use identical stocks and horizons and include
claims owed by both household and member. Scheduling and household distribution
order are unchanged.

The opt-in path also honors existing collective purchasing charters: members do
not bid against their household for food. Five private-sale and thirteen supplier
forecast tests pass, including identical-horizon household/member claim controls
and audited CPU continuation. Collective buying is the next bounded adapter.

## 3. Collective consumption bids

The private-market option now lets households buy food under their existing
collective purchasing charter and permitted needs-oriented policy. A bounded
whole-lot search reuses the ordinary consumption and claim readers, offsets member
private food, and chooses the smallest count attaining its best non-worsening
consumption result. It does not assume future harvests, wages or market fills.

The quote preserves its lot authorization. Clearing protects collective and
member monetary claims, then enforces real opening money and storage. A household
with no money can express demand without receiving an invented purchase. Wealth
policies do not acquire a new food-buy mandate.

Eight private-market tests, twelve household-market tests and eleven need-order
tests pass. Coverage includes multi-lot minimum useful consumption and collective
cash protection for an unfunded member claim. The coin-delivery/metal-prepayment
claim fixture checks physical simulation only: that denomination swap is outside
the reporting-coin-advance/noncoin-delivery valuation adapter. Normal food trades
retain full audited CPU/reference comparisons; this is not broader FX support.

## 4. Composed private circulation under matched scarcity

Added an eight-case [private circulation comparison](PRIVATE-CIRCULATION.md):
public stocks 16/8 × food lots 3/1 × private policy off/on. Household venue admission
is explicit and identical across variants. Under scarce public supply, household
sales reach the worker; worker deficits fall 11→3 with three-unit lots and 11→1 with
one-unit lots. Large public stocks produce no private fills, which prevents
attributing every improvement to new sellers. Every case retains endogenous
formation, actual loan repayment, annual dues and separate CPU/reference books.
Finite income and timing remain unresolved; this is a calibrated integration test.

## 5. Recheck personal claims after financial admission

Private sellers now recheck live needs and accepted obligations at clearing, as
households already do. A forward accepted after Open can reduce an earlier sale
authorization; a declined offer or delivery outside the protection horizon cannot.
The original quote floor and lot ceiling still bind. This fixes a reproduced
next-month food shortage without moving financial or market scheduling.

Nine private-sale tests and four circulation tests pass, including audited
CPU/reference continuation across the accepted/declined/outside-horizon controls.
An older unequal-price fixture now supplies genuine surplus over its horizon.

## 6. Seller authorization receipts

Stock-sale plans retain opening available stock, the explicit floor and authorized
lot ceiling, effective live protection, eligibility, feasible/submitted quantities
and matched lots. Public inventory records supply separately from demand-capped
submission. These are domain receipts; they do not execute additional decisions.
A permission revoked after Open retains a feasible but unsubmitted authorization.
Late financial admission records exactly why an authorized lot becomes unavailable.

Ten private-sale tests and thirteen public-sale tests pass. Independent review
requested the post-Open revocation control, now included. Matched quantities are
planning results; final accepted settlement remains a separate receipt.

## 7. Observe private stock budgets and accepted settlement

External settlement observers export seller budgets and private-mode purchase
budgets, including actual accepted quantities from either public or peer sellers.
Legacy public-only purchase records retain their name and filtering. Selecting a
private buyer or seller exposes its own budget; selecting the issuer does not
export unrelated person-to-person purchases. Observer replacement does not replay
old settlement rows. Eleven private-sale tests and thirteen public-sale tests pass,
including observed/unobserved CPU state, ledger, reports and accounting equality.
The independent reviewer identified the issuer-filter mismatch before commit.

## 8. Independent stock-policy horizons

Fixed an independently reproduced interaction: enabling public sales with a short
claim horizon previously shortened private sellers' reserves as well. Each seller
now uses its own configured horizon. Buyer cash can cross either source, so its
claim protection uses the longer enabled horizon. Five controls cover both
asymmetric configurations, each policy alone and equal short horizons. Twelve
private-sale tests and thirteen public-sale tests pass; the financial reservation
fixture retains its explicit physical-only denomination limitation.

## 9. Forty-month circulation and the remaining income constraint

Extended the scarce-public-stock/unit-lot comparison through forty months without
new endowments or authored jobs. Both variants repay the loan, renew the accepted
lease in month 26 and pay dues in months 14 and 38. Private access reduces worker
deficits from 36 to 25, but paid work stops in month 6. The private variant spends
all twelve earned coins on food by month 13; later receipts show feasible household
supply and an unaffordable worker bid. The public-only variant retains eleven coins
but records funded unfilled bids when no public surplus is available.

The integration test passes with exact outcomes, nonempty accepted contracts and
paid obligations, stock/cash reconciliation, CPU/reference financial books and
reconstruction at every phase. Independent review checked renewal and failure
attribution. This demonstrates the recurring-income gap; it does not resolve it
or claim long-run viability. See [the comparison](PRIVATE-CIRCULATION.md).
