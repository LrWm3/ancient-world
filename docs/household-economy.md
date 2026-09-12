# Household income and retail food

Households hold real cash wallets, separate from settlement operating cash and beneficial ownership of communal assets. New societies enable this model; older archives default to no household economy and retain pooled consumption until explicitly configured. No cash is minted at initialization.

The percentages below are initial defaults. Annual politics can now change them
per controlling council; see [household distribution policies](household-distribution-politics.md).

Each monthly production step has two CPU bookends around the existing GPU work:

1. Pay a finite payroll from settlement cash, capped by allocated adult labor and 20% of available cash. Allocate payroll among resident households using the industry-linked rule below; older policies retain equal division. Pay ownership-weighted dividends from 1% of the remaining cash.
2. Calculate age-weighted household food needs from known resident members and the pre-transition age cohorts. Reserve a common entitlement covering 50% of need. Households can purchase the remainder only up to their cash divided by the local food price.
3. Upload combined common and affordable demand as a consumption ceiling. The GPU produces food, consumes no more than available food and this ceiling, and applies age-cohort ration priorities and demographic effects. Unsold food stays in storage, subject to existing spoilage and capacity limits.
4. Divide actual consumed food into common rations and purchases. When supply is insufficient, paid food is allocated proportionally to funded demand after common rations. Transfer payments from wallets back to settlement operating cash. Unfilled demand is not charged.

Existing food and C/N/P ledgers record consumption once. Wallets are included in total money accounting. Each account independently reconciles wages + dividends - food spending - estate returns = cash. Transfers reconcile to the actual representable change in the settlement's f32 money pool; sub-ULP purchase charges can round down, leaving a tiny rounding discount, never an overdraft.

Known resident ages now weight each account's needs (10/18/14 food-equivalent kg per child/adult/elder month). Unrepresented cohort stock is shared across eligible accounts; overhanging named membership is scaled down within each age band. This preserves total cohort demand without inventing population. Common rations follow need, while purchases follow funded demand.

In individual demographic mode, realized household shortage replaces the age-band hunger contribution to each known resident's mortality probability. Disease remains a shared town exposure, and anonymous residents and aggregate mode retain age-band rates. There is one demographic commit, not a second starvation-death pass. The preceding completed household shortage also reduces next month's personal activity capacity by up to 35%; missing, stale or different-site observations have no effect. Birth opportunity still uses town nutrition and the existing individual age-structure adjustment. These are toy response rules, not physiological calibration.

Accounts still represent ownership households rather than independently simulated kitchens. Municipal payroll remains largely aggregate, while participating workshop workers and vessel crews receive their existing direct earnings. There are no household food stores or individual diets.

Wallet identity follows the existing household ID through head succession and relocation. Travelers cannot earn wages, dividends or buy resident rations; their wallet remains separately counted alongside existing journey cash and provisions. It is not counted again in the journey purse. Lost travelers' remaining wallets return to their origin estate's settlement pool at the next monthly preparation. No invented family or population is attached to a wallet.

## Interface and persistence

`History::household_account(id)` exposes cash, cumulative wages/dividends/spending/estate returns, and the last monthly need, common food, purchases and shortage. Settlement inspection shows these alongside household identity.

`Generator::configure_household_economy(common_share, payroll_share, dividend_share)` accepts fractions in 0–1 at a consistent boundary. It can establish an explicit zero-wallet baseline for an old society, and records policy changes. Setting common share to 1 makes dietary access communal while retaining income transfers. Account state and rules serialize in the existing archive header. The packed demographic GPU structure includes a retail entitlement vector; its size is derived rather than hardcoded.

`history_evaluate --no-household-economy` provides a pooled-food control for new runs. Reports include the full final account state. Do not disable the subsystem by removing funded wallets in a running world: that would discard real money. The evaluator control applies before any income is paid.

## Validation

Controlled fixtures check equal food needs despite unequal ownership, equal payroll but unequal dividends, finite purchases, cashless common entitlements, ledger reconciliation, corrupt account rejection and old archive defaults. The existing GPU ration sweep checks bounded consumption allocation; integration suites cover food/money/population conservation, inheritance, relocation, policy persistence and checkpoint continuation.

History records `food_access_crisis` after three months in which lack of funded entitlements leaves more than 10% of dietary need unconsumed despite physical supply. `food_access_recovery` requires six months without that level of exclusion; it explicitly does not imply physical food sufficiency. Episode state is saved, and notices link to a preceding policy or access crisis where available.

## First calibration

Paired seeds at crop yield 0.33 over 50 years, starting from 16 settlements:

