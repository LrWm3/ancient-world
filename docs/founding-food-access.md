# Founding food access

New histories make food in original landing settlements communally accessible.
This is a temporary entitlement policy over existing town inventory, including
arrival provisions. It neither creates food nor keeps a second copy of supplies.
Because founding provisions and subsequent harvests share an inventory, the policy
covers available food generally rather than tracking which meal came off a ship.

| Months since landfall | Common entitlement | Household purchasing |
|---|---|---|
| 1–12 | 100% of need | No food purchase needed |
| 13–60 | Linear taper to the configured common share | Pays for remaining need, subject to cash and food availability |
| 61 onward | Configured common share (default 50%) | Ordinary household retail |

Payroll and dividends continue during the communal phase, allowing wallets to
accumulate rather than requiring families to purchase provisions before their first
earnings. These transfers retain their existing funding limits. The taper is a
deliberate game policy, not an empirically calibrated economic transition; it does
not promise everyone will remain able to afford food afterward.

## Integration and controls

HouseholdEconomy.founding_access stores an optional FoundingAccess policy.
Generator::configure_founding_food_access accepts Some(FoundingAccess {
communal_months: 12, transition_months: 48 }) to configure it, or None to disable it.
Both intervals permit 0–1200 months; a zero taper switches to the target after the
communal interval. configure_household_economy sets the long-run common share and
ordinary income policy without resetting the founding schedule.
HouseholdEconomy::common_share_at(month, site.founded) exposes the effective rate.

Original landing sites are identified by their existing month-zero founding date.
Later daughter settlements get ordinary retail immediately. Reoccupation, migration,
policy changes and reloads do not restart the calendar. Older archives missing the
optional policy retain their previous access rules; enabling society in an already
advanced history does not automatically introduce it.

Reserve computes the effective entitlement once for each retail plan.
Relief requests subtract that entitlement before requesting council cash.
GPU consumption remains capped by physical supply; Execute/settle divides only
food actually eaten into communal and purchased portions. Fully communal access
can prevent an affordability shortage but cannot prevent famine when stores and
production are insufficient. Existing ration priorities continue to apply.

## Verification and reproduction

Focused tests cover the first/twelfth/thirteenth/sixtieth month boundaries,
daughter towns, missing-field archives, zero-duration transitions, and serialized
policy continuation. A hardware fixture checks cashless founding access, genuine
hunger under an imposed physical shortfall, accumulation of income in the first
year, and month-12 checkpoint continuation through month 61 with monthly versus
batched execution. Older fixed-policy fixtures explicitly disable this new policy.

Run the household_economy library tests with --include-ignored --test-threads=1.
For matched seed evaluation, run the nutrition_evaluate example with:
--affordability --common-shares 0.5 --yields 0.33 --years 6.
Add --founding-access for the new transition and give each run a separate
--output path under output/.

The evaluation example retains static access by default to reproduce earlier
experiments. Its explicit --founding-access flag enables this transition and is
recorded in local output settings. Reported common share is the long-run target.

## Six-year matched check

Seeds 17, 81 and 256; crop yield 0.33; five requested founding civilizations;
terrain/ecology 32; one geological epoch followed by frozen history, individual
demography/nutrition and ordinary income/relief. Compare immediate static 50%
common entitlement with the one-year communal/four-year taper policy.

| Seed | Final population static → transition | Access gap / total need static → transition | Physical gap / total need static → transition |
|---|---:|---:|---:|
| 17 | 497 → 505 | 4.818% → 1.625% | 3.428% → 6.249% |
| 81 | 562 → 584 | 4.385% → 1.541% | 1.446% → 2.733% |
| 256 | 564 → 604 | 3.803% → 1.371% | 0.732% → 1.659% |

All seeds began with 9,120 kg-equivalent need and 127,858.926 available.
Static purchasing consumed 4,810; founding communal access consumed 9,120.
No additional inventory was introduced. Maximum population residual was zero;
maximum relative food accounting residual was below 1.67e-7.

The transition reduces exclusion for lack of purchasing power. It also increases
physical shortage later: more initial consumption and surviving residents change
stock depletion and subsequent demand. It is not a production improvement or a
guarantee of survival. Six-year, three-seed results are an initial behavior check,
not long-run balance calibration. Generated results remain ignored under output/.

All ten household-economy tests passed with hardware tests enabled, including the
month-12 save/reload followed by batched versus monthly execution through month 61.
Formatting, all-target Clippy with warnings denied and repository artifact checks
passed. Initial test failures exposed static-policy fixtures that needed an explicit
opt-out, a missing synthetic ration-need input, and endpoint rounding; these were
corrected before the final verification.
