# Governed market acceptance — implementation record

This batch extends the recurring 48-month acceptance with organizational choices
about production and inventory trading. Initial identities, consent, recipes,
constitutional options and objectives remain supplied. The existing forward and
spot variants remain controls. This is a bounded implementation, not unrestricted
institution formation or price discovery.

## Iterations

1. **Signed stock authority.** Operating agreements can specify per-market side,
   admissible prices and inventory targets/floors, and a lot ceiling. Installation
   validates market units and compatibility before accepting signatures. Authority
   alone publishes no orders and grants no legal or venue access.

2. **Own quote programs.** `PeerStockQuotes` replaces only the issuing
   organization's independent stock quotes at the following Open. Signed bounds,
   lot units and whole-book compatibility are validated; law and venue eligibility
   remain live checks. Empty quotes explicitly stop discretionary trading.
   Other agents' quotes and the common claim-protection horizon are preserved.
   Five organizational tests cover consent and invalid/cross-agent commands.

3. **Atomic compound admission.** Programs reuse the live activity target
   validator before issuing any commands. Issuance stages a candidate world;
   a later prohibited trade cannot leave an earlier process start scheduled.
   The regression exercises that exact late-law failure and malformed/duplicate
   commands. Library, shared-agency and program regressions pass.

4. **Activation and cancellation boundaries.** Three integration tests verify no
   same-month quote effect, checkpoints on both sides of next-Open activation,
   actual accepted ore purchases, and CPU/reference agreement. Revoking trade law
   prevents execution without rewriting the accepted decision. An economically
   motivated explicit stop removes purchases while preserving other agents' quotes and
   employment obligations. A positive supply control prevents a vacuous no-trade
   test.

5. **Persistent work discovery.** A discovering signed organization derives work
   targets from its own positive reserve objectives and permitted recipes. Matching
   follows declared objective priority, not numeric resource IDs. Other discovery
   adapters retain their prior behavior. Initial discovery rejects supplied
   programs; changing holdings does not change the historical catalog. Focused
   discovery, consent and existing discovery regressions pass.

6. **Bounded market discovery.** Signed market options enumerate complete quote
   portfolios, including stopping all trading or dropping selected markets.
   Prices, floors and targets remain constitutional choices; no counterparty is
   preselected. Cartesian expansion and the overall catalog reject excess size
   rather than silently truncating. Three stock-discovery tests cover static
   catalogs, malformed options, partial stops and joint expansion bounds.

7. **Compound production and procurement.** Discovery evaluates bounded work plus
   quote portfolios as well as either command alone. A zero-ore test demonstrates
   that the combined program improves a valid forecast while neither standalone
   command does. Retaining policy wins equal-loss ties; among equally improving
   new programs, a productive bundle with quotes precedes work alone. This explicit
   tie rule permits posting useful offers before a peer has selected its program.
   Joint catalog size is checked before constructing combinations.

8. **Governed 48-month variant.** `acceptance::governed` starts without workshop
   or town work menus, work orders or ore quotes. Their signed governors discover
   and accept programs at month 1, effective month 2. The checker additionally
   requires discovered production and trading decisions and fails gracefully if
   an operating controller is absent.

   An initial cash-first workshop objective failed: 14 mints and 13 batches per
   organization, ending with wage arrears despite no food deficits. Forecasts
   preserve accepted peer intent but do not invent later autonomous state demand;
   cash-first choice therefore stopped replenishment at a later review. The
   bounded correction specifies metal, then ore working reserves, then cash as
   the workshop's objectives. Town objectives value cash before its output buffer,
   so the production target is not mistaken for a sale floor. This is explicit
   goal calibration, not a solution to counterparty expectations or arbitrary
   inventory valuation.

   The corrected reference and CubeCL CPU runs pass: 46 mints issue 460 coins;
   both organizations complete 47 batches and each pays 48 wage coins; food
   deficits and wage arrears are zero. All nine books reconcile. Each post-startup
   year records 12 batches per organization, 24 ore delivered and workshop/town
   income of 12/13. Final cash is 52/57, matching the supplied-quote spot variant.

9. **Failure controls and compatible mandates.** Five existing supply, access,
   margin and payroll controls now run against governed programs on both CPU and
   reference execution. Missing controllers, revoked town trading permission and
   the cash-first objective also fail readiness. Repaying late wages does not
   erase historical arrears. A three-month ore-production interruption recovers
   to the normal two later operating years without food deficits or wage arrears;
   readiness still records the interrupted year's lost work.

   Installation also checks the union of signed organizations' possible quote
   slots against the shared market schema. Individually valid mandates cannot
   collectively authorize incompatible listings or exceed the 32-quote limit.
   Controls reject conflicting aliases and 33 slots, while accepting a common
   listing and 32 slots. This deliberately conservative check can reject aliases
   that would never be posted simultaneously; it reserves no stock or money.
   The nine tests in the governed-controls, authority-compatibility and
   stock-discovery targets pass on the final implementation.

10. **Continuation, regression and documentation.** The governed scenario now
    uses the recurring acceptance's every-phase CPU reconstruction, uninterrupted
    reference comparison, dated replay and independent catalog reordering checks.
    State, World, ledger and audit observations agree. Reconstruction is in memory;
    replay requires the dated World contexts alongside ledger batches. This does
    not establish durable checkpoint serialization or replay from a bare ledger.
    README, integration status and operating-agreement documentation now distinguish
    discovered programs from the remaining supplied terms.

## Final verification

All 132 tests in the selected final regression targets passed: 18 library tests
and 114 integration tests across acceptance, recurring continuation, governed
controls, agency/program discovery, organizational authority, commitments,
stock/mint orders and telemetry. This was a scoped regression run, not every test
in the crate. A subsequent five-test organizational run also passed after a
Clippy-only test cleanup; those repeated tests are not counted twice.

Strict release Clippy passed for all targets, formatting passed, and the repository
artifact check passed. Independent review found no blocking implementation issues.
The actual CubeCL CPU acceptance CLI also passed the governed baseline.

## What this establishes and what remains

The workshop and town can select recurring production and market procurement or
sales through the shared agency controller, without supplied work menus or opening
ore quotes. Their choices preserve execution boundaries and existing obligations.
The calibrated economy sustains three complete post-startup years, and its tests
detect economic failure rather than merely checking conservation.

Prices, signed options, working-stock goals, employment, identities, initial assets
and consent remain supplied. The acceptance fixture permits one calibrated ore
price; it does not demonstrate negotiated pricing or ZIP. Static bounded search
is not unrestricted portfolio planning. The cash-first failure remains a concrete
counterparty-expectation gap, and the reserve priorities are a declared calibration
rather than an endogenous solution. Permanent supply/access failures remain
failures; the temporary interruption demonstrates recovery only after supply is
restored. This batch adds neither unrestricted organization formation nor
self-directed personal policy changes.

Run from `exp/economics`:

```sh
cargo +1.92.0 run --locked --release --example acceptance -- --governed
cargo +1.92.0 run --locked --release --example acceptance -- --governed --continuation-matrix
```
