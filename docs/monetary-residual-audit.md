# Monetary residual assertion audit

Review of estate recovery uncovered a verification error: several credit tests
in `src/enterprises.rs` and `tests/markets.rs` read index 3 of
`History::economy_residuals()` when checking money. Index 3 is water; index 4 is
money. Those particular assertions did not establish global monetary conservation.
Their explicit paired account-transfer assertions remain independent evidence.

The affected assertions now use `History::money_residual()`, a named accessor
for the existing global ledger calculation. No balance equation or tolerance was
changed. A deliberately broken fixture adds and removes one undeclared cash unit
from a hundred-unit economy: the money residual becomes -0.01 or +0.01 while the
water residual stays unchanged. This verifies that the assertion observes the
intended stock.

Verification commands:

- `cargo test --test markets`: 19 passed, two hardware tests not selected.
- `cargo test --lib enterprises::tests::credit_ -- --ignored`: two passed.
- `cargo test --lib persisted_credit_commits_cash_debt_and_failed_collection_together -- --ignored`: one passed.
- `cargo test --lib estates_ -- --ignored`: four passed, including the operator and institutional estate fixtures.
- `cargo test --lib export_contracts`: 13 passed.
- `cargo clippy --all-targets -- -D warnings`: passed.

Corrected assertions required no accounting fixes or relaxed tolerances in these
fixtures. This does not retroactively strengthen earlier test reports or prove
every monetary path correct. The crop-scarcity experiment's reported terminal
money residual already used index 4; this correction does not change its results.
A terminal residual is still not a maximum error measured over every month.

Raw outputs remain in ignored `output/`. Commit only this summary and sources.
