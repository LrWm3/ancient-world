# Army supply outcome comparisons

Army supply now uses the same projection/result receipt interface as relocation,
without changing provisioning, recruitment, battle or occupation rules.

At the start of `social_month` in Respond, capture opening armies before any social
mutation. Each soldier requires 18 kg food equivalent for that monthly supply step;
predicted consumption is capped by carried food. The existing rule forecasts 10%
losses whenever consumption falls short by more than 0.001 kg. This deliberately
records the current coarse threshold, not a proportional or empirical hunger model.

Named armies accumulate fractional expected losses and debit whole people; older
rosterless armies debit fractional manpower. `MilitarySupply` receipts retain the
aggregate conditional expectation in either mode and show actual losses without
explaining away rounding or carried fractional losses. Mode follows the army's
actual roster, not the settlement demographic switch.

Observe food and survivors immediately after supply attrition, before battle,
occupation, stranded-store disposal or return. Thus `soldiers_after_supply` is not
end-of-month deployed strength, and `food_consumed` excludes discarded provisions
when an army has no carriers. Existing ledgers still account for those later
transfers. The diagnostic neither supplies food nor applies deaths.

Receipts aggregate by origin and mode, at most two per site. Fingerprints include
army identity, arrival boundary, stocks, casualty remainder and member identities.
Capture checks existing same-month receipts before road weathering or other social
updates. A duplicate comparison is rejected without mutation. `social_month` now
returns errors to the coordinator instead of concealing a duplicate supply step.
This is a boundary guard, not transactional rollback of the entire monthly tick.

Latest receipts and cumulative summaries use existing persistence/reporting.
The global latest-receipt bound grows from sixteen to eighteen per site; no
per-army historical trajectory is retained. With resolution reporting absent,
physical history retains the same behavior.

## Verification

The Vulkan named-campaign fixture passes, including all four supply comparison
arms (named/legacy × provisioned/hungry). The declared-war campaign also passes twelve-month saved/batched continuation;
the three-seed frozen scheduler also passes monthly/batched/checkpoint equality. The controlled fixture uses actual funded campaigns and legacy archive
conversion. It compares
full provisions and a withdrawal of carried food back to the origin, preserving
world inventories. Reporting ablation leaves physical history identical. Serialized continuation
agrees, and repeated-boundary rejection preserves the complete history state.
Expected shortage deaths equal 10% of opening soldiers within 1e-5; actual deaths
plus survivors equal opening manpower. Named losses are integral; legacy losses
agree with the conditional expectation within 1e-5. Population and food residuals
remain unchanged within 1e-4.

Combat casualty forecasts, departure choices, resupply and reinforcement remain
unfinished. The 10% shortage threshold itself is a candidate for later balancing;
this increment measures it rather than changing it without evidence.


The older spontaneous-raid fixture remains unchanged. Turning on politics in that
fixture changes raid eligibility, so receipt continuation instead uses a valid
explicit war declaration and the normal aggregate-comparison configuration API.
Review caught a duplicate observation after combat; the campaign now checks that
cumulative expected and actual supply consumption agree, in addition to checking
whole-history continuation. No test treats battle deaths as provision losses.


Verification backend: Quadro RTX 5000 / Vulkan, development/test profile.
The regular library suite passes 129 tests with 113 hardware/extended tests
skipped. Separately executed campaign checks include named and legacy supply,
explicit-war checkpoint/batch equivalence, and finite occupation withdrawal.
The frozen scheduler fixture covers seeds 17, 81 and 256. These establish
boundary behavior and accounting, not the historical credibility of the shortage
threshold or broader warfare balance. No balance parameters changed.

All-target Clippy passes with warnings denied. Artifact-policy and whitespace
checks pass; generated logs and temporary checkpoints remain under ignored
`output/`.
