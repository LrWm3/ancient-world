# Explicit service allocation pilot

The monthly scheduler now collects research and cultural work forecasts at one
site-service boundary, allocates their shared remaining capacity, then reserves
specific participants. Execution, payments and work settlement remain in their
existing phases. Care and committed cargo crews still precede this window;
workshops and additional vessel staffing still follow it.

This is a two-claim pilot, not a general allocation framework or a claim that
service priorities are balanced.

## Policy and state

`History.service_allocation.policy` is editable through the Rust API:

```rust
history.service_allocation.policy =
    ancient_world::service_allocation::Policy::Weighted {
        research: 1.0,
        culture: 1.0,
    };
```

The default is `ResearchFirst`, making the previous precedence explicit.
Archives missing the field get that default. Weighted policy is opt-in pending
broader scarcity evaluation. Both policy and latest boundary receipts persist
with history; each receipt also captures the policy used at that boundary.

- ResearchFirst gives research first access and offers its unused capacity to
  culture after individual matching.
- Weighted distributes capacity according to positive weights, caps shares at
  requested demand, and redistributes excess entitlement to the other claimant.
- Invalid capacity, demand and weights are rejected.
- The existing shared service ceiling and individual availability remain hard
  limits. Allocation creates neither time, goods nor money.

Forecasts are retained as the plans passed to reservation, rather than being
recomputed after another system spends capacity. Request amounts are feasible
forecasts, already limited by existing planning assumptions and service capacity;
they are not estimates of unlimited latent demand.

The service-work report includes site/month, policy, opening capacity, requests,
allocated caps and actual reservations. Execution receipts continue to report
used and released work separately. Under strict priority the final research cap
is reduced to its matched reservation before unused capacity is offered to
culture. These numbers are not a complete classification of every denial reason.

## Scope and limits

This separates policy from scheduler position for these two claims only.
Individual matching still runs research first. Equal site shares do not ensure
equal access to a scarce teacher. Weighted entitlements that cannot find eligible
participants remain available to later workshops/crews; this pilot does not run
another matching round between research and culture.

Research actions can also have minimum useful work requirements. Dividing labor
more evenly can therefore reduce completed work. The policy does not promise
equal outcomes or improved long-term welfare.

Do not use this allocator to split materials, treasury money, crews or all
settlement labor without defining their eligibility, indivisible requirements
and commitment rules. No late released work is backdated into production.

## Verification

Run the analytical checks:

```sh
cargo test --lib service_allocation
```

Run the hardware-backed matched-boundary fixture:

```sh
cargo test --lib matched_learning_claims_change_shares_without_spending_materials -- --ignored --nocapture
```

The fixture uses seeds 17, 81 and 256 with declared sample/material stocks and a
scarce 0.25 worker-month service allowance. It compares policies on the same
opening state, checks an inactive-culture negative control, unavailable-person
constraints, serialization continuation, and the former sequential priority
path. It checks goods and treasury balances before/after reservation. It does
not execute harvests, prove long-run balance, or compare full GPU checkpoint
continuation.

## Pilot results (2026-09-11)

- Regular library checks: 98 passed, 89 hardware-dependent tests ignored (the
  new hardware pilot was selected separately).
- Allocation suite including hardware fixture: four tests passed.
- Seeds 17, 81 and 256 each produced approximately research/culture reservations
  of 0.25/0.00 under priority and 0.15/0.10 under equal weights. Cultural demand
  was 0.10, so its unused nominal half-share went back to research.
- Reservation preserved exact goods and treasury inventories. The
  inactive-culture control allocated no cultural work. Fully committed people
  received no additional work despite positive site entitlements.
- Serialization of the opening history preserved allocation results. Explicit
  priority matched the former sequential path's reservation amounts.
- Initial fixture failure: a fresh town had no cultural request. The corrected
  controlled fixture establishes a teacher with knowledge and an eligible
  learner; it does not assume a random founding already supplies competition.
- All-target Clippy with warnings denied passed.

Recommendation: retain the pattern, with weighted sharing still opt-in. It makes
a controlled choice observable without moving execution stages or weakening
resource ceilings. Before extending it to workshops or public services, expose
their read-only demands, joint funding/worker constraints and minimum viable
grants. This run does not justify any particular long-term shares.

The separate full-scheduler check also passed:

```sh
cargo test --test history_environment frozen_schedule_batch_and_checkpoint_equivalence -- --ignored --nocapture
```

Seeds 17 and 256 used weighted policy; seed 81 retained research-first. For each,
24 batched months matched 24 single steps, followed by 12 months matching a
checkpoint-resumed 5+7 months. The existing check compares complete history and
terrain/ecology state. This establishes scheduling/continuation consistency;
the controlled fixture above, rather than these ordinary worlds, establishes
that competing requests actually receive different shares.
