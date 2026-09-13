# Commercial requests after eligibility filtering

The commercial pilot previously divided a town's opening funding gap by the
number of export repayment sources, then skipped requests with unsupported
maturities or unavailable counterparties. A skipped source still occupied a
share of the original gap. This could suppress a feasible bridge without any
inventory, lender-cash or borrower-debt constraint requiring that suppression.

Requests now pass those construction checks first. The one opening gap is split
equally among the remaining requests for that borrower. The existing underwriting
resolver still applies risk, source coverage, lender reserves, borrower exposure,
pledged receipts and simultaneous borrowing/lending restrictions. It does not
redistribute unused grants after underwriting or guarantee full funding.

The regression uses a two-unit input gap and an eligible delivery-backed source.
A matched branch adds a separately funded cargo whose arrival is beyond the
supported loan duration. Its cash comes from the actual buyer balance and its
goods from seller inventory. The valid source should still receive a two-unit
request and loan in both branches. Under the previous source count, its request
would have been halved before underwriting.

This changes request construction, not debt accounting or repayment promises.
It does not establish population benefits, profitable production after lending,
or suitable automatic operator financing. Long-run comparisons must distinguish
this correction from changes in interest, source coverage or issuance policy.

Verification: the automatic commercial-credit regression passed, as did both
commercial cost-sharing tests and all 13 export-contract tests (which include
the regression). Strict all-target Clippy passed. Existing controls still reject
disabled lending, absent funding need, fully reserved lender cash and unsupported
or delayed delivery dates. Raw logs remain under ignored `output/`.
