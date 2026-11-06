# Starting a private replay from main-world liquid assets

The [claim inventory and admission check](HOST-CLAIM-INVENTORY.md) now rejects
imports that would omit live debts/receivables or default history belonging to the
selected accounts. Successful receipts also retain source household-share evidence.

The optional `ancient-world-host` feature now provides
`host::ancient_world::stocks::StockBindings::start`. It creates a new experimental
economy using a completed main-engine settlement's physical inventory, town cash
and local household wallets. It does not retain the skilled-workshop fixture's
starting stocks or money, and cannot top up an already running simulation.

This is a partial liquid portfolio import for counterfactual experiments. It is
not a full settlement migration or a live economics replacement. The source
history remains read-only.

## Boundary and ownership

Supply a completed history at month M and the first labor replay history at M+1.
Closing stock from M becomes opening stock for the private economy at M+1, local
month one. The importer rejects same-month or skipped-month labor frames, source
identity changes and a catalog change between these two frames. When the completed
history contains participation, it must be dated and settled for the selected
site. History has no general Close marker; callers must supply completed exports,
not an arbitrary in-progress History value.

The existing main coordinator settles work in Execute/Respond/Close and increments
the next month in Open. This adapter changes none of those operations. Due arrivals,
cash movements and other Open changes between the exported boundaries are not
reconstructed. Labor frames retain the [historical replay](HOST-LABOR-REPLAY.md)
semantics: they supply counterfactual allowances from settled reservations, not
permission to spend live workers again.

Imported custody has three sources:

- Town goods use stable catalog IDs. `food` reads `Stocks.stock[1]`; ordinary
  goods read their catalog slot in `Economy.goods`. A nonzero virtual food slot
  is rejected. All positive balances in this stock pool must have a mapping.
- Town working cash reads `Economy.finance[0]` once and belongs to the explicitly
  named town custodian. It is not the civilization-wide council treasury.
- Each local household wallet reads `HouseholdAccount.cash` once. Every household
  at the selected site needs its own distinct target agent, even for zero cash.
  Household IDs and account indexing must agree. Vacant estate wallets remain
  household accounts; a deceased head does not erase their cash.

The native household-economy validator also checks the source wallets, including
reconciliation of cash with recorded receipts and spending. A newly balanced
experimental book cannot make an inconsistent source wallet admissible. This is
scoped validation, not a replacement for the main world's full archive validation.

The town custodian and household custodians must be distinct from each other and
from named labor participants. A household's `share` is beneficial ownership of
existing private stock, not another physical inventory: it creates no copied
goods. Household membership and beneficial ownership claims are not reconstructed.
The aggregate town account therefore represents custody for this experiment,
not an assertion that a government owns all private property.

## Quantities and opening books

Every commodity declares its experimental resource and ticks per kg. Currency
declares ticks per source money unit. Scales must be positive powers of two;
conversion floors each balance independently into the engine's nonnegative i32
range. Duplicate source goods, duplicate target resources, currency/commodity
aliases, merged wallets, unknown targets and non-finite or negative quantities
fail before any economy is returned.

Each receipt reports source quantity, units, integer balance, unrepresented
remainder and carrying cost. Remainders are not reassigned, carried into another
month or silently credited later. Large balances require a compatible unit scale
or a future wider-quantity design; overflow fails rather than saturating.

Commodity carrying cost per imported tick is an explicit caller assumption in
experimental currency ticks. It is not a main-world quote or reconstructed income.
The ordinary financial audit opens inventory and cash against opening equity.
There is no income or issuance event for copying the private opening portfolio.
Later production/consumption uses the existing process-cost policy and audit.

`State::empty()` creates the fresh local timeline. The caller supplies an editable
experimental `World` catalog, not a fixture `Simulation` or saved balances. Land,
equipment assets and use rights are rejected because their import is not yet
supported. Embedded controller/review history, discovery history and preserved
governance observations are also rejected, so a prior review cannot skip the first
decision against imported balances. `World` can still contain dated organizational
and contractual terms: supplying fresh terms appropriate to local month one is
the caller's responsibility. Needs, process recipes, organizational rules and
other catalog terms remain explicit scenario assumptions; they are not inferred
from the host.

