# Evolving naming vocabularies

A civilization's naming vocabulary can acquire eponyms: a locally associated person,
town, institution, patron or object supplies a new word for a concept. Other
civilizations can borrow these words through route contact. This only changes
future name generation, not production, beliefs, political allegiance or existing
entity labels. It is a fictional naming convention, not a language-acquisition model.

## Three active options

Each changed concept has at most three pronounced forms. The original word begins
as the oldest option. New distinct forms enter at the end; a fourth removes the
oldest. Selection weights are 1, 2 and 4 from oldest to newest (1:2 with two options).
Thus a new word is favored without instantly replacing everything. An old form
remains available until displaced, rather than expiring on a timer. Duplicate
exposure does not add an entry or refresh its age.

Names choose forms using their stable entity key, meaning and word slot. At adoption, a seeded 50/50 choice retains the incoming pronunciation or applies
the recipient's sound-change rules. If those rules produce no distinct usable
form, pronunciation stays unchanged. This is a stylized loan adaptation, not a
phonetic reconstruction. Later uses retain the chosen archived pronunciation;
it is not rerolled each time the word appears. The linking particle is excluded from evolution. Descriptive
collision bynames use active lexical options too.

Every newly coined name stores the actual lexical choices and their provenance.
That snapshot survives retirement of the word. Previously registered labels are
returned verbatim; neither towns nor people get retroactive renamings. Archived
proto-roots remain available as etymological baselines even after leaving active
use for a concept. Language display names remain unchanged.

## Local sources

Annual updates allow at most one adoption per civilization. A deterministic 20%
annual opportunity considers borrowing, plus a bounded delivered-volume opportunity
as described below. Otherwise a separate one-in-seven gate considers local eponyms. An opportunity with no eligible
source makes no change. These rates and thresholds are game conventions:

- An occupied town at least five years old, with at least 1,000 kg cumulative output
  of the associated timber, metal, ceramics, grain, cloth or fish product.
- An active local institution at least five years old with at least 50 cumulative
  expenditure, associated with sanctuary, market, craft or learning by its purpose.
- A patron whose recorded service has ended, associated with guiding.
- A civilization's named actor with at least five recorded actions, associated with
  their recorded farming, navigation, craft or teaching occupation. The threshold
  measures activity, not proof of excellence in that profession.
- A surviving, accessible object with a recorded knowledge topic, associated with
  books. Lost, destroyed and remotely held objects are excluded.

Each eligible local source normally contributes its activity-associated concept.
There is a 25% chance instead to select a concept from the recorded meanings used
to create that source's name. Only existing, locally recorded etymologies with
known roots qualify; missing records fall back to the activity association. The
linking particle and untranslated disambiguators are excluded. For a metal town
named from moon + water, this permits rarer moon or water associations. Repeated
meanings are deduplicated before selection.

One eligible source/concept pair is selected; it contributes a non-particle word
fragment of up to 16 characters. This is not a fame or prestige model. Different
produced materials can give the same town several candidate associations, and
there is no claim that population size determines actual linguistic influence.

## Borrowing

An open, unflooded society route between occupied towns of different civilizations
provides a contact observation. Actual cargo receipts provide one too, even if the
route closes before the annual update. Multiple routes do not multiply annual credit.
After three consecutive annual observations, the neighbor's latest eponymic forms
become eligible for borrowing. An annual update with neither an eligible route nor a receipt resets
contact credit; temporary closures between annual observations are not integrated.

Delivered trade now changes both borrowing opportunities and source selection.
The market receipt path records the surviving delivered kg between distinct
civilizations, in both directions as an observation of contact. It adds no goods
or money. In-transit cargo contributes nothing. Delayed shipments contribute only
when received; lost mass does not count. Receipts accumulate in the archived
`Language.trade_kg` map until the next annual lexical update, then are consumed.

