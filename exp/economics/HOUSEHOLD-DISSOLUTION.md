# Solvent household dissolution

Implemented extension to [adult membership](HOUSEHOLD-MEMBERSHIP.md). A last adult
can now wind down a household under its founding terms, settle residual stock, and
release the final membership. The household's identity, accounts and history remain.
This is a solvent wind-down path with explicit sales of unencumbered catalog
assets, portable equipment and residual stock distribution. General inheritance and bankruptcy remain
separate extensions.

## Founding rules and authority

Dissolution is opt-in: `Constitution.allow_dissolution` defaults to false. A template
can enable it at founding. `Charter.residual_recipient` is a static parameter:
`None` selects the last living member; `Some(agent)` names a different recipient,
such as the state. The recipient must exist and cannot be the household itself.
This allocation is a contractual surplus entitlement, not ownership of the household.

State household founding rules can refuse dissolution-enabled templates through
`Rules.allow_dissolution`. The accepted law snapshot continues to govern the founding
terms after later recognition changes, following existing household admission rules.
There is no amendment API or new legal discretion at the payout boundary.

The last living member supplies the wind-down instruction. This is an explicit
scenario/API decision, like supplied ballots; there is no autonomous decision to
close. It does not require a current governor, so a vacant office cannot veto this
constitutionally reserved right. Multi-member dissolution requires another policy.

## Lifecycle and monthly boundaries

| State | Entry | Behavior |
| --- | --- | --- |
| Operating | Founding / admission | Existing household pooling, labor and governance |
| Winding down | `dissolution::request` at Open | Stops new pooling, member support, dwelling sharing and delegated labor; retains affiliation and storage while obligations settle |
| Dissolved | `dissolution::finish` at a later Open | Releases final affiliation/storage, retains historical identity and separate accounts; cannot reopen or accept new property/claims |

Request and finish use the membership boundary: Open before work or governance
instructions, after the founding month, with at most one membership/lifecycle change
per household per month. They preserve the World/State checkpoint model. Dated
`WindDown` and `Dissolve` records cannot rewrite earlier rosters, elections or policy
authority. No new ballots, policy instructions or members are accepted during
wind-down. Existing policy history remains available, but the household no longer
executes collective work. The person continues their own independent activities
and remains responsible for their personal obligations.

The final affiliation and contributed storage remain reserved during wind-down.
The adult cannot join a second household until closure releases that affiliation.
The operational member iterator is empty during wind-down; this does not mean that
storage has become ownerless or available to two households at once.

At each Open, before ordinary work, the resolver checks clearance. If eligible, it
emits an all-or-nothing transfer of the household's positive stock balances to the
charter recipient. No transfer occurs if a blocker remains. The batch carries the
blockers, recipient and actual distributed quantities. The existing gather/commit
and accounting adapter apply the transfer; it is not an unjournaled API mutation.
Later obligations settled during the month become visible to the next Open's
clearance check. New receipts during wind-down remain household property and must
also be cleared before closure.

`finish` is a separate instruction at a subsequent Open. It requires zero household
balances, no remaining blockers, the living last member and valid storage after
release. It transfers nothing and cancels no debt. The retained financial reporting
entity stays separate from the former member. Later attempts to give a dissolved
household new property or claims fail validation rather than silently reviving it.

## Clearance and limitations

Clearance includes receivables as well as liabilities. It blocks on outstanding
loan balances, future configured advances, unpaid or still-running land agreements,
unfulfilled forwards, guarantees, unfinished recovery, employment obligations,
owned assets/equipment, active rights/processes, non-stock balances and configured
exchange roles without a cancellation adapter. Having no bill due *today* is not
sufficient grounds for releasing all property.

The recipient must be available and cannot itself be a winding-down or dissolved
household. Stock must fit both the present boundary and the destination after the
source household's contributed storage is released. If any resource cannot fit,
coins and other goods are retained too; the next Open can retry after circumstances
change. There is no implicit liquidation, forced sale, storage creation, fractional
claim buyout or write-off. Fractional sharing carry remains historical bookkeeping.

**General lending and recovery are still not composed with the household driver.**
Their existing validation rejection remains. Defensive clearance recognizes those
positions, but this change does not claim to service household loans or perform
household insolvency. The working debt-composition test uses supported land dues.
Unencumbered catalog assets and usable portable equipment have the explicit
disposal path below. Attached/exhausted equipment, live rights and attached
processes still block closure. Configured standing roles may require retirement rather than
being inferred finished from a zero balance.