## API

```rust,ignore
use economics_compute_smoke::{
    compute::Backend,
    host::ancient_world::stocks::StockBindings,
};

let mapping: StockBindings = serde_json::from_slice(&binding_json)?;
let (mut replay, opening_receipt) = mapping.start(
    &completed_history, // M: source inventory and cash
    &first_history,     // M+1: first dated labor allowance
    labor_bindings,
    experimental_world, // catalog only; no fixture balances or physical assets
    Backend::Reference,
)?;
let first_outcome = replay.advance(&first_history)?;
let second_outcome = replay.advance(&second_history)?;
```

`StockBindings`, `GoodBinding` and `WalletBinding` are strict JSON configuration
types. The existing `host_labor_replay` CLI retains its fixture endowments; it does
not automatically opt into this separate bootstrap API. Keep source exports and
generated receipts under ignored `output/`.

## Validation and limits

The focused host tests construct controlled main-engine histories and use actual
reservation/settlement methods. The import case uses 2.75 kg food, 3.375 kg wood,
9.375 town money units and 2.375 household money units. Food uses one tick/kg;
wood and money use four ticks/unit. The expected balances are 2 food, 13 wood,
37 town cash and 9 household cash. Food/wood carrying costs of 3/2 yield a town
opening asset value of 69 currency ticks. The separate council's 500 money units
are not imported.

The supplied collective food requirement consumes one tick/month. Two imported
ticks fund exactly two months, followed by a deficit; later source frames with
999 kg food cannot refill the private economy. Zero and fractional-stock controls
cannot consume a whole tick. Reference/CubeCL CPU agreement, in-memory clone and
JSON history continuation, source immutability, invalid identities/units, duplicate
mappings, omitted stock and stale boundaries are covered. A two-household case
checks distinct wallets and nonduplicated physical stock. A fresh signed
organization is accepted while retained review state is rejected.

Sub-agent review found two source-validation gaps: unreconciled household cash
could otherwise become balanced experimental opening equity, and JSON comparison
could hide a non-finite value in the next catalog. Native wallet and next-catalog
validation now reject both, with regressions. An initial controller equality
guard also rejected fresh signed operating agreements; an agency-owned predicate
now distinguishes static terms from runtime history. The first test runs also
required a missing fixture import and selection of the town's need report rather
than assuming report order; the production assertions were retained.

Verification on 2026-11-05: all 14 focused host tests passed (including the eight
existing labor tests), with exact Reference/CubeCL CPU agreement for the imported
consumption case and the depletion/continuation controls above. Release Clippy
passed for all targets with `ancient-world-host` and warnings denied; release
checking without that feature passed for all targets. Formatting and diff checks
passed. Generated logs remain under ignored `output/economics/`.

Reproduce from `exp/economics`:

```sh
cargo +1.92.0 test --locked --release -j8 --features ancient-world-host \
  --test host_ancient_world
cargo +1.92.0 clippy --locked --release -j8 --features ancient-world-host \
  --all-targets -- -D warnings
cargo +1.92.0 check --locked --release -j8 --no-default-features --all-targets
cargo +1.92.0 fmt --check
```

The importer excludes land, installed equipment, crops/herds, clothing, cargo,
escrow, council/institution/operator treasuries, beneficial-share claims, receivables
and debts. Relevant live credit claims/default history now block bootstrap; other
excluded property and contractual rights remain outside this check. It does not
assert that the imported assets are unencumbered or that
opening equity equals main-world net worth. It requires explicit household wallets;
legacy aggregate economies need a separate conversion. The existing fixed-person,
source-lineage and forecast-visibility limitations still apply. No generated-world
rollout, balance calibration, serialized engine checkpoint or live publication is
established here. Full claims/assets import and exclusive live Reserve integration
remain necessary before replacing the main economy.
