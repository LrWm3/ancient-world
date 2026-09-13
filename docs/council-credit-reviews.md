# Council credit opportunity reviews

The council pilot now retains the latest enabled decision boundary for each
council, including cases that never become underwriting requests. It also counts
construction outcomes cumulatively. This is diagnostic state; funding rules,
interest, tax forecasts and spending order are unchanged.

| Outcome | Meaning |
| --- | --- |
| NoCashGap | Opening treasury covers the current administration plus lagged relief forecast. |
| MissingTaxEvidence | No usable completed annual observation or current collectible ceiling. |
| InvalidCollectionWindow | The collection/maturity window cannot support a dated request. |
| NoContactedLender | No council with a voluntary surplus offer is reachable through an eligible direct route. |
| Submitted | Requests were constructed; underwriting may still reject every one. |

A review records opening cash, monthly demand and cash gap. Where evidence exists,
it also records expected taxes, annual support/road commitments, and the annualized
monthly operating forecast. It records the number of eligible contacted lenders,
not the number of loans. Multiple lenders produce multiple requests but only one
council review for that month.

## Why twelve months of operating costs remain

The pilot pledges the next annual tax collection. That collection also supports
the next operating year. Annual support and road requests are deducted separately
from twelve months of administration and the most recent relief demand. The latter
is a conservative persistence assumption, not a measurement of twelve future
months. Shortening it to the time remaining until maturity could treat money needed
for the following year's services as free debt-service capacity.

This audit therefore retains the cost horizon. A better forecast could distinguish
persistent administration from temporary relief, but needs dated demand evidence
and matched service-shortfall tests before changing underwriting. The first smoke
comparison already demonstrated default despite adequate gross tax collection
when road commitments were omitted; see [that comparison](credit-council-smoke.md).

## Limits and persistence

`Credit.council_reviews` contains the latest processed boundary, not a time series.
`Credit.council_review_counts` counts council-months since introduction of this
record. Older archives initialize both empty; no retrospective skipped decisions
are invented. Disabled policy leaves the last dated review unchanged. Repeating
an already processed month leaves both unchanged. These counts cannot identify
unique projects or prove the absence of economically useful borrowing opportunities.

Use the existing underwriting rounds for requested, eligible and granted principal,
capacity limits and actual committed loans. The review explains construction;
it does not replace underwriting or its repayment evidence.

Verification: all 19 active market tests and seven reporting tests passed, as did
strict all-target Clippy. Controlled cases cover missing evidence, disabled credit,
closed routes, a lost current tax base, annual road obligations, successful funding,
invalid diagnostic amounts, and same-month serialized continuation. The two
extended market tests remain ignored in that command. The [four-arm seed-1024 comparison](council-credit-review-evaluation.md) also
completed; it verifies unchanged existing values but finds no successful lending.
