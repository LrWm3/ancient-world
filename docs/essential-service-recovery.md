# Urgent shelter and essential-service recovery

The existing opt-in `production.waterworks_repair_priority` policy now splits urgent
shelter from optional housing headroom. A small housing deficit previously prevented
priority water repairs for the whole month, while housing could spend the entire
asset allowance building beyond current need. Towns can now resolve that deficit and
repair inherited water service within the same construction budget.

## GPU construction order

When waterworks, housing and repair priority are enabled:

1. Construct shelter up to current population need, bounded by the housing target.
2. If shelter is now adequate, apply the existing water-repair gate: usable water,
   damaged previously installed capacity, target capacity, materials and labor.
3. Spend remaining work on optional housing headroom.
4. Continue the existing water expansion and storage construction stages.

Urgent constructed places are bounded by:

```text
urgent target = min(housing target, max(population - inherited shelter, 0))
new places = min(max(urgent target - constructed places, 0),
                 remaining asset work / 0.2, timber / 2, bricks / 3)
```

Housing uses 2 kg timber, 3 kg bricks and 0.2 worker-months per place. Waterworks
retain their existing 2 kg timber, 4 kg bricks and 0.2 worker-months per served
resident. Both stages draw from the same 10% craft-work allowance; urgent shelter
does not add another allocation. Monthly housing construction now accumulates both
housing stages, preserving inspector and event totals. Water service from repaired
assets still begins on the next monthly operation pass.

Unresolved severe crowding continues to block priority water repairs. Missing
materials cannot be bypassed. Previously installed capacity and target limits still
bound repair; it does not create a free service expansion or a water source. With
repair priority disabled, the previous construction order remains unchanged. No
catalog defaults, archive fields or GPU buffer layouts change.

This is a game allocation rule, not a calibrated model of disaster response. It also
applies to ordinary crowding and wear. No new disaster types or automatic policy
activation are added.

## Verification

The existing GPU waterworks recovery fixture now includes two additional matched
branches with a 0.01-person shelter deficit: repair priority enabled and disabled.
The enabled branch constructs urgent shelter, repairs more water capacity and builds
less optional housing. Tests check timber/brick proportions, the combined asset work
bound and material/C/N/P residuals. Severe crowding, absent bricks and no historical
damage remain separate negative controls.

The marginally crowded branch is saved, loaded and continued in different batch
sizes, requiring exact equality. Existing waterworks health/service, housing and
storage suites cover related physical budgets and persistence. This is controlled
mechanism verification, not an ensemble estimate of recovery rates.

```sh
mise exec rust@1.89.0 -- cargo test --test waterworks -- --include-ignored
mise exec rust@1.89.0 -- cargo test --test housing -- --ignored
mise exec rust@1.89.0 -- cargo test --test storage -- --ignored
mise exec rust@1.89.0 -- cargo test --all-targets
mise exec rust@1.89.0 -- cargo clippy --all-targets -- -D warnings
```

See [Artifact retention policy](evidence/README.md) and the
[existing policy guide](waterworks-recovery.md) for activation and model limits.
