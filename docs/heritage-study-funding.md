# Financed heritage interpretation

`Culture.funded_heritage_study` optionally makes scholarly/religious institutions
pay for the writing supplies used to interpret returned heritage. Defaults and
older archives keep the existing publicly supplied path. The balance runner
exposes `--funded-heritage-study` and records the setting in report metadata.

This is the first financing connection for heritage stewardship, not a complete
curatorial workforce, donation, visitor or artifact-preservation model.

## Boundary and accounting

Opening requests and captured service-provider selection require an institution
that can afford the existing 0.05 kg writing-supply batch. This is a conditional
quote, not escrow: multiple studies can compete for the same treasury. Actual
execution rechecks the selected provider's live funds, supplies, artifact access,
work and room grant. It does not switch to a different provider after reservation.
An unaffordable execution consumes neither room nor materials.

The institution buys from existing town stock at the current writing-supply price.
The existing bounded cash-deposit helper computes the representable town credit;
the exact same amount leaves institutional treasury and enters its expense ledger.
Zero representable payment blocks financed execution. No second labor payment is
added; existing work funding and personal commitments remain authoritative.

Material withdrawal never exceeds the nominal batch and blocks when no positive
batch is representable at the inventory scale. Consumption uses the actual f32
inventory decrement for used materials,
waste and C/N/P returns, including on the legacy public-supply path. The financing
receipt records institution, stable good ID, actual kilograms and paid cash alongside
the dated interpretation. Old study records deserialize without an invoice; they
are not retroactively charged. Existing five-year study intervals and three-reading
limit remain in force, preventing a repeated call from charging again.

Prices are abstract game signals. Rounded credited cash can be slightly below the
unrounded quote, but cannot exceed either the quote or the institution's wallet.
This does not create a private writing-material inventory or change artifact
ownership. The physical find and earlier interpretations remain intact.

## Verification

The five controlled GPU finance/access cases and bounded-withdrawal unit fixture
pass. Funded studies use the same finite room/work allocation as an ordinary
lesson in both damaged and maintained rooms. Four funded readings consume at
most 0.2 kg of existing writing stock; limited room space still suppresses the
competing lesson. Withdrawing the payer's treasury after reservation blocks all
readings without consuming room, labor or supplies. Serialized continuation
reproduces the financed results, and repeated study does not charge again.
Money credited to the town equals treasury debits exactly in the controlled cases.
Legacy study records without funding deserialize without invoices.

The full frozen scheduler comparison passes for seeds 17, 81 and 256, including
monthly versus batched advancement and checkpoint continuation. Seeds 17 and 256
enable financing; seed 81 retains the public-supply path. This checks timing and
persistence, while the controlled fixtures above guarantee actual study execution.
The ordinary library suite passes 130 tests (113 hardware fixtures ignored); the
two focused heritage tests, including the GPU fixture, were run explicitly.
All-target Clippy passes with warnings denied.

Ordinary recent thirty-year institution ensembles produced no heritage studies,
so those runs cannot validate
heritage financing. Guaranteed-return fixtures are needed to exercise this path;
a quiet random seed must not be presented as evidence of balance.

Expedition inspection displays each invoice's date, institutional payer, payment
and supply mass. Balance reports expose `heritage_study_funding` totals for study
count, money paid and writing kilograms, separately from institutional upkeep.
No visual interaction test is claimed for this added text display.
