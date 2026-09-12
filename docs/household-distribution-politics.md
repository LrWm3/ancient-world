# Council household-distribution policies

Annual politics can change common food entitlement, municipal payroll share,
ownership dividends, council relief budget and relief's food-coverage target.
These are toy political platforms, not estimates of historical policy.

| Governing interest | Common food | Payroll | Dividends | Relief | Food target |
| --- | ---: | ---: | ---: | ---: | ---: |
| Growers | 65% | 25% | 1% | 8% | 90% |
| Merchants | 35% | 20% | 4% | 4% | 80% |
| Retainers | 40% | 25% | 2% | 4% | 80% |
| Artisans | 50% | 35% | 1% | 7% | 90% |
| Scholars | 50% | 25% | 1% | 6% | 85% |
| Congregations | 65% | 20% | 0.5% | 12% | 95% |
| Bread leagues | 80% | 30% | 0% | 18% | 100% |
| Revivalists | 70% | 20% | 0.5% | 12% | 95% |
| Warbands | 40% | 30% | 2% | 3% | 80% |

Completed household food deficits, weighted by food need across currently
controlled towns, alter these targets. Pressure is clamped to 0–1 from four times
the deficit fraction. At maximum pressure the platform adds 15 percentage points
of common entitlement (capped at 95%), 8 points of relief spending, and 15 points
of target coverage (capped at 100%); its dividend target falls to zero.

Each annual decision moves at most 5 points for common/payroll/coverage, 2 for
relief and 0.5 for dividends. No decision is made without observed household need.
Reviews cannot replace outstanding proposals or run twice in one month. The rates
are initial game settings; long-run balance remains unverified.

## Timing, money and food

Respond schedules policy; next month's Open activates it once. Events carry the
civilization reference, and activation links to the decision. Effective and
pending policies persist. Old archives without local policy inherit global
household settings until an annual political decision. The council inspector
shows active and pending shares.

The controlling council governs distribution in all its towns. Original founding
provisions still taper from communal access toward that policy; daughter towns
use its common share directly.

- Common food is entitlement to existing food, not new harvest.
- Municipal payroll uses town cash and existing work eligibility. Direct employer
  contracts are separate; raising the cap does not create jobs or labor.
- Dividends use remaining town cash and ownership shares.
- Relief spends council cash, apportioned across unmet requests in all controlled
  towns. A 100% target cannot guarantee coverage when food or money is missing.

Greater distributions can leave less cash for subsequent construction, shipping
and services. This feature does not change crop output or mortality coefficients.
It uses observed shortage, not a forecast of future treasury solvency or a
negotiated legislative coalition.

Explicit `configure_household_economy` and `configure_household_relief` calls
override their respective fields across councils and cancel pending distribution
changes. Later annual politics can revise them again. Direct edits to the global
baseline only affect councils without an active local policy.

## Verification

The platform fixture checks faction contrasts, hardship response and bounded
long-run policy values. A GPU-founded retail fixture checks delayed and once-only
activation, serialized continuation, increased entitlement in the affected
jurisdiction, unchanged entitlement elsewhere, and unchanged total money.
These checks demonstrate mechanics, not an established solution to century-scale
population decline.

Verification run (2026-09-12, Quadro RTX 5000 / Vulkan):

- All 15 household-economy tests passed with GPU cases enabled.
- Both politics-filtered tests passed, including crisis/recovery and legacy IDs.
- Ordinary library suite: 133 passed, 114 GPU/long fixtures ignored.
- Frozen scheduler comparison passed seeds 17, 81 and 256 at terrain 64 /
  ecology 16 through 36 months: monthly, batched and archived continuation agreed.
- The family-gift fixture required an explicit council-policy override after its
  60-month history; changing only its global baseline no longer disabled active
  local payroll/relief. Its intended isolated gift comparison passes again.
- Review found that annual politics temporarily takes ownership of controller
  state. Distribution decisions now run after that state is restored, so observed
  food deficits and eventual retail policies use the same political jurisdiction.

No century-scale balance claim is made for this change.
