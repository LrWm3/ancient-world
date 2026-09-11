# Named merchant crew service

With individual participation enabled, merchant fleets now hire known residents
for their existing monthly port-service allowance. Disabling participation retains
aggregate household staffing. No new population baseline or GPU readback is needed.

## Monthly boundary

1. Open advances existing cargo using the previous boundary's completed prepaid
   fleet capacity. New hiring cannot retroactively move that cargo.
2. Reserve clears prior assignments after that observation, reserves domestic care,
   and hires for cargo already committed. Research/culture and enterprises then
   reserve their work; standby crew hiring uses what remains.
3. Execute/settle rechecks the assigned people after demographic outcomes. Surviving,
   present workers complete their grant. Missing people contribute zero. Their
   prepaid wages remain household income; unproductive time earns no experience.
4. Respond uses settled capacity for new dispatches and releases the site labor
   allowance. Personal time remains committed through the month. The completed
   capacity persists for the next Open.

The original aggregate service ceiling, hull backing, affordability, flood and
cargo-load constraints remain. Named participation adds a constraint; it does not
add workers. Residents without a present ownership household cannot receive this
employment. Hired people share the same bounded time ledger as research, cultural
work and refined workshops. Ordinary aggregate production still uses its existing
workforce accounting; complete assignment of all productive work remains unfinished.

## Hiring and income

Available local residents are ranked by completed merchant worker-months, with
person ID breaking ties. Experience affects hiring priority, not work output or
navigation speed. Several residents can cover one hull's allowance. Committed
cargo hiring and later standby top-ups cannot reuse previously reserved time.

Each payment debits town cash and credits the actual worker's household cash,
wages and employer income. Rounding cannot purchase more than the reserved work.
There is no new wage source, flat crew payment or duplicate household subsidy.

Per-vessel receipts retain person, household, month, commitment, wages, granted,
completed and released work. Personal records retain cumulative merchant service.
The port inspector lists participants and their completed/granted work. Checkpoints
serialize both receipts and prepaid capacity. Older vessels default to empty crew
records and keep their existing prepaid interval; subsequent hiring follows the
selected participation mode. Disabling participation at a settled boundary detaches
expired commitment references without deleting the paid interval.

## Scope

These are named participants supplying pooled maritime service at each endpoint.
They are not yet aboard physically located ships or absent from their home census.
Sailing itineraries, onboard provisioning, maritime casualties and experience-based
technical improvements remain separate possible extensions. The 0.25 worker-month
per hull is the existing game-scale capacity abstraction, not a literal ship crew.

## Verification

Run `cargo test --lib vessels -- --include-ignored --test-threads=1` for named and
aggregate fixtures. Named checks use seeds 17, 81 and 256 with declared port assets
and cash to isolate participation from investment balance. They cover actual payees,
shared commitments, aggregate control, loss after reservation, one-time experience,
receipt validation and save/load continuation. Existing transport tests cover prior
funding, fractional progress, missed intervals and opening idempotence.

### Results (2026-09-11)

- Five vessel tests pass, including the named fixture on seeds 17, 81 and 256.
  Each seed also matches complete serialized history after a 15-month batch versus
  a saved/reloaded sequence of single-month steps, crossing an annual boundary.
- Busy residents cannot crew ships; the matched aggregate control can still fund
  its allowance. No cash prevents hiring. Experienced eligible workers receive
  hiring priority. Death after reservation removes completed work and experience
  while retaining the prepaid household wage. Repeated settlement adds nothing.
- The shipping integration test passes with actual intercontinental cargo, endpoint
  reservations, lane closures and checkpoint continuation. All four discovery
  integration tests also pass with the new staffing path.
- The regular library suite passes 104 tests (93 hardware-dependent tests skipped
  in that invocation). All-target Clippy with denied warnings and the repository
  artifact check pass. The complete GPU suite and long-run balance were not rerun.

The duplicate-call fixture exposed an unpayable cash-rounding remainder that could
reserve time without paying wages. Hiring now probes the actual f32 debit before
reserving that time. The controlled checkpoint fixture records supplied harbor
materials and capital in its opening ledgers; it does not waive budget validation.
These checks demonstrate bounded work, financial transfers and timing, not a claim
that the current crew sizes or labor priorities are well balanced in every world.

## Aggregate/individual resolution comparisons

Fleet projections now retain the opening demand, remaining pooled service labor,
affordable worker-months and already funded work separately at the committed-cargo
and standby reservation windows. Their minimum is a **pooled budget ceiling**,
not a promise that eligible households or named workers exist. It is not a full
parallel aggregate history. In particular, the later window sees actual remaining
budgets after earlier crew hiring and intervening services.

With resolution comparisons enabled, one `MerchantCrew` receipt per port records
conditional expected and actual hiring for each window, then granted versus
completed work. Hiring differences remain unexplained in the common metric;
the model does not infer a particular cause from a single missing match. Released
prepaid work is directly measured at settlement. Neither forecast pays wages,
reserves personal time, moves cargo nor supplies capacity. Aggregate staffing
receives the same budget comparison while preserving its existing behavior.

Current fleet projections are archived and reset alongside assignments at Reserve.
Old archives default to no projection for the already prepaid interval. Completed
receipts are bounded to the current month; cumulative comparisons retain numeric
summaries. Repeated settlement does not add observations or experience. Endpoint
service remains the model: these receipts do not imply onboard ship populations.

Verification for the comparison increment: six vessel tests pass, including the
three-seed named-worker fixture, unavailable-worker aggregate control, departure
from a prepaid grant, idempotence, and 15-month save/load versus batch continuation.
The analytical fixture checks pooled limits, unexplained hiring differences,
measured release, stale dates, invalid grants and serialized forecasts. The regular
library suite passes 112 tests (99 hardware tests skipped); the hardware shipping
integration fixture also passes. All-target Clippy with warnings denied passes.
These checks establish accounting and timing, not balanced crew labor shares.
