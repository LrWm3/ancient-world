# Material objects and expandable institutional facilities

This is a bounded first implementation of material substitution for a game economy. It does not model structural engineering or a full material taxonomy. New bundled worlds enable it; older catalogs without `materials` retain their old goods and building rules. Definitions from `assets/materials.toml` are compiled into the ordinary economic catalog and archived with it.

## Objects and processing

Six material-specific goods occupy previously reserved slots 45–50. Each has a catalog role, a finite input recipe, worker-month cost, industry, service per kg and wear rate:

| Object | Inputs per kg output | Function |
|---|---|---|
| Wooden vessel | 1 kg timber | Storage and food preservation |
| Metal vessel | 1 kg base metal | More storage per kg, lower wear, higher manufacture cost |
| Wooden shovel | 1 kg timber | Soft-material digging |
| Metal axe | 0.4 kg timber + 0.6 kg base metal | Timber extraction |
| Metal pick | 0.3 kg timber + 0.7 kg base metal | Ore extraction |
| Roof tiles | 1 kg bricks | Masonry tiles cut from existing fired material |

Existing pottery is the ceramic vessel alternative. Towns compare vessel price per service and a simple ten-year wear estimate; existing vessels of every type count toward the same need. Vessels affect the existing granary preservation rule and add a bounded dry-storage benefit (at most 20% of existing yard/warehouse capacity). They cannot create unlimited storage by stacking themselves.

All six remain ordinary goods through production, inventories and shipments. Composite goods carry the mass-weighted C/N/P of their inputs. Worn wooden components return organic matter; the recoverable base-metal fraction returns to scrap. No fuel or feedstock is free. These recipes are deliberately simple assembly/cutting recipes, with upstream smelting and firing remaining in existing industries.

Compatibility is expressed as explicit supported manufacturing methods. The validator rejects ceramic cutting/impact heads and metal tools without a metal working head, invalid quantities, references and table bounds. It is not an arbitrary property solver: this version uses generic timber, existing ceramics and the generic base-metal pool. Separate wood species, ceramic grades and copper/bronze variants of these composite objects remain future content. Existing alloy tools still contribute their older general equipment capability.

## Tool-sensitive extraction

The GPU evaluates cutting, breaking and digging independently. General tools provide limited shared capability; specialized equipment serves only its task. Effective equipment is bounded relative to assigned workers. The game extraction rule is:

`kg / worker-month = base_rate / (1 + difficulty / (0.2 + 2 × capability))`

Base rates remain 20 for timber and 5 for ore/clay. Capability is capped at 2. Ore difficulty combines host-rock catalog hardness (scaled to 0.1–1) and, when the shared-resource registry is enabled, the fraction already extracted from the canonical source. Older unregistered local reserves have no inferred depletion history. Clay uses a low digging difficulty. Timber uses a woodland-stock-density proxy; it does not distinguish trunk sizes or tree species yet.

Sources remain finite and shared; equipment changes extraction rate, never the source inventory. These are progressively slower accessible prospects, not deep mine shafts, pumps or a hard technology unlock. Production forecasts use the same task-rate function. Unused mining labor may serve the other material that month. Equipment wears through existing material accounting.

## Institutional facilities

The previous fixed 2,000 kg hall is no longer the default for new institutions. A facility contains up to 32 rooms. Each room has structural and roofing components, material quantities, condition, useful capacity and unfinished labor.

| Structural method | Material per capacity unit | Work per capacity unit |
|---|---|---|
| Timber enclosure | 10 kg timber | 0.06 worker-months |
| Masonry enclosure | 50 kg bricks | 0.12 worker-months |
| Metal frame/enclosure | 8 kg base metal | 0.20 worker-months |

Roofs independently use timber (4 kg), tiles (8 kg), or base metal (2 kg), with additional catalog labor and wear. A capacity unit is an abstract service allowance, not a floor-area or building-code claim. Roof loads, foundations, spans and climate-specific architecture are not solved.

Target service capacity follows member count and institution kind: religious 1.5, merchant 2, craft 1.25, scholarly 1 unit per member, bounded to 2–64. Each affordable room supplies 2–16 units. The planner compares useful capacity, local prices, maintenance and labor. It keeps half of each available material stock outside the project.

Founding material gifts come from existing community goods, capped in value by 15% of site cash; the previous 25-money organizational endowment remains a separate cash transfer. Initial assembly uses the existing 0.2 worker-month founding allowance. Subsequent construction and repairs share at most 0.1 worker-month per quarter, alongside 0.025 for administration, subject to staffing and the settlement's finite cultural allocation.

An established institution may commission another room when it has unmet capacity demand, finished construction, a maintained facility and sufficient treasury. It reserves 10 money for administration and four quarters of component upkeep, then may invest 75% of the remainder. Expansion buys actual stocked materials from its town, with exact matched cash transfers. Funded prospective extensions and the next quarter of bounded repairs feed ordinary material production/import targets; expected deliveries are not usable construction stock. Procurement and operating capacity both count living members currently at the site, excluding absent representatives.

Existing completed rooms continue serving while an extension is unfinished. Each component wears and repairs independently; replacement purchases remove equal worn mass into the waste/C/N/P ledgers. Installed components cannot change material for free. Destruction, loss and physical ownership still use the foundation artifact. Room component records describe that artifact's inventory; they are not another inventory in the ledger.

