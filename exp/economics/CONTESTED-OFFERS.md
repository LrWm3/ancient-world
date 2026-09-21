# Contested offers and allocation policies

Status: implemented pilot, not a general auction or ZIP pricing model.

An agent's search policy chooses what to request. A separate allocation policy
chooses which requests can reserve an oversubscribed pool. Timing stays unchanged:
Open establishes availability, Acquire collects and clears conditional applications,
and Productive executes the dated work plan. Due and ClearArrears retain annual
payment timing. Losing an application is not a breached agreement.

## Generic policy boundary

`allocation::resolve` takes a pool/round/seed context, capacity, stable-ID claims,
a `RankingPolicy`, and an atomic reservation callback. Claims carry requested
quantity, minimum useful quantity and externally computed priority. The allocator
has no knowledge of people, farming, land, prices or which side posted the offer.
It orders claims, caps grants by demand and remaining supply, skips unusable partial
grants, and releases capacity when reservation fails so the next claimant can try.
Receipts distinguish requested quantity, offered grant and actual reservation or
rejection. Callbacks must leave their own state unchanged on failure.

Three built-in policies are available:

- `PriorityLottery`: lower priority number first, seeded lottery within that class.
- `Lottery`: seeded lottery without priority classes.
- `StablePriority`: priority followed by stable application ID, useful as a control.

Other policies implement `RankingPolicy::key`. Stable IDs break any remaining ties;
input vector order and mutable random-number state do not choose winners. The seed,
pool and dated round are explicit. This can later rank competing buyers for a scarce
ask, or competing sellers against a limited bid budget. Those adapters must supply
units, eligibility, funding, minimum lots and settlement rules. This does **not** yet
clear two-sided order books, choose transaction prices or implement ZIP.

## Multiple offers, one round

The state posts an open offer ID for each available plot. Its catalog debtor/right holder fields use
the issuer as an unbound placeholder; the open-offer marker means those fields grant
nobody use. Acceptance binds the actual debtor, right holder and crop beneficiary
in state. Both people discover the same ID and terms, rather than receiving two
private promises to the same plot.

Each person independently searches a local projection of the same opening state.
A proposed bundle can contain citizenship, land and farming/wood work. Its ordinary
forecast checks future labor and rent before submitting the application. The land
adapter prepares each submitted bundle without mutation, gives people without
current tenure first priority, then runs the generic allocator over the offered indivisible
slots. Repeated applications by one person are alternatives; at most one wins. Explicit API applications receive current bundle feasibility checks; callers
bypassing agent search do not automatically receive a sustainability forecast.

Only awarded bundles create agreements. Seed and labor are reserved for
Productive, not spent at Acquire. Unsuccessful people replan with the award visible;
they can collect wood but cannot farm the winner's plot. Existing work and the awarded
bundle precede new fallback work in the shared resource resolver. The receipt,
applications, seed and policy are attached to the committed batch; settlement
recomputes the proposed round and rejects inconsistent results atomically. Existing
tenure is never redrawn by the lottery.

`World.competition` opts into the automatic coordinator. All open postings clear
at the same Acquire boundary. Each person evaluates each alternative against the
same opening state. Alternatives are currently unranked: stable offer IDs order
the search, and policy priority ranks admission rather than preferred plot ownership.
The lowest posting ID identifies the pool for the seeded lottery.

`allocation::assign` supports generic unit claims with acceptable slot IDs. It
uses the existing ranking policy and can reroute a tentative assignment along an
alternating path: a person accepting plots A or B can move to B so a person accepting
only A can also win. No earlier-ranked winner loses admission during this matching.
For independently feasible slots this avoids leaving usable capacity unassigned.
It is not an optimization of utility or prices.

Before publishing, the domain adapter checks the combined bundles in policy order,
including joint labor, seed, physical parcel exclusivity and shared inputs. A failed
applicant/offer edge is removed and matching retries; each edge can fail only once.
This is conservative joint-budget validation, not an exhaustive optimizer over
all resource-constrained combinations. One atomic batch accepts all memberships
and land agreements and reserves one combined productive plan. Continuing processes
are included once, and unsuccessful agents then plan fallback work. The round
records final assignments and alternative rejection reasons as well as grants.

Search forecasts are conditional
on acquiring the requested opportunity; they do not predict rivals or winning odds.
Subsequent ordinary productive planning still uses the existing bounded planner;
this change does not establish fully independent decisions for every monthly action.
Shared wood fallback ordering is still explicit winner/existing-work then stable
agent order, not a newly fair allocation market. No relief, transfer of tenure,
subletting, food trade or automatic reassignment after death was added.

## CPU controls

Run from `exp/economics`:

```sh
cargo +1.92.0 run --locked -- opportunity-two-plots
cargo +1.92.0 run --locked -- opportunity-one-plot
cargo +1.92.0 test --locked --test allocation --test competition
```

Both CLI controls run 36 months. They share two people, starting stocks, crop terms,
two-grain annual rent, three monthly labor units each, and finite wood (24 initial
units, regeneration 2/month). Only plot count differs. The default lottery seed is 7.

In the 18-month CPU/reference checks both people acquire land in month 1 when
two plots exist, with no nutrition or warmth shortfalls. With one plot, seed 7
still awards person 89 the plot and person 88 dies from unmet food needs in month 11.
This is an exclusion/consequence control, not a sustainable two-person economy.

The 36-month CPU runs show:

| Plots | Admission | Completed harvests | Unmet nutrition | Unmet warmth | Crop aborts |
| --- | --- | --- | --- | --- | --- |
| 2 | Both in month 1 | 10 (5 each) | 0 | 0 | 0 |
| 1 | One in month 1 | 5 | 6 | 0 | 0 |

Clearing together removes the previous one-month admission delay and its one-unit
food deficit. Crop, tax, starting-stock and wood parameters are unchanged. The
one-plot case retains its previous outcome. Neither control establishes economic
balance beyond these settings.

Tests cover demand caps, minimum useful grants, failed-reservation fallback,
policy replacement, seed variation, reversed application/participant ordering,
infeasible applicants, tenure priority, same-month warmth replanning, no losing
seed/tax commitment, tampered receipt rejection, CPU/reference agreement and
checkpoint/monthly continuation. New controls cover alternative reassignment,
at-most-one awards, atomic multiple acceptances, joint input shortage and both
people starting in month 1. Generated logs remain under ignored
`output/economics/`.

Validation for simultaneous clearing uses the allocation, competition, common-offer,
membership, access, agreement, condition and settlement controls, plus Clippy with
warnings denied, formatting and the repository artifact check. The earlier
single-posting baseline passed the full 173-test suite.
