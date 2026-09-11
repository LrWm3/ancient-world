# Work execution boundaries

This closes specific gaps in the monthly game schedule, not a general simultaneous
allocation solver. No additional terrain readback is introduced.

## Sea cargo and completed travel

New sea cargo stores a travel clock containing the dispatch month and remaining
nominal travel months. Opening a month advances one completed interval using the
previous month's funded fleet capacity divided by all outstanding cargo at each
endpoint. The weaker endpoint limits progress. Every shipment sees the same
snapshot of endpoint loads; delivering one shipment does not grant another a
second use of those crews. A half-staffed journey advances half a nominal month.
Zero staffing stalls travel. `arrives` becomes a revised projection, not a promise.

Calling the opening travel update again in the same month cannot use newly hired
crews. Skipping multiple months does not multiply one funding observation into
several months of work. The normal scheduler advances one month at a time. Clocks
survive archives; old cargo initializes remaining travel from its existing due
date. Configurations without vessel records retain scheduled travel. A remainder
below 0.0001 month is rounded to zero to avoid f32 payroll rounding causing an
entire extra month of delay.

Existing flood handling still governs blocked delivery and associated spoilage.
Crew shortfalls do not themselves apply that flood loss rule. Relief and expedition
journeys have their own travel models and are not silently converted into sea cargo.
This is aggregate endpoint staffing, not individual vessel assignment.

## Reserved cultural work

Reserve captures a dated site bundle containing the chosen actor, requested action
kinds, a teaching successor/topic, the selected institutional teacher, and the
identities and knowledge of those named participants. Institutions and eligible
local objects retain their own identity guards. Execution
checks those identities again. Death, departure, changed custody or local knowledge
can cancel the bundle rather than retargeting its grant to a replacement person or
object. The next regular reservation can reconsider the site.

Only requested action kinds can spend the grant. Execution retains live checks on
materials, cash, travel and eligibility. Goods are not escrowed. Requests compete
for a shared site grant and the existing execution priorities, rather than receiving
separate guaranteed material allocations. A bundle may therefore complete partly
or not at all. The calibrated default ignores unrelated residents' changes;
changed institutions or objects can still conservatively defer a bundle. This is not a claim that every social
operation has an immutable individual action record.

## Reserved research

A workshop plan pins each method's teacher and caps each specimen kind's processing
quantity. Shared tools, fuel and writing supplies are counted once when proposing
work. Execution cannot replace a missing teacher or process newly arriving samples
beyond the planned quantity. It still checks current routes, stocks and GPU labor.
A settled grant cannot be spent again, even if supplies appear later that month.
Unfinished planned work expires; next month's preparation can request it again.

## Accounting and inspection

`History::service_work_report()` exposes the latest cultural request bundle and
receipt, per-workshop research plans and receipts, paid crew work, and enterprise
funded/completed/idle work. Research and culture receipts distinguish requested,
granted, used and released worker-months. Culture's receipt is aggregate across
sites; cancellation reasons remain on site bundles. Enterprise idle employment and
crew standby remain paid work, not fictional completed manufacturing output.

Receipts and travel clocks are archived with defaulted fields for older saves.
History validation rejects nonfinite clocks, future plan dates, invalid identifiers
and work exceeding grants. No cancelled labor is backdated into the GPU production
pass. Public advance/checkpoint semantics remain completed monthly boundaries;
this change does not provide rollback after a GPU failure.

## Verification

Verified on the Quadro RTX 5000 with Max-Q Design, Vulkan, using Rust 1.89.0.
Commands use `CARGO_INCREMENTAL=0 mise exec rust@1.89.0 -- cargo` as the prefix.

| Check | Result |
| --- | --- |
| `test --lib` | 73 passed; 68 hardware tests ignored by this command |
| `test --lib FILTER -- --ignored --nocapture` for `culture::`, `discoveries::exchange_tests`, `vessels::tests`, `enterprises::tests`, `institution_capacity::tests`, `civic_petitions::`, `expedition_heritage::tests`, `hazards::tests` | 27 hardware fixtures passed |
| `test --test history_environment --test markets -- --include-ignored` | All 3 history and 8 market tests passed; 28.69 s and 7.93 s excluding compilation |
| `clippy --all-targets -- -D warnings`, `fmt --check` | Passed |
| `python3 scripts/check_repository_artifacts.py` | Passed; generated outputs remain ignored |

The enterprise fixture includes 144 reservation-scarcity cases over seeds 17, 81
and 256, varying adults, illness, culture and discretionary ordering. It checks
shared labor and money; it is not a long-run demographic experiment. The history
fixtures retain exact full-state comparisons across compact/full environmental
observations, batching and checkpoint continuation for those seeds.

New boundary assertions cover no crews, fractional crews, repeated opening,
December-to-January travel, restored travel clocks, actor death and household
relocation, stale cultural grants, lost crafting supplies, two specimen kinds
sharing a single tool stock, closed teacher contact, and no reuse of settled
research grants when materials return. The ordinary suite also retains the
annual-policy next-month activation/replay fixture. A receipt mutation that spends
more than its grant must fail validation.

Small fixtures deliberately isolate timing and resource accounting. These checks
do not establish long-run balance, arbitrary mid-month recovery, or cross-GPU
identity. The conservative cultural cancellation rule can defer feasible work;
its measured longer-term frequency is reported in
[cultural work calibration](cultural-work-calibration.md).
