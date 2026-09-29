# Household wind-down completion

Status: implemented, 2026-09-29. This closes the three remaining items in the
household wind-down slice: explicit live crop/right transfer, salvage/write-off,
and general household lending/insolvency. It extends the existing household
Open boundary, shared lending/recovery engine and double-entry audit.

## Scope and completion criteria

| Item | Implemented behavior | Evidence required |
| --- | --- | --- |
| Live property control | Sale explicitly names title-following rights and unfinished processes; buyer accepts future work | Complete consent, no third-party rights silently moved, actual completion or missed-work failure after transfer, preserved WIP and unchanged collateral valuation |
| Equipment salvage | Explicit discard or catalog-defined material recovery, including attached equipment on a free owned site | Single permanent retirement, finite shared storage, carrying-cost transfer or loss, no duplicate wear, later property clearance |
| Household insolvency | General lending and recovery compose with household boundaries | Separate member claims, independent custody, priority payments, authorized deficiency discharge or continued debt, no premature residual distribution |

These criteria are covered. This does not make every experimental driver
composable: mortgage-purchase and paid-employment household drivers still require
separate integration. General loans can already carry supported collateral terms.

## Timing and control

At Open, residual clearance first inspects opening claims and property. Explicit
sales then reserve the existing finite buyer budget, transfer paid property and
accepted control. Retirement then uses the resulting state, reserves storage for
recoverable material in stable household/asset order, and archives accepted assets.
Ordinary monthly regeneration and aging follow. A successful disposal never
retroactively frees work or pays residuals at the earlier clearance boundary.
Residual distribution waits for the next Open; final closure still needs `finish`.

Requested sales, accepted payments, process changes, rejected requests and actual
material outputs remain distinct receipts/effects. Settlement recomputes them and
checks buffer limits before publishing any state. The audit verifies that same
boundary before publishing statements. Checkpoint continuation preserves these
receipts, ownership, outstanding claims and the retirement archive.

## Live rights and crops

`disposal::Sale.control` is optional. Without it, the existing live-right/process
restriction remains. With it, `Control.rights` and `Control.processes` must exactly
name the current rights and active processes attached to the plot. All rights must
follow ownership; the household must currently control both work and output, with
no active dues agreement requiring a different transfer of obligations. The buyer
must have land-access and relevant process permission as well as asset-trade
permission. Funding and the complete attachment set are checked again at execution.

The shared credit attachment primitive changes operator and beneficiary, clears the
previous operator's personal goal, and preserves stage, start date, elapsed work
and remaining commitment. Buying the plot supplies no free labor. A labor-capable
buyer completes the crop; an otherwise identical buyer with zero capacity fails
under the existing process consequences.

Financial WIP moves at carrying cost through TransferExpense/TransferIncome. Crop
cost and expected yield do not increase plot consideration or collateral value.
Effective right ownership, rather than historical holder fields, determines whether
the old household is still blocked from closing. Independent tenancy/lease
novation requires additional parties' consent and is deliberately refused here.

## Salvage and write-off

`retirement::request` retains the exhausted-portable behavior. Explicit
`request_with_mode` adds:

- `Discard`: permanently retire usable or exhausted equipment, recognizing any
  remaining carrying value as DisposalLoss.
- `Recover`: use `Activities.salvage[kind]` to recover specified stock quantities
  and permanently retire the equipment. This describes immediately recoverable
  material; labor-intensive recycling should use process definitions.

Attached equipment can be decommissioned only on a household-owned site without
live rights/processes or financial encumbrances. Default exhausted-portable consent
cannot authorize demolition. Offers, delivery commitments, pledges, recovery
listings, current use and live permissions are rechecked. Storage is shared across
all accepted recoveries; rejected requests retain their equipment and produce no
material. Retrying requires fresh dated consent. Archival provenance reserves the
asset ID permanently, including historical attachment and condition.

Salvage transfers remaining equipment basis into the output stocks. Quantity shares
in resource-ID order allocate cost; this is an explicit simple cost convention,
not fair-value pricing of different materials. Exhausted zero-basis equipment
produces zero-cost salvage; actual later sales recognize actual proceeds through
normal stock accounting. Recovery creates no cash flow. The financial adapter
rejects recovery of its monetary denomination rather than treating it as issuance.
There is no second depreciation charge and no hidden value left in the archive.

## Lending, creditors and insolvency

The general lending composition guard now permits households. Loan issuance and
repayment do not count as pooled earned output. A member creditor retains its
receivable while the household retains the payable; reporting does not net them.
Existing priority servicing, collateral and authorized recovery semantics apply.

Household recovery requires explicit wind-down before the configured case opens.
Custody must be an independent agent, never the household or one of its current or
historical members. Listed property is sold through the existing finite funded
exchange; proceeds enter custody and reach creditors at the next Due boundary.
Closing a case does not erase its unpaid loans. An explicit discharge term can
write off the deficiency; otherwise claims remain and prevent residual payout and
household closure. Residual assets reach the named member only after clearance.

The controlled distressed case starts with two loans of ten coins, a plot carrying
cost six, two grain carrying cost four and no available household coins. Sale for
eight coins pays the senior creditor eight; the member's junior claim receives
zero. Authorized discharge clears the remaining twelve. Without discharge, all
twelve remain outstanding and the household retains its residual grain. The
initial cash loss is a pre-audit scenario condition, not a claimed journaled loss.

## Verification

Tests compare CubeCL CPU and reference state, ledger and separate-agent statements,
including checkpoint continuation. Focused controls cover exact transfer consent,
third-party rights, changed terms at execution, forged receipts, insufficient
storage, duplicate recovery, attached-site claims and unsupported valuation rollback.

Run from `exp/economics`:

```sh
cargo +1.92.0 test --locked --test household_credit --test household_property_package --test household_equipment_disposal --test household_dissolution --test households --test household_accounting --test manufacture_accounting --test recovery --test resale --test accounting --test laws --test agreement_laws --test telemetry --test lending
cargo +1.92.0 clippy --locked --all-targets -- -D warnings
```

Final validation: **252 distinct tests passed across 20 suites**. The command above
covers 213 of those checks; the earlier passing activity, equipment and credit
suites add 39. Strict all-target Clippy, formatting and repository artifact checks
passed. One slow 32-person accounting test remained ignored; this was not the full
crate suite. Generated logs remain under ignored `output/economics/wind-down-*.log`.

The initial wider run found an obsolete assertion that household lending must be
rejected. That test now verifies the intended replacement: wind-down is admitted,
but future loan commitments still block residual distribution. The final affected
suites passed after this correction.

## Separate future extensions

Automatic death/estate triggering, compulsory multi-member organizational
insolvency, third-party tenancy novation, autonomous liquidation pricing,
labor-intensive recycling and universal composition across the experimental
mortgage, employment and market drivers are not part of this completed slice.
These require additional legal terms or decision policies rather than silently
changing the current agreement's meaning.
