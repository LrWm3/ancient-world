# Physical minting within the shared contract economy

Implemented in Fibonacci batch 13. Physical issuance now composes with direct
loans, direct prepaid deliveries, accepted annual land agreements, employment,
households and authorized recovery. These run through the existing simulation,
claim books, accounting and CPU settlement. This supersedes earlier blanket
statements that minting requires isolation from those arrangements.

## Execution and reservation boundaries

Open regenerates current capacities from monthly endowments and existing condition
modifiers. Due services existing financial claims and authorized recovery before
market decisions. Acquire reserves direct credit first, then due forward deliveries
and new prepayments, then mint-market packages. This is an explicit scoped priority;
it is not a universal fair allocator. Incoming resources appear in projected holdings
but never replenish the same acquisition window's spendable budget.

Generated fixed and provisioning orders use that remaining budget in both discovery
and matching. Dated material/capacity packages retain atomic acceptance: a failed
labor or metal leg cannot leave a partial purchase behind. Employment then reserves
only remaining eligible hours. Productive consumes real materials and capacity;
only completion of the authorized mint process creates coins. Close settles actual
earned wages. Annual dues and ordinary repayments transfer existing stock or coins;
they cannot issue currency. Existing phase order is unchanged.

Household contributions use the same own-capacity endowment as Open, before market
or employment transfers. Conditions change at Close, so these monthly inputs remain
the same through Acquire and Productive. Purchased hours do not increase the member's
contribution; sold hours cannot erase it. Actual availability still caps reservations.
Mint matching reserves the household contribution and the storage needed by income
pooling. Only actual trade receipts are pooled, once, with existing fractional carry;
loan principal and unpaid claims are excluded. Public standalone order previews use
the same household reservations; for a financed boundary use `acquisition::evaluate`
to include earlier contract reservations as well.

New mint-market commitments reject a party in active authorized recovery. Custody
agents cannot appear as issuers, scripted counterparties or generated quotes.
Existing native performance and creditor distribution retain their existing rules.
This does not add a new automatic insolvency trigger or liquidation market.

## Combined scenario and results

`tests/mint_finance.rs` includes a six-month run, months 12–17, with two people,
a household, a state issuer, food/metal/labor orders, a three-coin loan, one prepaid
food unit, and a two-coin annual land bill. The household pools purchases and earnings.
Two existing provision policies start from identical holdings and contract terms:

| Outcome | Full buffer | Incremental |
| --- | --- | --- |
| Prepaid food delivered | 1 | 1 |
| Loan | Repaid | Repaid |
| Annual land bill paid | 0 | 2 |
| Land arrears retained in both parties' books | 2 | 0 |
| Physical coin issuance | None | Positive |
| CPU/reference/checkpoint reconciliation | Pass | Pass |

This is a controlled composition test, not evidence that either policy is generally
superior. Full-buffer planning can leave mutually necessary input trades unmatched.
A funded loan or an active marketplace does not establish sustainable production.
The default policy remains unchanged; person self-directed policy changes remain
deferred.

Thirteen focused tests also cover same-window loan/prepayment restrictions, a due
forward competing for mint metal, outgoing loans competing for procurement, actual
wage settlement, annual native and coin dues, recovery admission, custody exclusion,
member labor reservations, pooled-storage rejection and preview/settlement agreement.
Financial statements distinguish loan financing, operating payments and physical
issuance. They preserve unpaid claims instead of inferring payment from activity.

## Remaining scope

Mortgage-purchase admission, bilateral negotiation, town order books, legacy tool
underwriting, competing opportunity search and the mint order driver are still
separate acquisition configurations. This work does not combine multiple matching
algorithms into one market session. Optional household hiring offers remain outside
mint configurations; preaccepted employment is covered. Constitutions and charter
parameters remain static. Autonomous credit underwriting, general liquidation,
physical guarantees and wider legal arrangements still need their own adapters and
composed regressions. The stress-test proposal remains a long-term verification
program, not a completed release checklist.

Generated logs stay under ignored `output/economics/`. See
[Fibonacci integration](FIBONACCI-INTEGRATION.md) for validation and batch contents.

## Household offer follow-up

Posted household labor offers now run after actual mint-market reservations.
A paired control completes the mint when its material/labor package is feasible;
otherwise the household hires the remaining two hours and produces four goods.
The two paths reconcile on CPU/reference execution. This removes the earlier
blanket exclusion for household hiring offers, not the other market-driver limits.
