# Household land-bill purchases and tender preference

Implemented integration pass 29. This extends [native land support](HOUSEHOLD-LAND-SUPPORT.md)
through the existing need-generated market orders and voluntary support paths.

## Static charter choices

- `fund_land_dues`, default false: collective orders may buy missing stock for the
  household's own current land bills. The same finite money, goods, storage and
  protected-claim checks used for wage/loan funding apply.
- `land_tender`, default `NativeFirst`: collection tries the native resource, then
  an accepted alternative for any remainder. `AcceptedAlternativeFirst` reverses
  those attempts. Without an accepted alternative it falls back to native payment.

The preference is a funding and settlement choice, independent of whether market
funding or voluntary support is enabled. It does not negotiate new terms or convert
an obligation into another debt. A four-grain bill accepting two coins per grain
still owes four grain-equivalent units; paying eight coins discharges it once.
Only actual native deliveries count toward collection-linked issuance.

Current funding targets and market stock protection use one selected denomination
per bill. Alternative funding replaces current native protection; it does not add
an eight-coin demand alongside the four-grain demand. Future annual bills retain
native projection protection and do not generate current funding orders. An
active estate disables ordinary alternative preference; existing estate admission,
denominations and collection eligibility remain in force.

The existing `accept_payment_support` opt-in also uses the selected funding resource.
Signed coin support can therefore fund accepted coin rent. Donor reserves, consent,
membership, claims and storage checks remain applicable. Receipt `payment_funding`
quantities describe funding units in the offered resource, not a re-denominated
liability. Donations remain separate transfer income/expense until rent is paid.

## Timing and allocation

Town markets now permit already accepted land agreements. Land-offer discovery,
mortgage purchase configuration and recovery proceedings retain their existing
integration restrictions.

Due issues/collects annual bills before Acquire. Acquire can buy for the unpaid
current bill; voluntary support remains before Productive. Land collects again at
ClearArrears after Productive. Wages still collect at Close and loans funded after
Due wait until next month's Due. There is no scheduler change or same-batch reuse
of incoming sale proceeds.

Under proportional collection, alternative-first land claims compete with loans
and other claims at their existing rank in the shared currency pool. Only the
unfunded remainder requests native fallback after that currency allocation. Whole
alternative-payment lots and finite outgoing budgets remain enforced. This is an
explicit tender preference, not a universal optimizer across payment routes or
simultaneous allocation across land, loans and employment.

## Verification

`tests/household_land_funding.rs` exercises:

- Opt-in native purchases, disabled/unfunded controls and the month before a bill.
- Acquired grain visible before ClearArrears while the payable remains outstanding.
- Storage-blocked collection retaining bought grain with money and seller stock
  still available, without buying the same bill again next month.
- Native-first, alternative-first, missing-alternative and partial whole-unit
  payments, with native bills and separate double-entry statements preserved.
- Bartering other stock for accepted coins, then paying rent; no second purchase
  after settlement despite remaining purchasing power and seller inventory.
- Scarce coins shared by rent and an originated loan under proportional collection,
  followed by native fallback, without double-spending; a lower-priority loan
  receives no coins before the senior rent claim.
- Signed voluntary coin support under both tender preferences, with capped support
  and no repeated donation after the bill clears.

The cases compare the reference engine and CubeCL CPU, replay committed ledgers,
and reconcile the financial journal. Purchase cases also check checkpoint
continuation; the native case reverses household and market participant tables.

The selected regressions passed **105 integration tests across 12 suites**,
including these six new tests, plus **four claim-reader unit tests**. Strict
all-target Clippy, formatting and the repository artifact check passed. The full
crate suite was not run.

From this directory: `cargo +1.92.0 test --locked --test household_land_funding`.
Generated validation output stays in ignored `output/economics/land-funding-*.log`.

## Remaining work

These are supplied land agreements and explicit charter settings. Autonomous land
acquisition, household forward admission/funding, negotiated hiring and sustained
employer viability remain separate work. Funding preference is static and scoped
to household-owned bills; it is not an agent-wide currency optimizer or new member
land-assistance policy. Person self-directed policy changes remain deferred.
