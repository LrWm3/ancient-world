# Shared council payroll under fiscal pressure

Administrative payroll previously consumed each council's treasury while iterating
through towns. In a shortage, early settlement indices received full wages and later
towns received the remainder or nothing. This created apparent political favoritism
without an explicit political rule.

## Snapshot and proportional allocation

At the existing monthly governance boundary, collect all active towns' wage
requirements before paying any town. The existing formula is unchanged:

```
required = population * rate * (1 - 0.5 * autonomy)
rate = 0.01 for a homeland administration, 0.04 for an occupied administration
```

For each controlling council:

```
funding_fraction = min(1, available_treasury / sum(required))
payment[town] = required[town] * funding_fraction
```

Abandoned towns make no demand. Councils remain separate budgets; one wealthy
council does not subsidize another automatically. Occupied administrations still
cost more and autonomous towns cost less. Full funding and zero funding retain
their former behavior.

The allocator uses the treasury available at this point in the existing monthly
schedule. It does not forecast future taxes, borrow, issue currency or reserve a
second stock of money. Each payment debits the controlling council and credits
existing town cash. Conversion to f32 payments rounds downward; a final bound
against remaining treasury prevents overdrawing. Tiny rounding remainders stay
in the treasury. No new persistence fields, GPU buffers or user settings are added.

Proportional sharing is a neutral game-model rule, not an empirical assertion
about historical governments. Future favoritism or emergency priorities should
be explicit policy inputs rather than consequences of settlement index. Floating
point arithmetic is not a promise of bitwise invariance under every arbitrary
reordering and scale of demands.

## Actual consequences

The resulting paid/required ratio drives the existing unpaid-month count, loyalty,
unrest and occupation-crisis system. This means a fiscal shortage spreads across
the council's obligations instead of concentrating on whichever town was processed
last. It does not guarantee greater stability: a previously protected town can
now experience a real shortfall.

A controlled two-town example has wage requirements of 1 and 4 money and a shared
2.5-money treasury. The former sequential rule would pay 1 and 1.5, yielding
funding ratios of 100% and 37.5%. The new allocation pays 0.5 and 2, yielding 50%
for both. Neither changes total spending.

## Verification

The analytical fixture checks the exact example, reversed demand order, separate
councils, zero demand, full funding, no funding and small non-binary budgets. It
checks that allocated payments remain within the treasury and that funding ratios
match within f32 tolerance.

The GPU-backed history fixture gives one council a homeland and an occupied town.
It checks payments of 0.5 and 2, both unpaid counters rising, total cash conservation,
and lower loyalty/higher unrest than a fully funded comparison. Serialized
continuation produces identical subsequent governance state. These are declared
fiscal and population fixtures, not naturally generated demographic histories.

```sh
mise exec rust@1.89.0 -- cargo test --lib payroll_tests -- --include-ignored --nocapture
mise exec rust@1.89.0 -- cargo test --test governance --test society -- --include-ignored --test-threads=1
```

Succession vacancies, explicit office jurisdictions, public debt and discretionary
regional subsidy policies remain separate future work. This increment fixes a
specific administrative allocation bias rather than introducing those systems.

### Recorded validation

The ordinary all-target suite passed 51 tests (118 hardware/long-running tests
remain ignored by that command). Targeted runs additionally passed six GPU-backed
tests on the Quadro RTX 5000 Max-Q using Vulkan: the new fiscal fixture, three
governance regressions, and two society regressions. They cover occupation
secession/autonomy, treaty and inventory continuation, trade trust, and society
conservation. Formatting and Clippy with warnings denied passed.

[Artifact retention policy](evidence/README.md) preserve
the reproduction commands and source fingerprints. These controlled comparisons
verify the allocation rule and its immediate political effects; they are not a
multi-seed calibration of long-term state survival.