For annual total delivered kg K, a second seeded borrowing gate has probability
`floor(min(sqrt(K), 30)) / 100`. Combined with the baseline 20% gate, the maximum
opportunity is approximately 44%, not certainty. Among eligible neighbor/concept
pairs, a neighbor with volume k gets `1 + floor(min(sqrt(k / 100), 8))` tickets,
compared with one for a route-only neighbor. These saturating values are game
settings, not fitted communication rates. All goods count by kg; shipment splitting
does not multiply exposure. There is no cargo-value or bilingual-speaker model.
Borrowing retains the original namesake plus the immediately transmitting
civilization. Subsequent borrowing can
carry the word onward, but all civilizations read the previous annual vocabulary
snapshot: a word cannot traverse multiple edges within one annual update.

## Records and explorer

`Language.lexicon` stores active options; `contact_years` and `lexicon_month` preserve
contact memory and guard against processing an annual boundary twice. Missing old
archive fields default to empty state without inventing prior adoptions.
`NameRecord.words` stores the exact word choices; old records may have no such
snapshot. Adoption events retain the source entity and civilization IDs.

The civilization's naming panel exposes current words, selection weights, adoption
months, sources and borrowing origins. Name hover text includes the word choices
used when that name was created, even if the active vocabulary has since changed.

## Checks

The weighted fixture draws 7,000 choices from three words and observed counts
1,002 / 2,009 / 3,989, close to its configured 1:2:4 ratio. It verifies duplicate
suppression, oldest-option removal, existing-name stability and deterministic
serialization continuation.

A controlled three-civilization GPU history fixture uses two declared route edges.
It compares open and closed contact, requires at least three annual observations,
rejects same-year two-hop propagation, checks unchanged town labels, repeated
boundary idempotence and serialized continuation. This isolates the lexical
mechanism; it does not evaluate route construction itself.

The existing seeds 17, 81 and 256 naming fixture checks ordinary ten-year histories,
valid event references and seed 17's full-history save/resume versus monthly steps.

```sh
mise exec rust@1.89.0 -- cargo test --lib naming -- --include-ignored --nocapture
```

Initial evaluation, before pronunciation/etymology/volume changes: the ten-year worlds produced 5, 1 and 3 evolved civilization/concept
entries across seeds 17, 81 and 256 respectively. Several civilizations had none;
rare adoption is intentional. These totals do not imply natural borrowing occurred
in those small worlds; the controlled route fixture establishes that mechanism.
All eight naming tests passed, including the two GPU fixtures; all 50 ordinary
library tests passed (51 hardware tests ignored by that command). Clippy, formatting
and the source-artifact check passed. The first integrated run exposed missing
civilization/site subject validation for adoption events; those reference types
were added and the complete naming suite passed afterward.


## Pronunciation, etymology and volume checks

A 1,000-choice fixture with nonidentity sound rules retained pronunciation 482
times and adapted it 518 times. The metal-town/moon-water fixture selected metal
759 times, moon 113 and water 128. Each word records its incoming form, whether it
was adapted, and whether adoption followed activity, name etymology or borrowing.
The explorer displays these details and pending annual trade mass.

A real market-receipt fixture checks no credit while cargo is in transit, exactly
12 kg credit on arrival, no double credit on another market update, and conserved
economic quantities. A matched annual-opportunity experiment compares 10 kg and
10,000 kg of declared receipt exposure, holding the donor word and contact history
fixed. Across 200 matched opportunities, the low-volume case produced 45 borrowings
and the high-volume case produced 86. This latter experiment
isolates lexical response; it does not simulate production of those two volumes.

All ten naming tests passed, including three GPU fixtures on the Quadro RTX 5000
Max-Q/Vulkan. The ordinary ten-year seed runs now produced 4, 1 and 3 evolved
civilization/concept entries for seeds 17, 81 and 256; existing names and checkpoint
continuation remained stable. The ordinary library suite passed 51 tests with 52
hardware tests ignored. Clippy with warnings denied, formatting and repository
artifact checks passed. These fixtures verify the implemented game rules, not
empirical rates of linguistic change.
