# Progressive cash tax and town-sponsored practical research

Two opt-in experiments address different circulation constraints:
`--household-wealth-tax[=true|false]` and `--practical-research[=true|false]`.
Omitting either retains its archived setting; older worlds initialize disabled.
These are game policies, not estimates of historical tax systems or invention rates.

## Annual household cash tax

Collection follows completed household food/clothing settlement at annual monthly
boundaries, before existing estate review. Six months of full household food need
at the current food quote is exempt. Marginal annual rates are 2% on cash between
six and twelve months, 5% between twelve and twenty-four, and 10% above that.
Crossing a bracket does not apply its rate to the whole balance. Need is the last
completed household dietary projection, not an invented resident count.

Only represented, resident households in inhabited towns participate. Households
with recorded debt are excluded conservatively; vacant estates continue through
their separate review rules. Existing committed capital and escrow are outside
household cash. There is no assessment when usable food need is absent.

Collection requires this month's completed work by the current named local office.
At most 25% of that delivered administration supports tax collection, at 0.002
worker-months per eligible account; insufficient capacity scales all local
assessments proportionally. Estate review uses a separate 25% share. No new work
or payment is invented. This is a scoped use of paid administration, not a claim
that every administrative duty has a comprehensive allocation policy.

Actual payments move from household wallets to the controlling council. Receipts
retain opening cash, food cost, assessment, collection fraction, payer and recipient.
Household expenditures reconcile against these receipts. Repeated calls in the
same year do not collect again. Tax receipts are not income or new money. Councils
can spend through their existing relief, support, service and research mechanisms;
taxes are not automatically earmarked or returned to their originating town.

## Practical research pilot

The pilot targets weaving, metalworking and preservation using existing topic IDs.
Crop-calendar knowledge is the starting prerequisite. A candidate must lack the
technique locally and personally, have unmet demand for its product, and have
actual trial inputs (flax, generic metal-bearing ore, or wheat respectively).
Mineral-specific experimentation and further topics remain extensions.

A quarterly cultural work plan captures the topic and reserves 0.1 worker-months
for its named actor. Execution rechecks the dated plan, presence, knowledge,
materials, funding and remaining granted work. A completed experiment cannot repeat
under the same plan. Other cultural activity competes for the existing allowance;
this is not additional free time. The pilot currently requires an eligible household
head with an account; it is not research by every resident or a new researcher roster.

Council funding must cover wages (18 kg food-price equivalents per worker-month)
and 0.1 kg of trial material without exceeding 1% of current treasury. On execution,
wages enter the researcher's household account and material payment enters town
operating cash. Trial material leaves actual stock and enters existing waste/C/N/P
returns. No payment or progress occurs without work. Forecast funding is not escrow:
intervening council expenses can prevent execution and leave work uncompleted.

Progress accumulates in the researcher's existing study record: completed work
relative to three worker-months, modified by curiosity. Incomplete trials consume
resources without unlocking production. Completion grants the actual person the
existing topic with event provenance; closing knowledge synchronization makes it
available to later production. It does not create fiber, ore, workshops or workers.
Teaching and manuscripts retain their existing rules. This pilot adds local
experimentation, not remote teacher recruitment or automatic universal diffusion.
Knowledge can still leave or die with its holder. Research records and policy
settings serialize with existing history.

## Verification and comparison

Controlled tests cover marginal brackets, protected reserves, completed-office
requirements, exact cash transfer, receipt reconciliation, repeated annual calls
and serialized continuation. A research fixture covers unavailable money/materials/
work, finite material and money residuals, accumulated learning, named provenance,
serialized cultural state and repeated-plan prevention.

Matched baseline/tax/research/both comparisons use seeds 1024, 256 and 409 with the
same 32/32 founding archives and fifty-year horizon as the clothing quote screen.
Clothing, named office work, inheritance, contract/demand staffing and stock recovery
are enabled; credit, issuance and estate reclamation disabled. No histories or raw
logs are committed; outputs remain under ignored `output/civic-production-screen/`.
Measure actual collection, completed learning and production, not only policy
activation or treasury balances.

### Initial four-arm screen

All twelve runs completed. No new techniques completed in any initial research
arm despite 95–382 trials per world. Inspection found progress distributed across
rotating actors, including researchers dying before completion. Merely increasing
funding was insufficient. A follow-up retains all budgets and learning rates but
prefers an eligible in-progress researcher on alternate quarters. Other quarters
retain ordinary actor selection. Dead, absent, unfunded or materially blocked
researchers cannot receive this priority; completion removes eligibility.

