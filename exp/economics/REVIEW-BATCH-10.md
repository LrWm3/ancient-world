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