If the last member or recipient dies before payout, wind-down defers. Automatic
last-member death estates, guardian/executor appointment, contested distributions,
insolvent dissolution, multi-member partition and general organizational dissolution
remain extensions. Existing households retain their default behavior unless their
founding template explicitly enables this path.

## Explicit physical-asset disposal

`households::disposal::accept` records a dated, mutually accepted sale while the
household is winding down. The last living member supplies the household's
instruction; buyer consent and price are supplied scenario terms, like the existing
accepted lending inputs. There is no autonomous solicitation, auction, negotiation
or forced sale. This adapter does not give a governor authority to amend the charter.

The sale names an asset, buyer, payment resource/quantity and current month. Admission
requires Open, the correct last member, household ownership and permission for both
parties to perform `AssetTrade`. Unknown parties/assets, nonpositive payments,
perishable payment resources and duplicate same-asset/month instructions fail
atomically. The accepted terms remain in the household agreement's dated sale history.

At Open, clearance observes opening property **before** disposals. The sale resolver
then rechecks ownership, parties, rights, commitments and current law. Pledged assets,
assets reserved for configured lending/recovery, live use rights and active attached
processes are excluded. It never strips a tenant's right or transfers unfinished crop
work implicitly. Winding or closed households cannot buy. An unavailable last member
or buyer prevents execution.

Funded sales use the existing asset-exchange payment primitive and authoritative
ownership/value registry. A single opening spending allowance covers every accepted
sale; stable `(household ID, asset ID)` priority resolves insufficient shared buyer
funding, independent of instruction insertion order. This is an explicit bounded
priority, not a fairness claim or price-discovery policy. Incoming proceeds do not
increase the same boundary's allowance. Payment must fit storage and ownership moves
only with the complete payment. There is no need to activate the lending scheduler
to hold property; debt/recovery state still requires its existing supported driver.

Rejection retains the asset and money and emits a reason. A dated failed sale expires;
a later attempt needs fresh accepted terms. Successful proceeds remain in household
custody until the **next Open** clearance check, after this month's ordinary supported
obligations can settle. Final membership release still requires a later explicit
`finish`. Sales neither discharge creditors nor bypass any dissolution blocker.

Receipts and payment effects are separate from pooled gifts, replay-checked and
included in the effect-buffer limit. The read-only settlement observer emits
`household_asset_disposal`, including buyer, asset, proposed price, settled status
and rejection reason. A rejected proposed price is not revenue or a market trade.

The financial adapter currently requires payment in its reporting denomination.
It records buyer cost, seller derecognition, actual disposal gain/loss and investing
cash flows. Residual transfers remain TransferExpense/TransferIncome. An unsupported
payment valuation rejects the combined simulation/audit step atomically.

### Portable equipment

The same `disposal::Sale` now accepts a durable equipment ID. Globally unique IDs
select its existing equipment registry; catalog property keeps its existing
ownership registry. Equipment transfers update the owner only. Kind, attachment,
remaining uses and last-use month are retained; no replacement tool or useful life
is created. Receipts include the complete opening equipment condition, verified
again during replay, and the settlement observer exposes kind and remaining uses.

Admission and execution share the ordinary equipment-sale availability rule:
portable, positive remaining uses and no use already recorded this month. Unfilled
posted offers and existing tool-delivery/output-share agreements block disposal;
this adapter does not cancel or novate them. Those contracts remain conservative
blockers even if their nominal purchase appears paid. Attached equipment cannot
be detached for sale, and its plot cannot be sold independently through this path.
A combined plot/dwelling/crop transfer needs explicit terms in a later adapter.

Disposal stays at Open, **before** ordinary equipment aging. The buyer's purchase
price becomes its carrying cost before that month's scheduled decay. The seller
records gain/loss against the opening carrying amount; the buyer bears subsequent
depreciation and productive wear. Remaining life and last-use history survive the
transfer. A tool with one remaining use and one scheduled decay can therefore be
bought and fully depreciated in that same Open; supplied sale terms do not promise
that a purchase is economically sensible. Already exhausted equipment cannot be
sold through this usable-equipment adapter.

Equipment and catalog sales share the same opening buyer budget and stable sale
priority. An unfunded or restricted sale leaves payment and ownership unchanged;
ordinary monthly aging still occurs for the owner. Proceeds stay in the household
until the next Open clearance. Equipment purchases and residual transfers use the
same separate-agent double-entry audit, including rejected unsupported valuations.

