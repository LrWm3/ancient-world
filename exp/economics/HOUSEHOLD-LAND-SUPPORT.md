# Household funding of current land dues

Implemented integration pass 28, September 2026. The optional
`Charter.accept_payment_support` now recognizes a household's own unpaid land bills
alongside earned wages and current loan dues. A member's signed surplus mandate can
fund these obligations without taking over the household's liability.

Pass 29 adds [market purchases and explicit tender preference](HOUSEHOLD-LAND-FUNDING.md).
The native-only funding behavior below describes the default and pass-28 controls.

## Shared claim rules

`commitments::current_claims` supplies the native land demand used by proportional
credit collection and by voluntary support. Ordinary land settlement uses the same
eligibility gate. The collector supplies its staged recovery book, so an estate
opened at this boundary already affects collection eligibility.

- At Due, use the existing annual bill-generation rules. At other boundaries, read
  already issued bills only. Future anniversaries are not advanced into current debt.
- Count outstanding amounts after actual payments, retaining old arrears even after
  a right expires. A posted, unaccepted land offer creates no bill.
- Exclude terminal debtors and land claims assigned to an active estate in that
  native denomination. Native performance in a different denomination retains its
  existing collection path.
- An accepted coin alternative remains a settlement option, not an additional coin
  debt or an exchange-rate instruction to the donor. Funding targets native units.

No second obligation book is introduced. Claim views neither issue bills into live
state nor settle them. Land ranks, alternative tenders, estate custody and collection
allocation remain with the existing executors.

## Monthly integration

At the existing before-Productive support boundary, sum current wage, loan and land
requirements only within the offered resource. Offset actual household holdings
once, then cap acceptance at the remaining shortage and the member's authorized,
protected, storable surplus. Existing consumption/income benefits still take priority
over this fallback. The charter flag still defaults to false.

The donation records a transfer expense and income on separate books. Dues payable
and receivable remain unchanged until actual payment. Physical goods retain carrying
cost; no new accounting adapter or creditor relationship is created.

Timing remains significant: land has an existing **ClearArrears** pass immediately
after Productive, wages collect at Close, and loans funded after Due wait for the
next month's Due. This is not simultaneous allocation across all creditor kinds.
In particular, partial support can clear rent while leaving wage arrears.

## Combined evidence

The supplied fixture has a household owing four grain in annual rent and six grain
in earned wages. A member holds fourteen grain, protects two, and authorizes limited
surplus support. Needs and further production are disabled to isolate financing.

| Control | Observed outcome |
| --- | --- |
| Support disabled | Neither rent nor wages receives a donation |
| Enabled with fourteen private grain | Ten donated; all claims remain at Productive; ClearArrears pays four rent; Close pays six wages; member retains four |
| Enabled with seven private grain | Five donated; four rent and one wage paid; five wages remain owed; member retains two |
| Creditor has room for zero or two grain | Rent payment is storage-capped; undelivered grain remains in the household with its unpaid claim; next month requests no duplicate donation |
| Month before the first bill | No rent donation, despite accepted future annual terms |
| Three of four rent units already paid | Only one donated and paid |
| Rent fully paid | No rent donation |
| Coin mandate, grain rent with accepted coin alternative | No inferred coin funding target |
| Coin mandate, native coin rent | Four coins donated and paid |

CPU/reference runs agree on state, receipts and financial statements. Stable member
ordering survives reversed member tables. Checkpoint continuation and ledger replay
agree. A focused reader test checks staged estate exclusions and read-only bill
preview. Existing recovery tests cover native performance, estate cash allocation,
alternative tenders, storage and delivery relief after the shared-reader extraction.

## Verification and limits

The selected regression run and final focused run passed **99 distinct integration
tests across 11 suites**, plus **four claim-reader unit tests**. These include four
new integration tests and one new unit test. Formatting, strict all-target Clippy
and repository artifact checks passed. The full crate suite
was not run. Generated logs stay under ignored `output/economics/land-support-*.log`.

The new cases are in [household payment support tests](tests/household_payment_support.rs).
From this directory, run `cargo +1.92.0 test --locked --test household_payment_support`
and `cargo +1.92.0 test --locked --lib commitments::candidate_tests`. The broader run
also covers commitments, finance, recovery, estate/ordinary dues accounting, household
finance/credit/composition, member employment and need orders.

These are supplied agreements and consent terms, not autonomous household land
acquisition or demonstrated business viability. Support does not reserve funds for
a named creditor or promise future donations. Pass 29 adds purchases and explicit
alternative-tender funding; forward support remains open. The current forward
admission path still requires person participants in a tool purchase;
this pass does not bypass that restriction. Person self-directed policy changes
remain deferred.
