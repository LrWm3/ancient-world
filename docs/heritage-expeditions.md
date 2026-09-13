# Founding, religious and literary expeditions

See [the latest memory, fleet and research extension](memory-fleets-and-research.md) for subsequent changes.

Three new charter objectives use the existing expedition interface and transport:

| Objective | Motivation | Possible modest find |
|---|---|---|
| PatronSearch | Seek traces of the named, departed patron associated with the port's local tradition | One of three finds associated with the charter patron’s archetype |
| Inscriptions | Compare old marks with inherited founding accounts | Broken plaque with repeated signs or tally-like strokes |
| OldLiterature | Preserve old verse, domestic records or devotional writing | Fired-clay text fragment with a provisional household-list or refrain reading |

Use **Seek patron**, **Study inscriptions**, or **Seek old texts** in the expedition
panel, or pass the corresponding `Objective` to `Generator::launch_expedition`.
The charter preserves its originating tradition, patron reference, and motive even
if local affiliation later changes. Patron searches require an actual recorded patron
who has departed; legacy cultural baselines cannot invent one.

## Funding and consequences

The same eight-adult crew, finite food, timber, tools, 600-money escrow, civilian
reserves, harbor requirements, cooldown, hazards and rescue rules apply. Religious
and scholarly institutions can sponsor these objectives if operational and holding
at least 1,200 money. Religious orders are preferred for patron searches; scholarly
circles for inscriptions/literature. Merchant, public and existing private funding
remain available. Unspent escrow returns through the existing payer-specific path.

Automatic selection retains health and resource-demand priorities. Otherwise its
six-way rotation includes these three motives alongside charts, geology and ecology.
Ordinary field observations still generate bounded expedition knowledge on return.
These voyages do not automatically collect resin or geological samples; those remain
specific to ecological/geological objectives.

## Finds, custody and uncertainty

A deterministic seed/cell rule provides baseline opportunities at roughly three
fifths of surveyed endpoints; explorer experience can open additional opportunities.
Ordinary objectives recover one 0.125 kg minor ceramic, textile, weight or tool.
Patron searches instead choose one of three archetype-specific objects, weighing
0.02–1.2 kg. These are fictional archaeological caches. The first month of fieldwork
can claim one find per endpoint across all voyages and objectives. A lost expedition
does not reset the cache.

Discovery is recorded in the expedition's external field custody. Only successful
return imports the fragment into the managed settlement economy. Its material is
resolved by the corresponding stable catalog ID (`wood`, `pottery`, `cloth`, `metal`, `leather` or `tools`); the import is declared in the goods and
C/N/P exchange ledgers, including custom catalog composition. Failed voyages import
nothing. The voyage record retains the claimed source and its outcome.

Delivery creates an ordinary unique artifact, owned by the sponsoring institution or
origin community, with observation and receipt provenance. It therefore participates
in existing custody, dedication, ownership, destruction and ruin systems. It grants
no technology, magical effect, permanent fertility, or additional cash. A living
tradition leader may author an explicitly uncertain interpretation citing the facts.
The original maker, language, date and meaning remain unresolved.

No search physically reunites people with a patron, proves who entrusted the patron,
or uncovers a major historical revelation. Texts remain fragmentary. These outcomes
add local religious/anthropological significance, not an answer to the setting.

Charters and finds serialize with expeditions. Old voyages default to no heritage
charter; historical discoveries are not retroactively fabricated. Validation checks
references and rejects duplicate source recoveries.

## Verification and limits

The hardware fixture `heritage_voyages_preserve_minor_finds_and_checkpoint_continuity`
launches a funded patron voyage, verifies a finite returned artifact with no knowledge
topic, checks an attributed account and economic residuals, and compares batched
advancement against monthly checkpoint continuation. Existing expedition tests cover
escrow, rescue, hazards, recall and failed transactions. Cultural tests exercise
artifact and account persistence and ownership behavior.

```sh
mise exec rust@1.89.0 -- cargo test --test expeditions -- --ignored --test-threads=1
mise exec rust@1.89.0 -- cargo test --test culture -- --ignored --test-threads=1
```

The cache occurrence rate and automatic motive rotation are game settings in code,
not calibrated archaeological claims. Patron searches currently use the selected
frontier route; they do not autonomously navigate to the patron's recorded departure
region. Multi-seed frequency calibration remains follow-up work.

## Patron-specific objects

New patron searches select from the charter's preserved patron archetype, using
stable archetype IDs. Changing the port's faith later does not change its charter.
The seed and surveyed cell select among the three options; no extra random stream
changes other expedition decisions.

| Patron | Possible finds |
|---|---|
| Human guide | Strange wooden staff; flute of an unknown material; fabric resembling the guide's clothing |
| Reed-crowned | Braided reed crown; many-legged track tablet; reed signaling whistle |
| Glassback | Translucent plate fragment; mosaic fragment; polished lake lens |
| Lantern elk | Antler-shaped pendant; antler-handled lamp; forest procession bark strip |
| Basalt tortoise | Shell-pattern rubbing; relief tablet; shell-shaped survey weight |
| Storm heron | Flight-feather remnant; heron-shaped wind vane; storm-call pipe |
| Root marten | Root-woven seed pouch; layered-fur grooming comb; burrow-marked tablet |
| Silver manta | River ribbon; manta-shaped bowl; river-pattern chime |
| Ash bear | Claw-shaped digging handle; paw impression; ash-stained carrying wrap |
| Many hands | Many-gripped tool; miniature glove remnants; eight-strand instruction braid |
| Salt listener | Listening cup; dialogue tokens; interpreter mantle scrap |
| Deep weaver | Loom shuttle; deep-fiber veil; woven hand-sign sampler |
| Dawn keeper | Throat-fan ornament; dawn-call double pipe; feather-patterned sash |

The 39 definitions live in `src/expedition_heritage/patron_finds.rs`. Each has a
stable saved type, description, material proxy and mass. Ownership remains uncertain:
a resemblance may be an imitation or coincidence. Existing artifact custody,
interpretation, study and renown mechanisms handle the returned object.

Unknown substances use the existing `tools` accounting category, and the feather
remnant uses `leather` as an organic material proxy. Those ledger categories do not
identify the object's substance. Imports use catalog C/N/P composition and the
object's actual configured mass, exactly once on successful return. No additional
material or biological functionality is implied by these descriptions.

Old finds lacking a patron item retain their original category and 0.125 kg mass;
custom patron archetypes without definitions use the generic fallback. Saved item
IDs preserve the selected type rather than selecting again when loading.

### Patron-variety verification (2026-09-12)

- All 39 definitions pass material-ID, unique-ID, positive-mass and serialization
  checks; legacy generic finds retain their original interpretation and mass.
- Four targeted heritage library tests pass, including the GPU-backed fixture
  exercising survey and delivery for all 13 patron archetypes, with exact chosen
  material/mass and no repeated-delivery import.
- The real funded voyage/checkpoint test passes: batch and monthly resumed history
  match, the patron-specific object survives return, and economic residuals remain
  below the fixture's `1e-3` tolerance.
- Ordinary library suite: 148 passed, 130 hardware/long-running tests ignored.
  This is targeted verification, not a new multi-seed frequency calibration.
