# Receivables during recovery and household wind-down

An estate with a loan deficiency now checks existing receivables before closing.
The view includes outstanding loan principal/interest, materialized land bills,
accepted prepaid deliveries and actually earned wages owed **to** the debtor.
It excludes hypothetical future rent, payroll and production. Native quantities
remain in their denominations; the view creates no coins or converted claims.

Ordinary contract servicing continues to collect these assets. Receipts go to
their existing legal creditor, then eligible cash enters the existing estate
custody window. New receipts cannot be spent again in the same boundary. A
closure check also examines actual post-transaction debtor cash, so collecting
the last receivable at Due cannot cause discharge before that cash is swept.
`AssetsPending` receipts report both uncollected cash and outstanding receivables;
settlement logs can be filtered by the receivable's counterparty.

The check applies while loan deficiencies remain. A fully paid estate can close
without liquidating unrelated financial property. Existing claims must be
performed or explicitly disposed of before deficient closure. A counterparty's
authorized discharge resolves the corresponding loan asset and records its loss
through the same double-entry adapter. There is no automatic bad-debt write-off
based solely on how long a receivable has remained unpaid.

Closure checkpoint validation rejects unresolved receivables predating a recorded
deficient closure. Recognition respects the phase boundary: wages earned at Close
or agreements admitted at Acquire after that month's Due closure are new assets,
not retroactive reasons to invalidate the earlier closure.

## Verification

Six focused tests cover:

- One coin collected in the closure month: wait for custody and distribution.
- Three coins collected over installments: recover all three before writing off
  the remaining seven of a ten-coin loan.
- An unfunded counterparty: retain the full receivable and defer closure.
- Two authorized estates: counterparty discharge recognizes the asset loss;
  the creditor estate then closes through its next ordinary boundary.
- A winding-down household: retain membership/property until collection and
  recovery finish, then permit dissolution without distributing creditor funds
  to its member.
- Earned wages: collect actual employer payment before deficient closure; later
  work remains a new, material receivable. Counterparty-filtered observer output
  explains the deferred closure.

CPU/reference, checkpoint and separate-account checks accompany the composed
cases. Fixtures explicitly start reporting from a distressed opening snapshot;
their initial cash depletion is not presented as a simulated expenditure. The
affected commodity, wage, mixed-claim, guarantee, household-dissolution and
recovery suites pass alongside these tests. Strict all-target Clippy passes.

## Remaining boundaries

This is collection and closure protection, not receivable trading, assignment,
netting, impairment estimation or automatic counterparty insolvency. No custody
of native goods or automatic sale of all debtor inventory is added. An unresolved
asset can therefore keep a deficient proceeding open until explicit performance
or disposition terms are supplied. Those broader liquidation adapters remain on
the consolidation roadmap.
