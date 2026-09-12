# Illness and effective work

The existing disease-burden index affected births, mortality and remembered social
pressure, but did not reduce work supplied by surviving adults. Public health now
also feeds the shared GPU labor budget.

## Rule and timing

With demographic simulation enabled, available worker-months are:

```
adult_population * 0.8 * (1 - 0.5 * clamp(disease_burden, 0, 0.5))
```

Disease burden is an abstract index, not a clinically measured prevalence. The
0.5 coefficient is a bounded game-model assumption. The usual maximum burden
of 0.5 reduces work by 25%; it does not imply that 25% of residents have a named
infection. Legacy values above this bound cannot reduce work further. Without
demographic simulation, the existing `population * 0.5` rule is unchanged.

Production reads opening-month health, before the month's food consumption and
demographic update. New illness therefore reduces the next production month's
capacity. Environmental disruption applies its existing separate multiplier.
Farming, forestry, extraction, crafts and asset maintenance share the reduced
work allowance. Existing fishing callers use this same helper, without changes
to fishing allocations or catch rules. No extra sick population stock is created,
and lost work is not an inventory requiring a material debit.

This closes several existing feedback paths:

- Water service reduces exposure, preserving later productive capacity.
- Hunger and contamination increase illness and reduce later work.
- Recovery and consumed expedition remedies can restore effective work.
- Smaller construction allowances can slow repair and recovery.

These are potential paths, not guarantees that every affected town declines.
Land, demand, water and material limitations can make extra workers unproductive.
Named people, army combat performance and sparse CPU institutional decisions do
not receive a new illness multiplier in this increment. The burden is not an
infectious-disease compartment model, and shipping does not transmit it here.

## Inspection and persistence

The settlement economy inspector shows current illness burden and its implied
work reduction for the next production month, separately from the last measured
labor allocations. It does not describe the current index as last month's input.
No buffers or archive fields were added. Existing saves remain readable, but
resumed worlds with illness now follow this changed economic rule; exact
continuation across executable versions is not promised.

## Controlled verification

The GPU fixture starts matched five-town histories from one saved baseline,
changing only disease burden. Fixed staffing isolates the labor response while retaining actual land claims.
Burdens 0, 0.2, 0.4 and 1.0 must yield 100%, 90%, 80% and 75% of healthy
work. Cultivated area must match the lesser of available land and the resulting
farming work capacity in the first month, before
mortality could explain the production change. It also checks material/nutrient
and population residuals and save/resume versus batched continuation.

```sh
mise exec rust@1.89.0 -- cargo test --test health_labor -- --ignored --nocapture
```

This is verification of a causal connection, not empirical calibration of disease
or a multi-seed study of the resulting famine feedback. Long-run balance needs
separate evaluation before making claims about realistic mortality or recovery.

### Recorded validation

On the Quadro RTX 5000 Max-Q using Vulkan, the new fixture and nine existing
GPU-backed regressions passed (waterworks: two; discoveries/remedies: four;
society: two; rations: one). The ordinary all-target suite passed 51 tests,
with 119 hardware/long-running tests ignored by that command. Formatting and
Clippy with warnings denied passed. The viewer text was compiled but not manually
exercised in an interactive window.

[Artifact retention policy](evidence/README.md) include
commands and the exact source fingerprints used for these checks.

The later [contagion extension](contagious-illness.md) adds SEIR partitions and
traveler exposure. Infectious prevalence feeds this same burden index; the
existing work and demographic paths still apply the consequences once.
