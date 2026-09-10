# Optional system startup controls

New desktop and CLI histories start with every implemented optional extension in
`System::ALL` enabled. The registry covers social history and indicators, politics,
governance, offices, shipping, expeditions and specimens, living ecology, shared
resources, mineral/alloy processing, environmental returns, enterprises and
occupational payroll. It also enables the optional production, facilities, market,
farming and fishery rules, including experimental adaptive prices and fishing.
This changes default behavior; it is not evidence that the combined settings have
been calibrated over long histories. Diagnostic ablations such as open wildlife
barriers and fixed labor remain off. They remove normal constraints rather than
add systems. Patron aid remains default on with its existing separate control.

Desktop: before founding, expand **Optional systems (default on)**. Checking a
system enables its prerequisites. Unchecking a prerequisite disables dependent
systems. Existing live policy controls remain available after founding.

CLI:

```sh
# All extensions on without a chain of enable flags:
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

Absent entries default on. CLI overrides take precedence over configuration entries.
Startup options override the corresponding economy-catalog booleans; numeric catalog
parameters remain as supplied. Disabling society also suppresses politics,
governance, offices, shipping, expeditions, discoveries, enterprises and payroll.
Disabling living-world suppresses environmental returns. Shared resources are
required by mineral processing, which is required by alloys. Workshop specializations
require workshops; supplier profitability requires export contracts. Fishery
subsystems require their parent fishery settings.

## Existing saves and the library

Loading without explicit overrides preserves the saved baselines and policies,
including histories made before this default change. An explicit override requests
application of the startup configuration, adding missing enabled baselines and
applying the editable rule switches. Established baselines containing persistent
people, sources or cargo cannot be removed through startup flags: requesting that
fails before edits. Choose those omissions before founding a new history. This
avoids deleting inventories or stranded journeys. Policy switches that already
support live changes can still be changed through their existing controls/APIs.

`Generator::apply_systems(&Systems)` is the shared desktop/headless startup API,
called after successful founding and optional catalog configuration. The low-level
`found_civilizations*` and individual `enable_*` methods remain explicit for diagnostic
fixtures and callers constructing custom baselines. They do not apply the application
preset automatically. `Config.systems` archives the startup choices. No dense GPU
state layout changes are needed.

## Verification

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