| Seed | Pooled population | Household-retail population | Active retail settlements | Access crises | Access recoveries |
|---|---:|---:|---:|---:|---:|
| 17 | 2330.1 | 1913.6 | 20 | 97 | 93 |
| 81 | 2012.5 | 1720.2 | 17 | 98 | 95 |
| 256 | 2972.4 | 2370.8 | 19 | 32 | 29 |

Retail populations were about 15–20% lower, with more war declarations than the pooled controls. More settlements were active at the end, so lower total population does not mean every town collapsed. Account balances stayed finite, and maximum relative conservation residual was below 1.54e-5 in both configurations. Access episodes are debounced but can recur after recovery. These results establish functional feedback, not a final balance target or evidence that all inequality is realistic. The common ration share and income rates remain first-pass policy assumptions.

Reports: `output/household-retail-final.json` and `output/household-pooled.json`. Run the evaluator with `--seeds 17,81,256 --years 50 --crop-yield-scale 0.33`, adding `--no-household-economy` for the control. GPU timings overlapped tests and are not controlled performance benchmarks.

All 52 targeted tests passed: 24 library tests, 13 culture, seven markets, four politics, two GPU ration/archive fixtures, and two society integration tests. These include century histories and batch-versus-monthly checkpoint continuation with funded wallets. A final transfer-precision check also passed for withdrawals exceeding half a pool. Clippy with warnings denied, formatting and whitespace checks passed. The century political fixture now checks initial rival owners and persistent overlapping site claims rather than requiring rivals to remain politically separate after conquest.

## Targeted council relief

New societies now allow councils to spend up to 5% of their current treasury each month on household food access, targeting the purchasing power needed for 75% dietary sufficiency including common entitlements. This is additional to the existing annual council-to-settlement relief path: both debit the same finite treasury.

Requests are calculated after wages and dividends, deduct existing wallet cash, and exclude traveling households. Each council gathers all its towns' requests before allocating its budget proportionally to unmet purchasing power. Town order therefore cannot award first access to the fund. This targets affordability rather than promising that food physically exists: households keep grants they cannot spend until supply arrives, and that retained cash reduces future eligibility. Relief competes with the council's other funded activities; it cannot eliminate harvest shortages or make a bankrupt council effective.

Wallet accounting now includes cumulative `relief`: wages + dividends + relief - purchases - estate returns = cash. Council treasury and cumulative relief expenditure record the other side of each transfer. Inspectors display relief separately from earned income.

`Generator::configure_household_relief(treasury_share, food_target)` changes the bounded policy at a monthly boundary and records it in history. Older archived household economies default both new policy fields to zero; old accounts default relief received to zero. The evaluator's `--no-targeted-relief` supplies a paired control before historical advancement.

Tests cover finite council-to-wallet transfers, money conservation, independent account reconciliation, proportionate allocation under reversed request order, zero requests/budgets, oversupply, and old archive defaults.

### Relief comparison

Three paired 50-year runs at crop yield 0.33:

| Seed | Population without relief | Population with relief | Cumulative council grants | Access crises without / with |
|---|---:|---:|---:|---:|
| 17 | 1913.6 | 1905.5 | 3541.0 | 97 / 84 |
| 81 | 1720.2 | 1744.2 | 8329.4 | 98 / 66 |
| 256 | 2370.8 | 2394.4 | 4189.4 | 32 / 28 |

The population effect was modest and not uniformly positive. Fiscal opportunity costs and subsequent political/economic feedback remain; this comparison does not isolate each cause. Maximum relative conservation residual stayed below 1.54e-5. Reports are `output/household-relief.json` and `output/household-relief-control.json`. Three focused household tests and six political/society integration tests passed, including century runs and deterministic save/resume checks.

## Industry-linked household income

New household economies enable `occupational_payroll`; archived economies without
that flag retain equal payroll. Each household keeps four bounded earnings weights
(farming, forestry, mining, crafts) and its latest sector wages. A farmer
representative initializes the farming weight to 2; a craftworker initializes
crafts to 2; other weights start at 1. Households without a recorded representative
start as generalists. These are household livelihood proxies, not additional
workers or personal skill measurements. Profiles persist through relocation and
head succession.

The existing finite payroll is divided among sectors in proportion to the
**preceding production step's allocated labor**, then among resident households
in proportion to their sector weights. This is a lagged income rule, not payment
for verified completed orders: allocated labor can include unsuccessful work.
Traveling and lost households receive no resident income. Dividends continue to
follow beneficial ownership and are not conflated with wages.

After payment, each weight moves 2% of the distance toward `1 + sector labor share`
per monthly call, remaining within [1, 2]. The half-life of an initial difference
is about 34 months under identical opportunities. This is an explicit game-balance
assumption about gradual livelihood adaptation, not empirically fitted wage or
retraining data. No skill multiplier increases physical production. The fixed
four-element state leaves labor matching and independent employers open.

