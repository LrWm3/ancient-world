# Founding, religious and literary expeditions

Three new charter objectives use the existing expedition interface and transport:

| Objective | Motivation | Possible modest find |
|---|---|---|
| PatronSearch | Seek traces of the named, departed patron associated with the port's local tradition | Worn ceramic charm with an ambiguous animal-like outline |
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

A deterministic seed/cell rule makes roughly three fifths of surveyed endpoints
eligible for one accessible 0.125 kg ceramic fragment. This is a provisional fictional
archaeological cache, not evidence produced by a simulated ancient civilization.
The first month of fieldwork can claim it. One find per endpoint across all voyages
and objectives prevents farming repeat imports. A lost expedition does not reset the
cache. There is no navigable ruin or ancient creature population behind this proxy.

Discovery is recorded in the expedition's external field custody. Only successful
return imports the fragment into the managed settlement economy. Its material is
resolved by the stable catalog ID `pottery`; the import is declared in the goods and
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
region. Multi-seed frequency calibration and richer catalog-driven fragments remain
follow-up work.
