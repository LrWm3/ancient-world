# Credit capacity: observed limits and decision receipts

The 200-year held-out runs from `547aaba` have completed. Their retained
underwriting rounds explain submitted rejections; they do **not** explain every
reason a pilot failed to submit a request.

| Seed/arm | Submitted requests | Approved | Zero net repayment source | No lender surplus |
| --- | ---: | ---: | ---: | ---: |
| 256 credit | 0 | 0 | 0 | 0 |
| 409 credit | 0 | 0 | 0 | 0 |
| 1024 credit | 6 | 0 | 6 | 5 |
| 256 combined | 0 | 0 | 0 | 0 |
| 409 combined | 1 | 1 | 0 | 0 |
| 1024 combined | 58 | 0 | 58 | 5 |

The last two columns can overlap. They compare dated observed receipts with
protected operating costs, and lender cash with its reserve, before existing
pledges or exposure caps. Every rejected request in this ensemble already has
zero net source capacity at that first check.

At seed 1024 month 13, credit-only requests 2,156.23 against 18.85 expected export
receipts and 2,353.29 protected operating costs. The lender has 1,258.98 cash
against a 1,406.38 reserve. Both constraints rule out lending. At month 220 the
request is only about 0.007, but receipts of 6.06 still fail to cover protected
costs of 121.74. Small requested principal does not make a repayment source sound.

Seed 409 combined's accepted month-698 request is different: principal 2.024,
expected receipts 88.275, operating costs 9.974 and lender cash surplus 3,198.243.
It is a feasible small bridge under the existing policy. See the
[ensemble report](monetary-estates-heldout.md) for repayment and later outcomes.

## Persisted capacity details

New grants record an optional capacity snapshot at the completed underwriting
boundary:

- Lender principal capacity after cash reserve, voluntary offer and exposure cap.
- Borrower principal capacity after existing borrowing.
- Repayment-source capacity after operating costs, coverage margin and pledges.
- Total eligible demand on each of those three pools.

Source capacity/demand includes contracted interest; lender and borrower amounts
are principal. The grant remains eligible principal multiplied by the smallest
capacity/demand ratio, capped at one. Recording all pools exposes simultaneous
constraints without inventing a single winning explanation. Actual transferred
cash remains in the existing disbursement receipts and may be smaller due to
account representation.

Pre-capacity eligibility failures have no snapshot. Older archives load with
`None`; missing historical measurements are not reconstructed from current
loan balances. Validation checks finite nonnegative capacities, positive demands
and agreement between the snapshot and granted principal. Independent analytical
fixtures isolate lender, borrower and source limits, simultaneous exhaustion,
serialization and old-record loading. The existing allocation-order fixture
continues to compare complete receipts with reversed request/offer ordering.

## Next policy investigation

Commercial operating costs currently protect a town's entire next planned input
gap, not just the delivery contract being pledged. That is conservative, and can
exclude a feasible small dedicated project. It is not evidence that all those
costs should be removed: town cash also funds other production.

A subsequent experiment should distinguish a genuinely funded/senior input
commitment from an unfunded planning wish, then test a bounded project with actual
procurement, work, delivery and repayment. Record why requests were not submitted
as well as why submitted requests were rejected. Do not reduce reserves, ignore
operating costs or increase issuance merely to obtain nonzero loan counts.

## Verification

Five underwriting unit fixtures and all fifteen non-GPU market integration tests
passed, including proportional actual-cash funding, replay rejection, servicing,
defaults, issuance conservation and retained abandoned-town accounts. The two
GPU-only market fixtures were not run in this diagnostic-only pass. Strict
all-target Clippy passed. Lending policy, forecast construction and allocation
arithmetic are unchanged; no fresh balance ensemble was needed to select new
parameters because this change selects none.