Remaining: scrap/retirement of exhausted equipment, attached-property packages,
contract novation, autonomous buyers/prices and household insolvency. Retained
zero-use equipment is still property; it is not silently deleted to permit closure.

## Accounting and verification

Residual coins and stock use the existing verified household transfer adapter.
Inventory moves at its carrying cost; the source records TransferExpense and the
recipient TransferIncome. No equity ownership or consolidation is inferred. There
is no separate legacy accounting path.

The integrated case has 2 grain (carrying cost 6) and 5 coins in the household. An
annual land agreement requires 1 grain in month 13. Open 13 retains all assets;
Due pays the grain obligation. After the right expires, Open 14 distributes 1 grain
and 5 coins, with total carrying value 8. At Open 15 the member closes the household.
Its final assets and liabilities are zero; all three agents' separate statements
balance. CPU/reference states, complete ledgers and audits match, including cloned
continuation from after the dues settlement.

Focused tests also cover constitutional/legal admission, historical recognition,
Open timing, storage deferral and retry, named recipients, owned-asset and future
advance blockers, inactive governance/labor, exact receipt replay and tamper
rejection, closure guards and read-only observer output. Settlement logs add
`household_dissolution` with the recipient, blockers and distributed quantities.

Run from `exp/economics`:

```sh
cargo +1.92.0 test --locked --test household_dissolution --test households --test household_accounting --test agreement_laws --test laws --test telemetry --test storage_currency
cargo +1.92.0 clippy --locked --all-targets -- -D warnings
```

Verification on 2026-09-28: **105 focused checks passed** (10 dissolution, 55
household, six household-accounting, 11 agreement-law, six action-law, seven
storage/currency and ten telemetry tests). Strict all-target Clippy, formatting,
diff whitespace and repository artifact checks passed.

The normally ignored 32-person annual accounting test and the full crate suite are
not part of this validation run. These tests establish settlement consistency,
not economic desirability of dissolution or complete coverage of institutional estates.

## Asset-disposal verification (2026-09-28)

The additional case starts with an asset carried at 6, two grain carried at 4 and
five coins. The state buys the asset for either 8 (gain 2) or 3 (loss 3). Open 2
retains the proceeds; Open 3 distributes remaining grain and money; Open 4 permits
closure. Household, person and state statements reconcile separately. CPU/reference
states, complete ledgers and journals match, including continuation from immediately
after the sale.

Controls cover insufficient funds, shared buyer budgets, reversed instruction order,
recipient storage, live-law denial, changed ownership, active rights, future pledges,
authority/date/duplicate admission, forged receipts/effects, buffer exhaustion,
unsupported payment valuation and observer noninterference. The legacy lending-driver
restriction remains tested: asset ownership alone does not authorize loan state.

Validation results for this increment are recorded in INTEGRATION-STATUS.md. The
slow 32-person accounting test and full crate suite remain outside this run.


## Portable-equipment verification (2026-09-28)

A worn household tool starts with six remaining uses, carrying cost 12 and last
use in month 1. At Open 2 it sells for either 9 or 15 coins. The sale preserves
condition; ordinary aging then reduces it to five uses, with depreciation charged
to the buyer's new basis. The seller records loss 3 or gain 3. Open 3 releases
residual stock/money and Open 4 permits closure. CPU/reference states, complete
ledgers and separate-agent audits agree, including continuation from after the sale.

Eight focused equipment-disposal tests cover the integrated case, complete decay
in the sale month, outstanding offers/delivery agreements, live ownership/use
rechecks, attached equipment and plot rejection, competing catalog/equipment
purchases, forged condition/replay and read-only observer evidence.

**164 tests passed** across equipment disposal (8), dissolution (18), households
(55), household accounting (6), equipment (8), equipment accounting (3), manufacture
accounting (7), activities (10), recovery (26), resale (7) and accounting (16).
Strict all-target Clippy, formatting, diff whitespace and the repository artifact
check passed. The slow 32-person accounting test remained ignored and the full
crate suite was not run. Logs are under ignored `output/economics/household-equipment-*.log`.

Run from `exp/economics`:

```sh
cargo +1.92.0 test --locked --test household_equipment_disposal --test household_dissolution --test households --test household_accounting --test equipment --test equipment_accounting --test manufacture_accounting --test activities --test recovery --test resale --test accounting
cargo +1.92.0 clippy --locked --all-targets -- -D warnings
```
