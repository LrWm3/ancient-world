# Crop resource competition

Managed crops now calculate their monthly demands before withdrawing water or
nutrients. Previously each crop consumed supplies in catalog order. Under shortage,
later crops could lose their entire allocation to earlier entries, even if their
planted area and habitat supported growth.

This change applies to ordinary diversified farming and the optional seasonal
experiment. It does not enable seasonal farming by default or change crop catalog
coefficients, food demand, land area or founding inventories.

## Monthly allocation

For crop j, let G[j] be potential growth after planted area, habitat, seed or stand
availability, seasonal canopy and industrial stock limits. Chemistry comes from
the crop good; water requirement comes from the crop catalog.

```
N_demand = sum(G[j] * nitrogen_per_kg[j])
P_demand = sum(G[j] * phosphorus_per_kg[j])
W_demand = sum(G[j] * water_m3_per_kg[j])
f_resource = min(1, available_resource / max(demand_resource, 0.000001))
             [1 when demand is zero]
f_growth = min(f_N, f_P, f_W)
actual_growth[j] = G[j] * f_growth
```

Each crop receives the same fraction of its potential growth. Larger planted
areas and more suitable crops consequently receive larger absolute allocations.
Seasonal reproductive damage uses `1 - f_W`, so preceding catalog entries no
longer manufacture an apparent local drought. Harvest, finite seed retention,
frost losses, residue, herds and fisheries retain their existing behavior.

GPU invocations still own one settlement. Two local loops and six temporary demand
values replace sequential allocation; no extra dispatch or host readback is needed.
Withdrawals retain a final available-stock cap for float32 roundoff. Carbon growth
is still an atmospheric import; nitrogen, phosphorus and water leave existing
stocks. Damage returns the crop's embodied C/N/P to detritus.

This is proportional game allocation, not a model of roots, irrigation rights or
optimal farming. One limiting resource scales every crop, potentially leaving
other resources unused. It does not model species-specific competitive advantages
beyond their existing potential and requirements. Summation order can still change
float32 roundoff; equivalence is tolerance-based rather than bitwise.

## Verification

`scarce_crop_resources_are_not_awarded_in_catalog_order` runs paired GPU worlds
with declared phosphorus depletion and reverses both catalog crop order and the
corresponding physical crop stocks. Standing biomass, seed and cumulative harvest
must match by crop identity within 0.0001 relative error, with a unit denominator
floor. The fixture also requires positive growth, valid stocks and normalized
economy residuals below 0.001.

```
mise exec rust@1.89.0 -- cargo test --test economy scarce_crop_resources -- --ignored --nocapture
mise exec rust@1.89.0 -- cargo test --test economy -- --include-ignored --test-threads=1
```

The broader economy suite covers phosphorus exclusion, residue/manure return,
seasonal save/resume and monthly stepping, and the three-seed crop/price factorial
experiment. Accounting success does not imply that the optional seasonal package
has viable yields; see [the earlier experiment](crop-and-price-experiments.md).

## Ordinary-world comparison

The existing material-history fixture was repeated for seeds 17, 81 and 256,
terrain 32/ecology 16, five founding civilizations, one geological epoch and
30 history years (31 for seed 17's continuation check). Reported endpoints were
unchanged from the previous material-tuning run:

| Seed | Population | Usable institutional capacity | Extensions |
|---|---:|---:|---:|
| 17 | 754 | 121.7 | 13 |
| 81 | 711 | 94.7 | 4 |
| 256 | 763 | 122.6 | 6 |

Normalized economy residuals stayed below 0.000006 in absolute value. This fixture
reports population, construction and material output rather than crop-specific
yields; unchanged endpoints do not establish identical agricultural trajectories.
The controlled phosphorus intervention provides the direct evidence for changed
resource allocation. These checks do not establish broader yield calibration.

Verification completed on the Quadro RTX 5000 Max-Q/Vulkan: 23 economy tests
passed with ignored hardware tests explicitly enabled, including the 12-case
crop/price comparison; 47 ordinary library tests passed (50 hardware tests ignored
in that command). After the final zero-demand diagnostic guard, the reversed-order
and seasonal continuation fixtures passed again. Clippy with warnings denied,
formatting and the source-only artifact checks passed. No raw experiment output
or binary archives were committed.

The later [crop and price revisit](crop-price-revisit.md) corrects seasonal annual-budget
scaling and quote drift, with new multi-seed and long-run results. Earlier failed
calibration results above describe the implementation at that time.
