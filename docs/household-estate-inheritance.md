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

The receiving household acquires the estate's actual cash and town ownership
share. Identities, historical heads and genealogy remain intact. Zero-property
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
audit tests and twelve monetary report tests pass. The CLI override test, strict all-target Clippy and native build pass. The matched
seed-1024 four-arm 50-year screen is running; outputs stay under ignored
`output/household-inheritance-screen/on/`. Results are pending.