The household inspector exposes the latest sector payments. The account's
existing cumulative wage/cash identity still applies, and archives preserve both
profiles and latest payments. `Generator::set_occupational_payroll(enabled)` switches the rule at a completed boundary and records a policy event. Disabling the policy retains profiles but uses the
legacy equal split; it does not discard wallets. No new inventory, wage fund or
money source has been introduced.

Controlled tests reverse farming/craft labor while keeping payroll fixed, check
reversed recipient ordering and finite budgets, and preserve legacy equal-pay
behavior. A hardware-backed history fixture checks that the wage difference
changes funded food demand, that total money reconciles, and that serialized
household state continues identically through the next allocation. That fixture
is a monthly-bookend test, not a full GPU checkpoint or long-run calibration.
Independent firms, workshop ownership, operating accounts, insolvency and
household-specific worker capacity remain unfinished.

Verification for this increment: 23 ordinary library tests passed (35 hardware
fixtures ignored by that command); all five focused household tests passed when
hardware fixtures were explicitly included. The existing regional recovery
fixture also passed full archive reload and three-month batch-versus-monthly
continuation with industry-linked payroll enabled. `cargo clippy --all-targets
-- -D warnings` passed. Focused logs are retained in
[`evidence/household-livelihoods`](evidence/household-livelihoods).
No new seed ensemble has been run for this rule; the earlier household and relief
calibration tables describe the previous equal-pay model and must not be read as
validation of the new income distribution.

## Independent employers

[Workshop operators](workshop-operators.md) now contribute wages from separate
funded accounts. Their cash and material handling are documented there. Wallets
track `capital_invested` and `capital_returned` explicitly, extending the cash
identity without treating investment as consumption. Employer wages are included
once in cumulative wages and the craft-income display. The municipal craft work
basis excludes prepaid operator shifts before applying the existing town-cash cap.

## Resident nutrition integration verification

Focused fixtures use seeds 17, 81 and 256 at terrain resolution 32 and ecology
resolution 16. They enable society, politics and individual demography, transfer
existing town cash to one account, disable additional payroll/dividends/relief,
and compare that account with an unfunded resident household. The fixture supplies
the funded consumption total directly to isolate allocation from crop yields.
Funded accounts receive more food, lower personal hunger mortality probabilities,
and higher next-month nutrition capacity. Wallet serialization preserves exposure;
a different-site account observation cannot penalize a newly arrived resident.
This is a mechanism check, not a long-run famine calibration.

An analytical fixture checks unequal family sizes, anonymous residual cohorts,
overhanging identities and exact common/purchased food totals. A matched-random
mortality fixture verifies that household risk replaces average hunger mortality,
aggregate resolution ignores personal overrides, and snapshot replay retains its
inputs. The full individual-demography suite checks population authority,
observational comparison and checkpoint continuation. Existing household ledger
and relief fixtures remain regression checks for cash and food settlement.

Validation for this pass: 7 household tests, 10 individual-demography tests and
5 vessel tests passed with hardware-required cases enabled. The ordinary library
suite passed 106 tests (94 hardware cases ignored by that command). Clippy with
warnings denied and the repository artifact check passed. These checks include
checkpoint consistency but do not establish long-run population balance under
the new unequal food exposure.

## Matched nutrition control

`HouseholdEconomy.individual_nutrition` defaults to true, including archives that
omit it. Setting it false at a completed monthly boundary retains membership-based
food allocation, wages, wallets and individual birthdays, but omits household
mortality overrides and the next-month personal hunger work penalty. Town disease
and age-band mortality still apply. This isolates the feedback added by resident
nutrition; it is not the older equal-size household economy.

`cargo run --example nutrition_evaluate -- --years 20` compares both settings on
seeds 17, 81 and 256 at crop yield scales 0.33 and 0.15. The example uses one
geological epoch, frozen history, five requested civilizations and terrain/ecology
resolution 32, with society, politics, governance, offices and shipping enabled.
It writes monthly mediator measurements and resolution metrics to ignored
`output/nutrition-evaluation.jsonl`. Long-run population differences must be read
alongside immediate hunger/work/mortality changes; they are not isolated causal
estimates after the worlds diverge.

See [the scarcity comparison](household-nutrition-calibration.md) for the longer
paired run, its negative-recipe regression, results and remaining balance limits.

The [food-access comparison](food-access-calibration.md) separates physical food
gaps from entitlement/purchasing-power gaps, using the actual consumption boundary.
It varies common access independently of crop yield, without adding food or cash.
