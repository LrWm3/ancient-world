# Native commodity finance and guarantees

Fibonacci batch 21 extends the existing credit book and financial statements.
Commodity loans no longer require a separate reporting path. Physical wage, land
and direct prepaid-delivery guarantees use actual transfers and the same recourse
loan records as coin guarantees.

## Valuation and statements

`Opening.exchange_values` supplies positive, fixed reporting ticks per native unit.
It is an explicit reporting convention, not a market quotation, conversion trade
or promise of liquidity. Principal and accrued interest keep their denomination
in the simulation and are valued symmetrically on both parties' statements.
Missing values reject reporting atomically before publishing the simulation step.

Advances, repayments and guarantee performance consume the shared opening
inventory-cost allocation. Incoming inventory cannot finance another disposal in
that boundary. A lender acquires a valued receivable and releases inventory basis;
a borrower receives valued inventory and records the corresponding liability.
Differences between disposed historical basis and fixed claim value appear as
settlement gains/losses. Principal transfers create neither sales nor cash flows.
Interest remains separate income/expense in the reporting denomination.

Commodity collateral and foreign-denomination estate cash still require additional
valuation adapters. Supporting unsecured native lending does not imply FX markets,
mark-to-market accounting, general liquidation or automatic conversion of claims.

## Native claims during coin insolvency

An authorized coin proceeding admits unsecured loans in other native denominations.
Interest freezes and their full outstanding quantity becomes collectible at Due,
but coin custody and distribution include only coin-denominated loan claims.
Native loans and guarantee recourse remain explicit closure blockers, even when
coin deficiency discharge is authorized. Real native repayment uses ordinary
ranked collection; no appraisal or reporting value becomes a conversion rate.
Commodity collateral still requires a separate enforcement adapter.

The combined control starts with five debtor coins and four unpaid firewood wages.
A guarantor delivers four firewood units after the coin proceeding opens. The five
coins remain untouched. The debtor later earns four firewood units for real work
at Close and repays native recourse at the following Due; only then does the
proceeding close. Both discharge settings, separate statements, forged closure,
CPU/reference and checkpoint continuation are covered.

## Guarantees

Accepted terms can cover an original loan, one earned wage, one annual land bill,
or a direct forward identified by its supplied original terms. Actual payment
reduces the covered obligation and creates equal native-unit recourse. New recourse
is collectible at a later Due boundary. Caps, priority, optional proportional
allocation, protection, storage and accepted relief still apply.

Physical wage receipt is household income. Before paying it, guarantee execution
reserves both private storage and the mandatory pooled share, including fractional
carry. Non-income guarantee deliveries update that storage window without pooling.
Blocked quantities remain owed; no inventory is destroyed to make the payment fit.

A physical land guarantee records real native payment and can support the existing
native-linked issuance rule. It consumes guarantor inventory, not imaginary debtor
inventory. Its dues-unit valuation must agree with the recourse resource valuation;
inconsistent reporting conventions reject explicitly.

## Forward timing and historical cost

The ordinary delivery window is Acquire. Guarantees execute at Due and first become
callable in the month **after** the effective delivery month, plus their configured
delay. A seller therefore retains the first opportunity to perform. Later accepted
extensions move that eligibility date but do not extend the guarantee's validity.
Configured guarantees are consent, not autonomous guarantee-market discovery.

The buyer retains the historical prepaid acquisition cost. The seller recognizes
its prepaid sale and the cost of the guarantor-funded performance. The guarantor
recognizes a native receivable and any difference from its disposed inventory basis.
These values need not coincide: five coins prepaid for four goods can coexist with
three reporting ticks per unit of recourse. Multiple guarantee calls release the
historical basis cumulatively, without losing integer rounding ticks.

Ordinary and guaranteed forward performance retain separate dated boundaries.
Guarantees of dynamically underwritten tool forwards, recourse guarantees, pending
resale mortgages remain unsupported. Authorized-liquidation lien subrogation
now has an explicit adapter; see [lien priority](LIEN-PRIORITY.md).

## Verification

`tests/commodity_lending.rs` covers physical advances, interest, partial/full
repayment, guarantee recourse, explicit valuation failure, separate statements and
CPU/reference/checkpoint consistency.

Additional `tests/guaranteed_claims.rs` controls cover:

- Physical wages blocked by a full communal store, preserving unpaid claims.
- Physical land payment without debtor goods or fabricated cash.
- Partial ordinary delivery followed by residual guarantee performance.
- Overlapping guarantees preserving five prepaid cost ticks across four goods.
- An accepted extension outlasting guarantee validity, leaving the claim unpaid.
- Reordered terms, separate native exposures and denomination-bearing telemetry.

The run record and final validation results are in
[Fibonacci integration](FIBONACCI-INTEGRATION.md). Logs remain ignored local artifacts.

Land guarantees may now explicitly select the original agreement's accepted coin
tender. Actual coins discharge whole native claim units; no goods are received,
and the guarantor's recourse remains native. Both actual tender and claim units
appear in receipts. See [guarantee tender](GUARANTEED-CLAIMS.md#accepted-coin-tender-for-land-guarantees).