The explorer shows usable/planned capacity, room methods, component mass and condition, remaining work and expansion investment. Completion and expansion create linked historical events. Readiness remains an abstract organizational score, with actual available space limiting quarterly operating support.

Shared/rented rooms, dedicated archives or machine-equipped workshops within facilities, remodeling/demolition plans, and explicit influence-funded subscriptions are not implemented in this increment. Other town buildings still use their previous construction rules. Old two-kg and 2,000-kg foundations retain their actual inventories and repair behavior; they are not freely converted into new facilities.

## Verification and small seed runs

Controlled tests cover component suitability and C/N/P, affordable size differences, wood/ceramic/metal substitution, no-work construction, funded expansion while existing rooms stay usable, repair waste, cash transfers and serialized continuation. A GPU fixture measures actual output: picks improve ore but not timber, axes improve timber but not ore, depleted ore is harder with the same pick, and the hand-tool analytical rate agrees. Extracted mass equals the decrease in source inventory.

At terrain resolution 32, ecology resolution 16, one geological epoch and five civilizations with social history:

| Seed | History years | Population | Facilities | Usable capacity | Wooden vessels made kg | Shovels kg | Axes kg | Picks kg |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| 17 | 11 | 673 | 15 | 82.0 | 545.1 | 56.8 | 31.0 | 37.5 |
| 81 | 10 | 652 | 13 | 84.3 | 444.4 | 45.5 | 11.2 | 12.7 |
| 256 | 10 | 686 | 15 | 123.0 | 546.4 | 54.2 | 12.1 | 14.6 |

No metal vessels, roof tiles or expansion events occurred in these ordinary runs. Those mechanisms need broader calibration; the tests do not establish broad adoption. All reported economy residuals were below 0.000003 in magnitude. Seed 17 additionally saved at year 10, then compared a 12-month batch with twelve one-month steps from the saved world: full historical state matched.

The current smoke fixture extends these runs to 30 years (31 for seed 17); see the tuning report below. Reproduce using `mise exec rust@1.89.0 -- cargo test --lib material_history_seed_smoke -- --ignored --nocapture` and `cargo test --lib tools_change_actual_extraction -- --ignored`. The ordinary library suite and the hardware-backed institutional and minority-founding fixtures provide the smaller checks. Generated world files stay temporary; only summaries belong in git.

Final checks: 39 ordinary library tests passed; the extraction fixture, three institutional-capacity fixtures, minority-founding fixture and three-seed history/continuation fixture passed with hardware enabled. Clippy, formatting and source-artifact checks passed.

## First tuning pass: longer matched histories

The same seeds, grids, one epoch and five founding groups were advanced for 360
months; seed 17 then ran another 12 months for checkpoint/batch equivalence.
The pre-tuning implementation was `3b9526a`, with only the test duration and
read-only diagnostics changed. Both runs used the same catalogs and GPU.

The baseline had 41 active institutions with at least two units of unmet local
space demand, no unfinished construction, and **zero affordable extension
quotes**. The 25% spending limit was the observed bottleneck, rather than absent
demand. Procurement also counted absent members and omitted repair supplies.

The changes align local membership, request affordable repairs through ordinary
production, and replace the fixed investment fraction with the operating reserve
rule described above. Unmaintained buildings must be repaired before extension
orders are proposed. No institution receives new money or free materials.

| Seed | Years | Expansions before → after | Usable capacity before → after | Active treasury before → after | Population before → after |
|---|---:|---:|---:|---:|---:|
| 17 | 31 | 0 → 18 | 73.3 → 109.4 | 1647 → 647 | 749 → 758 |
| 81 | 30 | 0 → 7 | 74.9 → 88.9 | 1088 → 720 | 694 → 696 |
| 256 | 30 | 0 → 9 | 100.9 → 119.1 | 1069 → 558 | 759 → 765 |

The paired changes are a combined game-balancing experiment; population differences
are downstream observations, not an isolated estimate of construction's effect.
Space demand remains partially unmet. A cash-poor institution is still allowed to
remain small. The largest absolute economy residual was below 0.000007; the full
historical state matched after save/reload and differing batch sizes on seed 17.

Wooden vessels, shovels, axes and picks continued to be produced. No metal vessels
or roof tiles were produced in either ensemble. Their relative service/cost still
favors timber here; no random demand or price subsidy was introduced to fill the
catalog. A controlled all-materials-stocked fixture verifies timber, masonry/tile,
and metal construction are each selected when their local prices favor them.
This does not establish that ordinary generated worlds produce those price regimes.

Additional fixtures check zero-cash/zero-labor repair orders, shared procurement
limits, exact-supply repair execution, and larger upkeep reserves when component
prices rise. This is small-world game tuning, not empirical calibration or a
long-term guarantee of institutional solvency.

Tuning-pass checks: 41 ordinary library tests passed (48 hardware tests remain
ignored in that run); the three-seed GPU fixture, three institution-capacity
fixtures and minority-founding fixture passed explicitly. Clippy with warnings
denied, formatting, and the repository artifact check passed.
