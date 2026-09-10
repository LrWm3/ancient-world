# Inherited waterworks and recovery priority

This increment connects persistent asset damage to the allocation of rebuilding
work. Previously, housing's 10% spare-capacity target could take the whole shared
construction allowance even when residents already had adequate shelter and an
existing water system had lost capacity.

## Mechanism

`production.waterworks_repair_priority` is opt-in, defaults false for old and new
catalogs, and can be changed in the explorer's economy controls. The existing
waterworks feature must also be enabled.

Each settlement remembers its greatest observed installed waterworks capacity,
measured in served-resident places. The GPU updates that observation before
monthly weathering. The record survives abandonment and reoccupation; it is
neither an inventory nor a guarantee of service. Old archives begin observing
current capacity rather than inventing an earlier undamaged system.

After paying finite operating work, the usual 10% construction allowance may
repair waterworks before optional housing expansion when:

- Remaining shelter can house current residents, or housing limits are disabled.
- Domestic water shortfall is below 1%.
- Current waterworks capacity is below both its observed historical peak and the
  current population-based service target.

The repair quantity is bounded by the missing capacity, available timber / 2,
available bricks / 4, and construction worker-months / 0.2. These are the existing
regional game-scale construction coefficients. Repair materials leave stock and
enter installed structures once. Housing, normal waterworks expansion and storage
then use the remaining allowance. Urgent shelter retains priority, and no new
labor pool, grant, water, money or material import is introduced.

Repairs enter service the following month, through existing domestic-water and
sanitation calculations. Repairs cannot exceed the remembered peak under this
priority, although normal construction may expand the system afterward. Declining
population reduces the target, so historical size is not a mandate to rebuild
unused infrastructure. Wear continues even when a site is empty.

## Inspection and persistence

One fixed GPU vec4 stores peak capacity, policy flag, monthly priority repair work,
and cumulative priority repair work. This costs 16 bytes per settlement and uses
the existing monthly dispatch without another readback. Work is descriptive:
it is already included in total waterworks construction work, and must not be
added to that total again.

The inspector, waterworks event details and production summaries expose these
measures. Existing service disruption/recovery events retain their threshold
and causal links, avoiding a new event for every small maintenance job.

## Scope and limits

This is a recovery allocation rule, not a complete reconstruction program. It
uses existing finite material orders, trade and construction. It does not yet
introduce neighboring-town reconstruction grants, return migration to ruins,
workshop inheritance decisions, or a new disaster process. Asset damage already
enters through monthly weathering and waterlogging exposure.

A peak records observed physical capacity, not a historical claim to ownership.
There is no forecast of future water supply or arrivals in the urgent-shelter
comparison. The 1% water gate and repair ordering are game-design assumptions,
not empirically calibrated policies.

## Verification

The GPU fixture declares the initial resources of a half-damaged system and
accounts for its lost material as waste/detritus. Matched branches change only
the repair policy. Additional comparisons test urgent crowding and absent prior
capacity observations. Checks cover finite shared work, installed capacity,
housing headroom, subsequent operating coverage, material/CNP conservation and
exact checkpoint continuation with different advancement batch sizes.

```sh
mise exec rust@1.89.0 -- cargo test --test waterworks -- --include-ignored --test-threads=1
```

A regional multi-seed disaster/reconstruction evaluation remains future work;
this fixture does not establish improved population survival or refugee return.

### Controlled result

At seed 42, terrain resolution 32 and ecology resolution 16, five identically
prepared towns had month-two operating coverage **55.6502% with priority versus
50.7945% without**. End-of-month installed capacity was 71.265 versus 63.992
resident places; cumulative priority repair work was 2.278 worker-months per town.
These towns share the same controlled starting conditions; they are not five
independent calibration samples. Both branches retain domestic demand and the
same material imports, damage ledger and ordinary construction mechanisms.

Urgent crowding and missing bricks each produced zero priority repair work.
Without a prior peak observation, only newly observed wear qualified for priority,
not the declared but unrecorded older loss. Economic residuals remained below the
fixture's absolute 0.001 tolerance. Same-backend continuation matched exactly.

Validation passed 49 ordinary tests plus 22 explicitly executed GPU tests across
economy, housing, storage and waterworks. Clippy with warnings denied, formatting
and whitespace checks passed. Hardware: Quadro RTX 5000 Max-Q, Vulkan. This is
verification of a controlled mechanism, not a population calibration or a timing
benchmark; the machine was shared with other workloads.

[Archived logs and source checksums](evidence/waterworks-recovery/) retain the
results. Run the broader regression suite with:

```sh
mise exec rust@1.89.0 -- cargo test --test waterworks --test economy --test housing --test storage -- --include-ignored --test-threads=1 --nocapture
mise exec rust@1.89.0 -- cargo test --all-targets
mise exec rust@1.89.0 -- cargo clippy --all-targets -- -D warnings
```

## Urgent shelter within the repair month

The policy now [builds current shelter need before testing water-repair eligibility](essential-service-recovery.md).
A small deficit can be resolved in the same monthly allocation; optional housing
headroom follows priority repairs. The older measurements above describe their
recorded revision; severe unresolved crowding still blocks repair.
