# Naming languages

Each civilization now has a naming-only fictional language. This is a game name
maker inspired by ordered sound changes, not a reconstruction of Latin or a
model of language acquisition. It does not affect production, trade, religion,
politics, cultural affiliation or migration.

## How it works

`src/naming.rs` contains a shared 200-root invented proto-vocabulary, loosely inspired
by classical word shapes. Each civilization's seed chooses consistent rules for
palatalization, intervocalic lenition, initial f, final m/s loss, kt/ai/oi changes,
vowels and preferred compound form. Rules apply in a fixed order. Vowel shifts
are simultaneous; outputs are not repeatedly shifted. Proper names used in a
dedication are not passed through the sound changes again.

Names can be separate words, full compounds, blends, clipped word pairs, or words
joined by a language-specific linking particle. Longer compounds become separate
words. A civilization favors one form, with seeded exceptions; order may reverse
or rotate. Referenced proper names contribute a selected word (up to 12 characters),
not always the first word. These fragments are not sound-shifted twice.

Personal naming chooses among four weighted conventions: 30% aspiration, 20%
nature pairing, 20% local livelihood, and 30% commemoration. Unavailable contextual
conventions fall back to aspiration. Local production above 1 kg makes associated
materials and occupations eligible; this describes a naming association, not the
person's own profession. Commemoration chooses among supplied references with
weights: each parent/predecessor 6, home town 3, local tradition 3, its patron 2,
and each of up to four active local institutions 1. Initial founders without these
records use unreferenced conventions. There is no scan of distant or future names.

The 200 roots include seasons, weather, plants, animals, landforms, materials,
occupations, virtues, rituals and social purposes. Entity naming heads also vary:
for example home/hearth/refuge, league/people/fellowship, or memory/song/oath.
Those are naming conventions, not additional legal statuses or historical events.
The vocabulary remains bounded, not a grammar capable of arbitrary translation.

Language rules, vocabulary and coined-name records are archived. Each record
stores the meanings, form and optional source kind/ID/name. Thus an opaque-looking
name can still be explained. The source is a naming association, not an extra
historical fact or an assertion that an artifact's supposed origin is true.

## Where names come from

| Entity | Inputs |
|---|---|
| Civilization | Founder's recorded personal name + league |
| Town | Initial survey landmark + home; daughter towns also reference the civilization's earliest site |
| Person | Weighted aspiration, nature, actual local production, town, tradition, patron, institution, parent or predecessor associations |
| Household | Founding household head + house |
| Patron | Human language's guide/journey name, with the existing archetype epithet |
| Founding tradition | Patron + memory |
| Schism | Actual reformer + covenant |
| Religious institution | Affiliated tradition + sanctuary |
| Merchant or scholarly institution | Actual founding actor + market or learning |
| Craft institution | Founder + craft, optionally the most-produced local timber, metal or ceramic material |
| Keepsake | Patron + gift/memory |
| Crafted artifact/manuscript | Creator + gift/book |
| Recovered ancient ceramic | Actual recovering expedition + clay/memory |
| Expedition crew | The originating civilization's personal-name conventions, in a separate identity namespace |

Town survey labels prefer an adjacent great-lake shore, otherwise river discharge
above 1 m³/s, elevation above 1,000 m, or prospective fields. These thresholds
are naming conventions. They do not change settlement suitability. The initial
survey is preserved; later environmental changes do not rename the town. Missing
old survey information falls back to an island association. Craft material
associations require actual cumulative local output above 1 kg, not knowledge of
hidden ores. The geological generic metal pool is called metal, not presumed iron.

Names are unique among newly registered entities of the same category in a
language. Collisions try descriptive bynames, then untranslated phonotactic family bynames.
Entity IDs seed choices but are never appended as digits. The extra meanings are
recorded; untranslated bynames are marked as such. Similar names
across languages are possible. An unarchived lookup index makes collision checks
logarithmic and rebuilds after loading. Randomness is keyed to civilization and
entity identity and does not consume the history simulation's random streams.

## Explorer and compatibility

In the history overview, expand a civilization's naming-language panel and hover
a name to see its gloss, form and reference. Existing entity labels elsewhere
show the generated name as usual. Public serialized language/name records are
also accessible through the history API.

Existing saves keep their names. New coinages extend older archived vocabularies
with missing roots, preserving existing root spellings. Newly generated children
use the same weighted conventions instead of always appending an English household
suffix; actual household and genealogy links are unchanged. An old civilization without a language receives
one when it next needs a new generated name; existing labels do not acquire
fabricated etymologies. There is no general renaming system, grammatical inflection, or language-based
political identity. [Lexical evolution and contact borrowing](lexicon-evolution.md)
now add bounded alternatives to concepts used by future names.
Patron names are human naming conventions, not a newly simulated ancient language.
Catalog species, goods, mineral names and faction category labels remain as before.
Some descriptive labels, including specimen descriptions and building labels,
still use their existing templates.