| Seed | Arm | Population | Ending hunger | Operator work | Operator margin | Tax collected | Council cash | Trials |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1024 | baseline | 155.88 | 0.05898 | 14.05 | 62.19 | 0.00 | 3675.48 | 0 |
| 1024 | tax | 151.37 | 0.06718 | 14.37 | 58.66 | 14550.77 | 7808.95 | 0 |
| 1024 | research | 154.94 | 0.09349 | 12.99 | 54.42 | 0.00 | 1411.26 | 144 |
| 1024 | both | 150.40 | 0.08831 | 13.75 | 55.31 | 15515.48 | 8376.04 | 382 |
| 256 | baseline | 364.14 | 0.01677 | 4.69 | 19.93 | 0.00 | 2203.14 | 0 |
| 256 | tax | 359.46 | 0.01882 | 4.69 | 19.93 | 8336.53 | 7989.12 | 0 |
| 256 | research | 362.76 | 0.01398 | 4.69 | 19.41 | 0.00 | 810.11 | 120 |
| 256 | both | 364.81 | 0.01984 | 4.69 | 19.41 | 6901.62 | 4516.33 | 240 |
| 409 | baseline | 345.34 | 0.04735 | 170.44 | 896.04 | 0.00 | 1153.20 | 0 |
| 409 | tax | 344.90 | 0.04040 | 168.27 | 900.49 | 11807.10 | 6516.15 | 0 |
| 409 | research | 344.11 | 0.04252 | 186.81 | 979.21 | 0.00 | 2868.89 | 95 |
| 409 | both | 345.01 | 0.04268 | 164.28 | 871.44 | 10466.82 | 4227.65 | 229 |

Tax-only council balances rise in all three worlds, while population falls slightly
in all three. Hunger worsens in two and improves in one. This does not support
enabling higher cash taxation by default. Research-only also has mixed outcomes
and initially fails its intended learning mediator. Seed 409's baseline history
matches the preceding clothing-quote export exactly after removing only new
default fields. Eleven targeted civic/GPU tests, 198 ordinary library tests
(155 ignored), nine audit tests and strict library/binary Clippy pass before the
research-continuity refinement; its targeted tests also pass.

### Retained research continuity screen

The six follow-up runs complete techniques in every research-enabled world.
Learning completions count person/topic acquisitions, not necessarily distinct
inventions. The table uses the final alternating-continuity policy; baseline and
tax-only arms above are unaffected because research is disabled.

| Seed | Arm | Population | Ending hunger | Operator work | Operator margin | Active operators | Completions | Cloth made (kg) | Tax collected | Council cash |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1024 | research | 154.03 | 0.04319 | 51.17 | 260.68 | 1 | 1 | 735.39 | 0.00 | 1278.42 |
| 1024 | both | 148.20 | 0.05821 | 82.67 | 422.22 | 1 | 4 | 786.02 | 16497.43 | 14466.36 |
| 256 | research | 352.93 | 0.02486 | 7.25 | 23.11 | 0 | 2 | 802.46 | 0.00 | 2030.06 |
| 256 | both | 367.29 | 0.01606 | 9.24 | 28.40 | 1 | 3 | 876.86 | 8189.73 | 9036.50 |
| 409 | research | 346.45 | 0.04395 | 165.57 | 871.05 | 1 | 1 | 1303.87 | 0.00 | 303.05 |
| 409 | both | 345.94 | 0.04512 | 175.40 | 868.44 | 2 | 4 | 1404.50 | 10544.08 | 3848.35 |

Continuity retains the same learning rate, inputs and funding limits. It repairs an
observed coordination problem instead of declaring every trial successful. Seed
256 now produces cloth and, in the combined arm, ends with one operator instead
of none. Seed 1024 gains appreciable production and operator work, but neither
arm restores baseline population. Seed 409 outcomes are mixed. Tax-funded council
balances can still rise substantially without corresponding welfare gains.

Keep both policies opt-in. The evidence supports a functioning tax transfer and a
functioning local learning-to-production connection, not universal economic recovery.
Local knowledge preservation, wider research materials/topics, explicit spending
priorities, vacant estates and off-site mining remain separate work. This is an
18-run, fifty-year screen; no century, resolution or cross-hardware study was run.

Final verification: 198 ordinary library tests pass (155 ignored); eleven targeted
civic tests including GPU tax/research fixtures pass; nine Python audit tests,
strict library/binary Clippy and native build pass. All eighteen native runs finish.
Maximum absolute independently audited endpoint money residual across both screens
is below 2.85e-7. Compilations overlapped some runs; recorded timings are not isolated
performance benchmarks. There were no failed tests or native runs in these screens.
