# Solvent household dissolution

Implemented extension to [adult membership](HOUSEHOLD-MEMBERSHIP.md). A last adult
can now wind down a household under its founding terms, settle residual stock, and
release the final membership. The household's identity, accounts and history remain.
This is a solvent stock-distribution path, not a general inheritance or bankruptcy
system.

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
Owned physical assets require an explicit sale/transfer path before this stock-only
closure can complete. Configured standing roles may require retirement rather than
being inferred finished from a zero balance.

If the last member or recipient dies before payout, wind-down defers. Automatic
last-member death estates, guardian/executor appointment, contested distributions,
insolvent dissolution, multi-member partition and general organizational dissolution
remain extensions. Existing households retain their default behavior unless their
founding template explicitly enables this path.

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
