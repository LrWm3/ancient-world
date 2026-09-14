# Optional system startup controls

New desktop and CLI histories enable the original world/history extensions by
default: social history, politics, governance, offices, shipping, expeditions,
living ecology, shared resources, production, facilities, markets and fisheries.
The more recent policy experiments are also in `System::ALL`, but remain **opt-in**.
Registering an experiment does not change its balance default. Use
`System::default_enabled()` rather than assuming every registry entry defaults on.
Patron aid retains its separate default-on founding control.

Desktop: before founding, expand **Optional systems and experiments**. Recent
experiments are marked **opt-in**. Checking a system enables its structural
prerequisites; unchecking a prerequisite suppresses dependents. Existing live
policy controls remain available after founding.

CLI:

```sh
# Standard extensions on, recent policy experiments off:
cargo run --release -- --headless --civilizations 5 --history-years 10
# No voyages/specimens; retain the rest:
cargo run --release -- --headless --civilizations 5 --disable-system expeditions
# Explicit enable and disable:
cargo run --release -- --headless --civilizations 5 \
  --enable-system adaptive-prices --disable-system adaptive-fishing
```

Both flags accept comma-separated names and can be repeated. `--help` lists the
complete registry. Earlier `--society`, `--politics`, `--governance`, `--offices`,
`--shipping`, `--expeditions`, `--discoveries` and `--living-world` flags remain
explicit enable aliases. Contradictory requests and an explicit enable with a
disabled prerequisite fail with an error. Merely leaving a dependent at its default
does not override a disabled parent.

The same settings can be saved in the planet configuration:

```toml
[systems.overrides]
expeditions = false
adaptive-prices = false
```

Absent entries use their per-system default. CLI overrides take precedence over configuration entries.
Startup options override the corresponding economy-catalog booleans; numeric catalog
parameters remain as supplied. Disabling society also suppresses politics,
governance, offices, shipping, expeditions, discoveries, enterprises and payroll.
Disabling living-world suppresses environmental returns. Shared resources are
required by mineral processing, which is required by alloys. Workshop specializations
require workshops; supplier profitability requires export contracts. Fishery
subsystems require their parent fishery settings.

## Recent registered policies (default off)

Every name below works with `--enable-system` / `--disable-system`, TOML
`systems.overrides`, and the desktop startup list. The previous
`--name[=true|false]` flags are compatibility aliases routed through the same
implementation; contradictory alias/registry requests fail before GPU startup.

| Area | Registry names |
| --- | --- |
| Credit and issuance | `council-credit`, `institution-credit-lenders`, `institution-credit-operating-reserve`, `commercial-credit`, `service-order-credit`, `shared-issuance` |
| Procurement and work | `service-order-procurement`, `contract-workshop-staffing`, `demand-workshop-staffing`, `named-office-service` |
| Household distribution | `household-estate-inheritance`, `household-estate-reclamation`, `household-clothing`, `household-wealth-tax`, `council-welfare-reserves` |
| Food and observation | `needs-based-food`, `gradual-nutrition`, `food-solidarity`, `municipal-food-relief`, `demographic-audit` |
| Knowledge and logistics | `practical-research`, `staged-harbors`, `abandoned-stock-recovery` |
| Export settlement | `export-default-recovery`, `delivery-paid-exports` |

Credit subpolicies can be configured while new lending is disabled: enabling
institutional offers or service-order eligibility does not itself enable credit.
Structural prerequisites still apply (households/councils need society, harbor
staging needs shipping, office service needs offices, workshop staffing needs
enterprises and workshops). Gradual nutrition and demographic auditing retain
their existing demographic-mode restrictions. No configuration switch clears
loan obligations or resets an issuance authorization window.

Enabled policy is distinct from delivered capacity. Wealth-tax collection and
estate reclamation require completed `named-office-service` work; the policy
switches do not implicitly fund or activate that service. Enable it explicitly for
an operational comparison, then inspect collection/review receipts. Available
staff, funds, taxable balances and eligible estates can still prevent execution.
See the [growth and circulation screen](growth-nutrient-screen.md).

```sh
# New history with selected experiments:
cargo run --release -- --headless --civilizations 5 \
  --enable-system household-wealth-tax,council-welfare-reserves,practical-research
# Change only a live policy in an existing history:
cargo run --release -- --headless --load output/example.world --epochs 0 \
  --disable-system council-credit --history-years 1
```

Numeric controls are not binary systems: `--founding-food-months`,
`--base-granary-months`, `--service-procurement-share` and `--crop-yield-scale`
retain their current validation and ownership. The [farm nutrient screen](growth-nutrient-screen.md)
also exposes numeric retention and geological-release controls, without new binary systems.
New founding food/storage defaults
remain 48 months; this registration does not alter that balance change.

## Existing saves and the library

Loading without explicit overrides preserves the saved baselines and policies,
including histories made before this default change. A request containing only recent registered policies changes only those policies,
using existing baselines. Requests for the original startup systems still apply
the startup configuration, adding missing enabled baselines and applying its
editable rules. Established baselines containing persistent
people, sources or cargo cannot be removed through startup flags: requesting that
fails before edits. Choose those omissions before founding a new history. This
avoids deleting inventories or stranded journeys. Policy switches that already
support live changes can still be changed through their existing controls/APIs.

