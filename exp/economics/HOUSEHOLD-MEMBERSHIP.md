# Adult household membership changes

Implemented extension to the [household governance basics](HOUSEHOLD-BASICS.md).
This is explicit admission and member-requested exit, with supplied decisions.
Market recruitment, automatic decisions to move, children and estate settlement
remain future work.

## Timing and consent

`households::membership::join` admits an unaffiliated adult. The entrant and every
living current member must sign acceptance of the unchanged household terms.
`leave` takes the departing adult's instruction; it requires no governor approval.
These are scenario/API instructions, like existing explicit ballots. They do not
simulate signatures or negotiation.

Both APIs apply atomically at **Open, before that month's work and governance
instructions**. Formation must have occurred in a previous month. This first slice
allows one change per household per opening. It rejects pending production,
midmonth requests, and changes after a ballot or operating-policy instruction has
already used that month's authority. There is no retroactive reassignment of labor
or goods and no scheduler change. Sequential changes in different households can
allow a person to leave one and join another at the same opening; this is not an
atomic multi-household move.

Joining reuses the current household agreement recognition and `FoundHousehold`
permission for all consenting participants. It checks current legal limits against
the unchanged constitution/charter and proposed living roster, retaining an
admission receipt. A separate legal accession action remains an extension. Existing
memberships do not vanish when recognition changes. Exit does not require renewed
founding permission. Changes must also satisfy the accepted founding law's adult
bounds, with at least one living adult and at most four in this adult-only pilot.
The last adult cannot leave through this API: dissolution and asset disposition
need their own agreement/estate settlement.

## History, authority and operations

`Agreement.adults` remains the immutable founding roster. Dated membership changes
supply historical rosters and the current roster. Validation rejects overlapping
memberships across households, including historical periods. Constitution, charter,
founding legal evidence, ballots and previously authorized policy stay intact.

Pooling, common-stock reservations, delegated labor and shared dwelling services
use current living members. The next Open batch records the accepted membership
change alongside governance authority. Settlement reconstructs those receipts and
rejects altered evidence. With settlement observation enabled, logs include
`household_membership` records with household, person, action and month, including
when filtered by the departing person.

Election eligibility is taken from membership at the term opening. Earlier ballots
retain evidence that their voters and candidates were members when submitted;
people absent at the election opening are excluded from that election. An entrant
can vote for later scheduled terms. Joining midterm does not change an already
resolved election. An elected governor's departure leaves a vacancy until a regular
election; previously authorized policy continues. Fixed-founder authority becomes
vacant while the founder is absent. Rotation uses the term-opening roster, retains
the founder as its calendar anchor, skips departed members, and admits new members
to the rotation at the next term boundary. Existing death handling still applies.

As with supplied policy instructions, membership history lives in the World.
Continuation restores **World and State together**. An old batch cannot be replayed
against an arbitrary later configuration as if membership changes never occurred.

## Property, storage and accounts

Membership changes have no financial transfer effects. Personal inventory, assets,
processes, rights, tax obligations, forwards and other debts retain their owners and
counterparties. Common inventory stays with the household; there is no automatic
exit payout or debt forgiveness. Financial statements remain separate. Household
payment support ends for a departed member; their underlying debt still exists.

The half-storage pool uses the updated roster. A departing person recovers their
private capacity, while the household loses that contribution. An exit is rejected
if either side's existing inventory no longer fits. Goods must first be moved or
disposed of through a separate supported transaction; the membership API does not
destroy or move them. An entrant's existing inventory consumes space, so joining
cannot duplicate capacity.

Fractional income-sharing carry is keyed by **(household, person, resource)**.
A move cannot use the old household's remainder to fund the new household. The old
carry remains dormant, resuming only if that person rejoins the same household.
This is sub-unit collection bookkeeping, not a newly minted stock or enforceable
financial receivable. General fractional-claim settlement is not implemented.

Terminal members retain their existing storage attribution until a future estate
policy disposes of capacity. They already stop operating and receiving new pooled
support. This extension does not silently introduce an estate or capacity-loss rule.

## Verification

Tests exercise consent, eligibility and adult bounds; current law and historical
admission; atomic storage rejection; unchanged state, property and debts; household
moves and fractional collection; historical elections, entrant voting and rotating
authority; actual labor pooling; Open receipt replay/tamper rejection; and observer
filtering without changing execution. A six-month audited join/exit case produces
identical CubeCL CPU/reference state, ledger and separate financial statements.
A separate four-month membership scenario checks cloned continuation as well.

Run from `exp/economics`:

```sh
cargo +1.92.0 test --locked --test households --test household_accounting --test agreement_laws --test laws
cargo +1.92.0 test --locked --test telemetry --test storage_currency --test forward --test forward_accounting
cargo +1.92.0 clippy --locked --all-targets -- -D warnings
```

Verification on 2026-09-28: **107 checks passed**: 55 household, six household
accounting, 11 agreement-law, six action-law, seven forward, five forward-accounting,
seven storage/currency and ten telemetry tests. Strict all-target Clippy, formatting,
diff whitespace and repository artifact checks passed. The full crate suite was
not rerun.

The slow 32-person annual accounting test is normally ignored. Its earlier passing
result is recorded in HOUSEHOLD-BASICS.md; it is not a new membership-churn test.
This change does not establish scalable membership-history performance, recruitment
incentives, migration balance, general institutional membership, household hiring
or borrowing, or negotiated exit settlements.
