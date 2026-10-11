# Mixed-organization acceptance scenario

Status: six-pass refinement in progress. This is one opt-in fixture built on the
integrated commitment economy, not a replacement scheduler or a claim of arbitrary
organization support. Run from this directory:

```sh
cargo +1.92.0 run --locked --release --example acceptance
```

The acceptance target is people, a household, a workshop, a town organization, a
state and a venue interacting through finite resources, lawful agreements and
separate accounting. Success requires actual outcomes, not merely named agents.

## Pass 1 — establish the existing chain

`acceptance::Run` reuses the elected state/household commitment fixture and normal
Audit stepping. It reports actual nutrition deficits, completed mint work, forward
deliveries, annual land payments, issuance and reconciled separate agent books.
It rejects evidence requests before a completed month and reconciles the entire
coin supply against issuance. The 14-month reference control has two opening
nutrition deficits, four completed mints, 40 issued coins, two delivered wheat and
one paid annual land due. A final delivered total alone does not prove timeliness;
the underlying commitment tests retain the month-four deadline assertion.

## Boundaries to close or retain explicitly

- Workshop: no distinct operating company yet in this initial pass.
- Town: no governed municipality yet; locality is not governance.
- Marketplace: the existing mint venue supports scoped posted quotes; it is not a
  universal market for all contracts. Do not combine incompatible acquisition drivers.
- Founding consent, constitutional menus, quotes and initial endowments are supplied.
- No automatic company ownership/dividends, municipal formation or dissolution,
  multiple jurisdictions or population/discovery integration is implied.
