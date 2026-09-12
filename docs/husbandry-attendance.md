# Husbandry uses granted farm attendance

Named farming already limited planting and harvesting. The managed herd loop did
not use that attendance: it delivered stored feed, collected products and
slaughtered animals even with no available farm workers.

Those three tasks now use the existing granted/requested farm-attendance fraction.
Feed delivery is capped by both attendance and the actual stored feed inventory.
Product collection and slaughter use attendance as well as existing biological and
material bounds. Reproduction still follows actual nutrition and carrying capacity;
mortality continues when nobody attends. Uncollected nutrient leftovers follow the
existing respiration/detritus path. This is a managed-ration herd abstraction;
independent grazing, separate herder occupations and escaped animals are not added.

The fraction belongs to the shared farming allowance. It does not reserve another
set of people, issue another wage or count husbandry output as additional work.
When named agricultural refinement is disabled, attendance remains one and the
aggregate path retains its previous arithmetic. Partial named attendance can now
reduce animal output, so individual histories are not promised to match older
versions. This change follows, and is not included in, the council/institution
ensembles started earlier in this work series.

The production attendance fixture compares: available
farmers, all residents occupied elsewhere, and aggregate staffing share the same
opening herd and goods. Checks require finite feed use, no unstaffed products,
continued biological loss, real household earnings and batch/reload consistency.

All three hardware-backed production participation fixtures pass: agriculture,
combined extraction, and construction. Each includes the three staffing controls,
actual household wage attribution, idempotent settlement and a twelve-month
batch-versus-reload continuation. The unavailable named workers deliver zero feed
and collect zero products while herd mortality continues; aggregate staffing
retains at least the output of partially available named farmers. The normal
monthly continuation validates the existing resource ledgers. Long-run husbandry
balance and a distinct herding-labor budget remain untested by this increment.

Reproduce with:
`cargo test --lib civilization::production_forecast::agriculture_tests -- --ignored --test-threads=1`.
