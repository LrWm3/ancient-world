# Local unclaimed-estate cash review

Opt-in implementation pilot; the first balance comparison is mixed. This handles local cash, not physical
recovery from distant abandoned settlements.

## Eligibility and boundary

`HouseholdEconomy.reclamation.enabled` permits review after household retail
settlement in Execute/settle. Current food purchases have already settled. Each
inhabited town may process cases using at most 25% of its completed named office
attendance, at 0.025 worker-months per case. This is a throughput rule for estate
review within general administrative service, not a second personal work grant.
Other administrative functions still use the existing office-capacity model;
action-specific court scheduling is not implemented here.

Only completed current-month service by the current living officeholder under the
same controller qualifies. Old, unsettled and unavailable work cannot finance cases.
Oldest vacancies are reviewed first, with stable household IDs breaking ties.
The monthly guard prevents duplicate reclamation.

An eligible estate has been vacant at least five years, has no known living member,
is not traveling/lost, and has no live debt or unrecovered default claim. Any
connection through recorded parent/child or marriage relationships to a living
person prevents reclamation. This deliberately conservative check includes distant
kin and former spouses; it will miss some legally reclaimable estates rather than
silently resolve unknown inheritance rights.

Three months of the last completed food need at the current local food quote stays
in the wallet. At least one currency unit of surplus is needed to initiate a case.
Only the amount representable in the town's f32 cash account is debited from the
estate. The account's `reclaimed` ledger and dated receipts preserve the source,
local destination, official, controller, food reserve and service capacity.

The cash becomes town operating money, usable by subsequent ordinary spending.
No money, work or population is created. Ownership shares, artifacts, recorded
claims and household identities remain unchanged. The policy does not determine
that anonymous cohort dependents have disappeared. Existing future dividends can
accumulate and be reviewed again; this is not a permanent transfer of property to
the government. Retrospective claims and restitution remain future work.

## Controls

- `--household-estate-reclamation[=true|false]` controls the cash-review policy.
- `--named-office-service[=true|false]` independently enables actual office work.
- Native omission preserves archived settings. Reclamation does not silently
  enable attendance; without completed service it performs no transfers.
- The monetary runner exposes both switches and applies each to every arm.
  Comparisons must hold named attendance constant when testing reclamation.

The prior monetary checkpoints had office service disabled. Comparing a run with
both switches against that older baseline would conflate changed labor scheduling
with cash reclamation. Use named attendance in both control and intervention.

## Verification

The capacity unit test and hardware-GPU transaction fixture pass. The latter
checks that unperformed office work and known living kin block reclamation, that
cash and ownership reconcile, that repeated calls do not duplicate transfers,
that serialization preserves the operation, and that stale work cannot fund a
subsequent month's cases. A deliberately corrupted cash receipt is rejected.
This is a transaction/serialization fixture, not yet a full natural-history
checkpoint-continuation comparison.

Strict all-target Clippy, the native build, four cash audit tests and twelve
monetary report tests pass. Matched history evaluation follows below.
Abandoned town recovery remains a separate unfinished connection requiring actual
access and transport, rather than adding its cash to a distant town by index.


## First matched history comparison

Seed 1024, frozen founding checkpoint, 32/32 terrain/ecology resolution, five
civilizations and 600 monthly history steps. Both sides enable named office
attendance, local inheritance, delivery-paid exports, contract-aware and
demand-aware workshop staffing, and service procurement at a 0.25 surplus share.
Only reclamation changes between sides. Four monetary arms were run on each
side, sequentially on the Quadro RTX 5000 using one frozen native binary.
All eight runs exited successfully, taking 16.50–16.98 seconds each. These are
small diagnostic worlds, not default-resolution performance estimates.

| Measure | Credit, off | Credit, on | Credit + issuance, off | Credit + issuance, on |
| --- | ---: | ---: | ---: | ---: |
| Reclaimed cash, cumulative | 0 | 6,038.80 | 0 | 10,595.00 |
| Completed cases | 0 | 20 | 0 | 59 |
| Retained household cash, endpoint union | 20,457.57 | 14,434.17 | 16,900.06 | 14,926.92 |
| Population | 154.824 | 151.573 | 155.181 | 155.162 |
| Need-weighted household hunger | 0.06166 | 0.07047 | 0.05348 | 0.05428 |
| Operator revenue minus wages and rent | -225.32 | -233.11 | -80.16 | -80.16 |
| Surviving operators / all operators | 0 / 12 | 0 / 12 | 0 / 7 | 0 / 7 |

The retained-wallet union means vacancy or registration at an abandoned site;
it is not a claim that every balance is available for confiscation. Cumulative
reclamation is not an additional money stock, and subsequent transfers, income,
food spending and changed history mean it need not equal the endpoint reduction.
Maximum absolute relative cash-audit residual was 1.53e-7. No loans were issued.
Within each side, baseline equals credit and issuance equals combined after
removing only the credit record; these pairs are not independent replications.

The first direct transfer occurred in month 269: estate 72 transferred 629.11 to
its own town, retaining 9.21 for food, after 0.10 worker-months of delivered
administration. Later small cases process new surplus or a reduced food reserve.
Thus the intervention crosses the intended accounting boundary. It does **not**
prove improved household welfare or viable workshops.

Without issuance, ending town cash rose from 6,969.07 to 8,269.09, but part of
that increase accumulated at a nearly empty town: Dumaslitur held 1,552.18 with
0.987 population, versus 70.26 and 1.004 in the control. Litugie still held large
food stocks while two other inhabited towns had no food at the endpoint. These
observations identify remaining allocation and transport questions; they do not
establish a single cause for the later population difference.

Keep reclamation opt-in. Moving money out of a retained wallet can simply move
it into another poorly circulating account. Next work must connect usable local
surplus to funded production and actual transport, test household access, and
address workshop input feasibility. Remote abandoned-site recovery and late-claim
restitution remain unfinished. One seed does not establish a satisfactory policy.

Reproduce each side with `scripts/monetary_experiment.py`, a common founding
checkpoint and the settings above; pass `--named-office-service` to both and
`--household-estate-reclamation` only to treatment. Local outputs are ignored under
`output/estate-reclamation-screen/{off,on}/`; only this summary is committed.
