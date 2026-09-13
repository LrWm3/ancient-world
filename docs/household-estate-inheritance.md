# Local household estate inheritance

Implementation pilot; long-run balance evaluation pending. This addresses a
specific continuity gap: a living descendant can already head another household,
while the former household retains its wallet and ownership indefinitely.

`HouseholdEconomy.inheritance.enabled` is an opt-in local succession policy. At the
Reserve retail boundary it considers households vacant for at least twelve months,
with deceased heads and no known living members. It excludes traveling/lost
households, abandoned sites and estates with live debt or unrecovered defaults.
It traverses recorded descent, stopping at living descendants; multiple living
branches remain unresolved. A sole descendant must be resident in a non-vacant
household at the same settlement. There is no distant cash teleportation.

The receiving household acquires cash above a protected food reserve and the
estate's town ownership share. Identities, historical heads and genealogy remain intact. Zero-property
households become valid; town ownership still sums to one. `inheritance_paid` and
`inheritance_received` distinguish these transfers from wages, profits and relief.
Dated receipts and events retain the source, beneficiary, heir, cash and share.
The policy runs once per monthly retail boundary. Future receipts reaching an old
estate can pass through a later eligible succession boundary; repeated invocation
within a month cannot spend them again.

The sole-recorded-descendant rule is an explicit toy inheritance convention, not
proof of exhaustive genealogical knowledge. In aggregate mode anonymous population
is not removed or reassigned. This policy changes economic ownership only. It does
not seize every vacant wallet, merge household rosters, grant a political office,
or resolve spouses' and distant relatives' competing rights. Unclaimed estates,
physical recovery from abandoned settlements, and economic calibration remain
outstanding.

## Controls and verification

Use `--household-estate-inheritance[=true|false]`; omission preserves the archived
policy. The monetary runner accepts `--household-estate-inheritance` and applies it
to all arms, explicitly disabling it otherwise. Existing histories default to off.

The pure descent fixture passes (living branches, nearest living ancestor and a
cycle guard). The GPU-backed transaction fixture passes: conserved cash and local
shares, protection of a living owner, no duplicate transfer on repeated invocation,
serialization/next-month equality, and rejection of a corrupted receipt. This is
not yet a full multi-month natural-history checkpoint comparison. All three cash
audit tests and twelve monetary report tests pass. The CLI override test, strict all-target Clippy and native build pass. The first matched
seed-1024 four-arm 50-year screen completed; outputs stay under ignored
`output/household-inheritance-screen/on/`. See the correction and results below.

## First screen and subsistence correction

The first four-arm screen completed before the food-reserve safeguard below.
All arms exited successfully. Without issuance, the policy transferred 1,539.70
in 146 receipts; with issuance, 2,093.87 in 78 receipts. The no-issuance endpoint
had population 159.435 and hunger 0.05832, versus 156.688 and 0.06939 without
inheritance. Operator operating margin worsened slightly (-275.98 versus -269.35).
With issuance, population was 157.960 and hunger 0.06391. Those endpoint differences
do not establish the mechanism for each downstream change.

Inspecting individual receipts revealed repeated small transfers from estates that
still had aggregate food need. In one case, 126 later transfers amounted to 18.82.
Absence of named members does not establish absence of anonymous dependents. The
revised policy therefore retains three months of the last completed household food
need, valued at the current local food quote, before transferring cash. It protects
full food cost rather than assuming common entitlement will be funded next month.
Ownership share can still pass to the heir. Receipts record protected cash separately
from transferred cash. This is an explicit conservative game reserve, not a solved
household population model; changing need and local prices affect future transfers.

The first screen is superseded for balance claims by this correction. The reserve-specific GPU fixture passed (0.94 seconds), as did strict all-target
Clippy. The corrected native build and revised screen are pending. Before the correction the full CPU
library suite passed: 184 passed, 143 hardware tests ignored. No portability or
performance conclusion is drawn from these runs.