`Generator::apply_systems(&Systems)` is the shared desktop/headless startup API,
called after successful founding and optional catalog configuration. The low-level
`found_civilizations*` and individual `enable_*` methods remain explicit for diagnostic
fixtures and callers constructing custom baselines. They do not apply the application
preset automatically. `Generator::apply_registered_policies` stages explicit live
policy changes on a history copy and commits the batch only after every change
succeeds. Unspecified policies retain their actual saved state. Loading imports
older policy fields into the registry without changing history, so old CLI
experiments become discoverable too. `History::registered_policy_enabled` reads
the actual policy; `Config.systems` records the configuration choices. No dense GPU
state layout changes are needed.

## Original startup verification

Tests cover default-on selection, dependency suppression, conflicting explicit
requests, option-name parsing and configuration serialization. The GPU startup
fixture builds all-on and all-off histories, checks enabled baselines/rules, rejects
removal of an established social baseline without mutation, and compares batched
history with save/resume continuation. These short checks verify startup integration
and continuation, not long-run economic balance or performance.

Observed validation on Quadro RTX 5000 Max-Q/Vulkan: all three system tests passed,
including all 38 individual disables and the GPU continuation fixture. The CLI
parsing test, 55 ordinary library tests, Clippy and formatting passed. The ordinary
library command skipped 53 hardware tests; the startup GPU fixture was run explicitly.
Two CLI smoke histories used seed 42, terrain 32, ecology 16, zero geological epochs,
three founding civilizations and twelve history months:

| Startup | Settlements | People (rounded CLI report) | Largest absolute managed relative residual |
|---|---:|---:|---:|
| All extensions enabled | 3 | 357 | 6.74e-7 |
| Society, living world and adaptive labor disabled; adaptive prices explicitly enabled | 3 | 366 | 6.27e-7 |

Exported state confirmed presence/absence of the selected baselines and policy
booleans, including dependent food-security staffing. Contradictory CLI requests
failed before GPU initialization. These worlds start from initialized terrain and
are short startup smoke tests, not a calibrated performance or seed comparison.
Generated JSON and logs remain under ignored `output/system-options/`.

## Registry expansion verification

The registry/CLI unit checks compare `System::ALL` to the parser's variants,
check unique names, default-off experiments, per-entry serialization and all 24
compatibility aliases. The optional GPU policy fixture exercises each binding,
preservation of unrelated settings, missing-parent errors, atomic failed batches,
and finite issuance authorization. The startup fixture checks archive continuation.

```sh
CARGO_INCREMENTAL=0 cargo test --lib --bin ancient-world
CARGO_INCREMENTAL=0 cargo test --lib registered_policies_apply_preserve_and_resume -- --ignored --nocapture
CARGO_INCREMENTAL=0 cargo test --lib all_on_and_disabled_startups_resume -- --ignored --nocapture
```

The five-year founding regression is also rerun to detect unintended default
changes. These are registration and compatibility checks, not a balance claim
for enabling all experiments together.

Expansion results on the available Quadro RTX 5000: 213 library tests and 16 CLI
checks passed; 157 hardware tests remained ignored in the ordinary library run.
Both targeted GPU registry/startup tests passed, including old-entry import and
exact save/resume continuation. All 12 founding-regression arms passed with the
same reported populations and 15 legacy shortage town-months as before this change.
Clippy passed with warnings denied. A one-year replay of the same seed-1024 archive
using `--household-wealth-tax=true` versus `--enable-system household-wealth-tax`
produced identical full history exports. The archive's existing living baseline
and disabled council-credit/issuance policies were preserved. Raw outputs remain
under ignored `output/`; no experiment artifacts are committed.

`municipal-food-relief` is an opt-in town-funded purchasing transfer after council
and household help. It protects a working-cash allowance and only backs purchases
with opening food stocks. See [municipal food relief](municipal-food-relief.md).

Institution service comparisons can independently enable `institution-working-core`
(core rather than full-membership operating space) and `institution-operating-funding`
(quoted operating budgets instead of legacy donations). Both retain their existing
archived culture policies and are opt-in. Neither enables named administration or
changes institutional work priority. See [the growing-world service comparison](institution-growth-followup.md).

The existing food-connection investment mechanism has a numeric planning horizon:
`--food-connection-months 0..24`, stored as
`production.food_connection_target_months` in the economy catalog. The default
remains three months. This changes prospective construction demand, not available
food or vessel capacity. See [the horizon experiment](food-connection-horizon.md).

`municipal-welfare-reserves` selects needs-first allocation of municipal surplus
for food purchasing assistance. It requires `municipal-food-relief` to actually
transfer funds; the controls remain independently enableable. The default is the
existing 5% surplus allowance, and disabling the new option restores it. Both
policies protect working cash and require available food. See the
[matched-checkpoint comparison](municipal-needs-first.md).

`ruin-resettlement` is an opt-in society/politics policy for funded household restoration of
abandoned sites. It uses managed crops and existing land routes. Old archives keep
it disabled; already-funded journeys and dated ownership notices continue when new
proposals are disabled. See [ruin resettlement](ruin-resettlement.md).
