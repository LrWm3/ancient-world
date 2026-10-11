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

Validation results and the remaining iterations are recorded here as completed.

2. **Own quote programs.** `PeerStockQuotes` replaces only the issuing
   organization's independent stock quotes at the following Open. Signed bounds,
   lot units and whole-book compatibility are validated; law and venue eligibility
   remain live checks. Empty quotes explicitly stop discretionary trading.
   Existing counterparties and the common claim-protection horizon are preserved.
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
   motivated explicit stop removes purchases while preserving counterparties and
   employment obligations. A positive supply control prevents a vacuous no-trade
   test.