## Verification

Small tests check hand-calculated ordered sound changes, simultaneous vowel
shifts, cross-civilization variation, 10,000 contextual personal-name registrations,
3,000 deliberately identical base names without numeric suffixes,
reference/gloss retention, invalid profiles, old-record imports and exact naming
continuation after serialization.

The hardware fixture generates seeds 17, 81 and 256 at terrain 32/ecology 16,
with one geological epoch, five civilizations and ten years of social history.
It checks survey-to-town name inputs, patron references, valid profiles and
existing economy residuals. Seed 17 additionally compares a 12-month batch with
twelve one-month steps from a saved world, including the full naming ledger.

Reproduce with:

```sh
mise exec rust@1.89.0 -- cargo test --lib naming -- --include-ignored --nocapture
```

Generated archives are temporary. The vocabulary and sound rules are fictional content, not evidence of realistic
historical linguistics.

First pass on the Quadro RTX 5000 Vulkan backend: all three ten-year naming runs
passed (five language records per seed), including full checkpoint/batch equality
for seed 17. Observed names included `Lidurdumu` (shore + home),
`Melo Selwa Teeto` (founder-associated timber craft), and `Dokes Sakra`
(tradition-associated sanctuary). The panel retains the full source identity even
when the displayed name uses only its first word or a blended fragment.

The existing 30–31-year material fixture also reproduced the pre-naming numeric
summaries exactly for seeds 17, 81 and 256: populations 758/696/765 and facility
expansion counts 18/7/9, with identical printed production and ledger residuals.
That is evidence for the intended naming-only boundary in these runs, not a proof
covering every future history path.

Verification: 44 ordinary library tests passed; all four naming tests passed when
the GPU fixture was explicitly enabled; six ordinary market integration tests
passed (two hardware market fixtures remain ignored). Clippy across all targets,
formatting and the source-artifact check passed.


## Expanded naming evaluation

The controlled 10,000-person sample (seed 256, civilization 1, with all five source
kinds supplied) selected 2,933 aspirational, 1,977 nature, 2,019 livelihood and
3,071 commemorative names. All five forms and all five source kinds appeared;
labels were unique within the category, contained no digits, and stayed below
100 bytes. An additional 3,000 identical-base test exercised descriptive and
untranslated collision bynames. Serialization preserved the next 100 choices.

Fixture source labels are illustrative; ordinary histories supply their recorded
names. Parent-name fragments, for example, can survive in a child's commemorative
name alongside newly selected meanings. Linking particles are excluded from source
fragments, preventing names built solely from an inherited “of.” These checks establish variety and provenance in a
bounded sample, not unlimited unique semantics or linguistic realism.

Expanded-pool examples include `Serenatsedara` (calm + cedar), `Saligatexira`
(willow + weaver), and `Morintaremhigara` (steadfast + fig, with Morin from the
supplied parent name). The names are still stylized and may repeat across languages;
there is no claim that each root combination expresses a unique human meaning.

Final checks: 49 ordinary library tests and all six naming tests passed, including
three GPU history seeds, source-ID resolution, and seed 17 checkpoint continuation.
The enlarged emblem pool required 3,000 forced collisions to reliably exercise the
untranslated fallback; the earlier 1,500-case coverage assertion was insufficient.
Clippy across all targets, formatting and the repository artifact check passed.

## War names

New territorial wars receive a persistent name in the attacker's naming language
at declaration. Six equally weighted templates favor the contested settlement
(four templates), with the launching settlement or attacking leader as alternatives.
The templates combine the source name with war, claim, campaign or frontier roots.
They describe the existing territorial campaign mechanic, not invented holy-war,
resource-war or siege causes. The name is one attacker's commemorative convention;
separate opposing names and later historical nicknames are not modeled.

The language's name record preserves meanings, source and lexical choices under
`war:<id>`. The war stores the resulting name, so later vocabulary changes, conquest
or leader death do not rename it. Declaration and peace prose and explorer war/army
lists use that label. Older archives without a name keep their `War N` fallback;
no historical name is fabricated during loading. Naming does not alter war decisions
or resource transfers.

Validation: a 64-seed fixture checks varied names, recorded context, no numeric
suffixes and stability after vocabulary/source-label changes. All 12 naming tests
passed, including the existing GPU histories. The GPU territorial-campaign test
checks declaration/peace labels, legacy fallback, full checkpoint continuation and
unchanged population/food/economic conservation assertions. All 53 ordinary library
tests passed (52 hardware tests ignored); Clippy with warnings denied, formatting
and the repository artifact check passed.
